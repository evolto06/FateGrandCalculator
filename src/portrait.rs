use std::io::{Cursor, Read};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use image::{ImageReader, Limits};
use reqwest::Url;
use reqwest::blocking::{Client, Response};
use reqwest::redirect::Policy;
use serde::Deserialize;

const MAX_METADATA_BYTES: usize = 4 * 1024 * 1024;
const MAX_PORTRAIT_BYTES: usize = 8 * 1024 * 1024;

/// Loads one servant portrait. Tests swap in a fake so they never touch the network.
type PortraitFetcher = fn(&str, u32) -> Result<ColorImage, String>;

pub struct PortraitStream {
    selected: Option<(String, u32)>,
    generation: u64,
    view: PortraitView,
    sender: Sender<PortraitResult>,
    receiver: Receiver<PortraitResult>,
    fetch: PortraitFetcher,
}

enum PortraitView {
    Missing,
    Loading,
    Ready(TextureHandle),
    Failed(String),
}

struct PortraitResult {
    generation: u64,
    image: Result<ColorImage, String>,
}

#[derive(Deserialize)]
struct AtlasPortrait {
    id: u32,
    #[serde(rename = "extraAssets")]
    extra_assets: Option<AtlasExtraAssets>,
}

#[derive(Deserialize)]
struct AtlasExtraAssets {
    #[serde(rename = "charaGraph")]
    chara_graph: Option<AtlasCharaGraph>,
}

#[derive(Deserialize)]
struct AtlasCharaGraph {
    ascension: std::collections::HashMap<String, String>,
}

impl Default for PortraitStream {
    fn default() -> Self {
        // Test builds never reach the network, even through the whole app.
        #[cfg(test)]
        return Self::with_fetcher(offline_fetch);
        #[cfg(not(test))]
        Self::with_fetcher(fetch_portrait)
    }
}

#[cfg(test)]
fn offline_fetch(_region: &str, _id: u32) -> Result<ColorImage, String> {
    Ok(ColorImage::new([1, 1], vec![egui::Color32::BLACK]))
}

impl PortraitStream {
    fn with_fetcher(fetch: PortraitFetcher) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            selected: None,
            generation: 0,
            view: PortraitView::Missing,
            sender,
            receiver,
            fetch,
        }
    }

    pub fn sync_selection(&mut self, selection: Option<(&str, u32)>, context: &egui::Context) {
        self.receive(context);

        let selection = selection.map(|(region, id)| (region.to_owned(), id));
        if self.selected == selection {
            return;
        }

        self.selected = selection.clone();
        self.generation = self.generation.wrapping_add(1);
        self.view = PortraitView::Missing;

        let Some((region, id)) = selection else {
            return;
        };

        self.view = PortraitView::Loading;
        let generation = self.generation;
        let sender = self.sender.clone();
        let fetch = self.fetch;
        let context = context.clone();
        let worker = thread::Builder::new()
            .name("servant-portrait".into())
            .spawn(move || {
                let image = std::panic::catch_unwind(|| fetch(&region, id))
                    .unwrap_or_else(|_| Err("Portrait loading stopped unexpectedly.".into()));
                let _ = sender.send(PortraitResult { generation, image });
                context.request_repaint();
            });

        if let Err(error) = worker {
            self.view = PortraitView::Failed(format!("Could not start portrait loading: {error}"));
        }
    }

    pub fn texture(&self) -> Option<&TextureHandle> {
        match &self.view {
            PortraitView::Ready(texture) => Some(texture),
            _ => None,
        }
    }

    pub fn message(&self) -> &'static str {
        match self.view {
            PortraitView::Loading => "Loading\nportrait…",
            PortraitView::Failed(_) => "Portrait\nunavailable",
            PortraitView::Missing | PortraitView::Ready(_) => "No portrait\navailable",
        }
    }

    pub fn error(&self) -> Option<&str> {
        match &self.view {
            PortraitView::Failed(error) => Some(error),
            _ => None,
        }
    }

    fn receive(&mut self, context: &egui::Context) {
        while let Ok(result) = self.receiver.try_recv() {
            if result.generation != self.generation {
                continue;
            }

            self.view = match result.image {
                Ok(image) => PortraitView::Ready(context.load_texture(
                    "selected-servant-portrait",
                    image,
                    TextureOptions::LINEAR,
                )),
                Err(error) => PortraitView::Failed(error),
            };
        }
    }
}

fn fetch_portrait(region: &str, id: u32) -> Result<ColorImage, String> {
    if !matches!(region, "NA" | "JP") {
        return Err("This servant region does not support portrait streaming.".into());
    }

    let client = Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(15))
        .redirect(Policy::none())
        .build()
        .map_err(|error| format!("Could not start portrait request: {error}"))?;

    let metadata_url = format!("https://api.atlasacademy.io/nice/{region}/servant/{id}");
    let metadata_response = client
        .get(&metadata_url)
        .send()
        .map_err(|error| format!("Could not load portrait details: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Atlas portrait details were unavailable: {error}"))?;
    let metadata = read_response(metadata_response, MAX_METADATA_BYTES)?;
    let portrait_url = portrait_url(&metadata, id)?;

    let image_response = client
        .get(portrait_url)
        .send()
        .map_err(|error| format!("Could not stream the portrait: {error}"))?
        .error_for_status()
        .map_err(|error| format!("The portrait image was unavailable: {error}"))?;
    let bytes = read_response(image_response, MAX_PORTRAIT_BYTES)?;
    decode_portrait(bytes)
}

/// Picks the first-ascension portrait URL out of Atlas servant details, accepting
/// only HTTPS links on Atlas Academy's asset hosts.
fn portrait_url(metadata: &[u8], id: u32) -> Result<Url, String> {
    let details: AtlasPortrait = serde_json::from_slice(metadata)
        .map_err(|error| format!("Atlas returned invalid portrait details: {error}"))?;
    if details.id != id {
        return Err("Atlas returned portrait details for a different servant.".into());
    }

    let first_ascension = details
        .extra_assets
        .and_then(|assets| assets.chara_graph)
        .and_then(|graph| graph.ascension.get("1").cloned())
        .ok_or_else(|| "Atlas has no first ascension portrait for this servant.".to_owned())?;
    let portrait_url = Url::parse(&first_ascension)
        .map_err(|error| format!("Atlas returned an invalid portrait URL: {error}"))?;
    if portrait_url.scheme() != "https"
        || !matches!(
            portrait_url.host_str(),
            Some("static.atlasacademy.io" | "assets.atlasacademy.io")
        )
    {
        return Err("Atlas returned an unsupported portrait URL.".into());
    }
    Ok(portrait_url)
}

fn decode_portrait(bytes: Vec<u8>) -> Result<ColorImage, String> {
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|error| format!("Could not identify portrait format: {error}"))?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(2048);
    limits.max_image_height = Some(2048);
    limits.max_alloc = Some(32 * 1024 * 1024);
    reader.limits(limits);

    let rgba = reader
        .decode()
        .map_err(|error| format!("Could not decode the portrait: {error}"))?
        .to_rgba8();
    Ok(ColorImage::from_rgba_unmultiplied(
        [rgba.width() as usize, rgba.height() as usize],
        rgba.as_raw(),
    ))
}

fn read_response(response: Response, maximum: usize) -> Result<Vec<u8>, String> {
    let length = response.content_length();
    read_bounded(response, length, maximum)
}

/// Reads at most `maximum` bytes, rejecting a body that declares or turns out
/// to be larger.
fn read_bounded(
    body: impl Read,
    content_length: Option<u64>,
    maximum: usize,
) -> Result<Vec<u8>, String> {
    if content_length.is_some_and(|length| length > maximum as u64) {
        return Err("Atlas returned an oversized portrait response.".into());
    }

    let mut bytes = Vec::new();
    body.take((maximum + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Could not read the portrait response: {error}"))?;
    if bytes.len() > maximum {
        return Err("Atlas returned an oversized portrait response.".into());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn fake_fetch(region: &str, id: u32) -> Result<ColorImage, String> {
        match id {
            1 => Ok(ColorImage::new([2, 3], vec![egui::Color32::WHITE; 6])),
            _ => Err(format!("fake failure for {region} {id}")),
        }
    }

    fn settle(stream: &mut PortraitStream, context: &egui::Context) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while matches!(stream.view, PortraitView::Loading) {
            assert!(Instant::now() < deadline, "portrait worker did not finish");
            thread::sleep(Duration::from_millis(5));
            stream.receive(context);
        }
    }

    #[test]
    fn selecting_a_servant_loads_its_portrait_texture() {
        let context = egui::Context::default();
        let mut stream = PortraitStream::with_fetcher(fake_fetch);
        assert_eq!(stream.message(), "No portrait\navailable");

        stream.sync_selection(Some(("NA", 1)), &context);
        assert_eq!(stream.message(), "Loading\nportrait…");
        settle(&mut stream, &context);

        let texture = stream.texture().expect("portrait texture");
        assert_eq!(texture.size(), [2, 3]);
        assert_eq!(stream.error(), None);
    }

    #[test]
    fn failed_fetch_reports_an_error_and_clearing_resets_the_view() {
        let context = egui::Context::default();
        let mut stream = PortraitStream::with_fetcher(fake_fetch);

        stream.sync_selection(Some(("JP", 2)), &context);
        settle(&mut stream, &context);
        assert_eq!(stream.message(), "Portrait\nunavailable");
        assert_eq!(stream.error(), Some("fake failure for JP 2"));

        stream.sync_selection(None, &context);
        assert!(stream.texture().is_none());
        assert_eq!(stream.error(), None);
    }

    #[test]
    fn results_for_an_earlier_selection_are_ignored() {
        let context = egui::Context::default();
        let mut stream = PortraitStream::with_fetcher(fake_fetch);

        stream.sync_selection(Some(("NA", 1)), &context);
        stream.sync_selection(Some(("NA", 2)), &context);
        settle(&mut stream, &context);
        // Give the first worker time to deliver its stale result too.
        thread::sleep(Duration::from_millis(50));
        stream.receive(&context);

        assert!(stream.texture().is_none());
        assert_eq!(stream.error(), Some("fake failure for NA 2"));
    }

    #[test]
    fn unsupported_regions_are_rejected_without_a_request() {
        let error = fetch_portrait("KR", 1).unwrap_err();
        assert!(error.contains("does not support"), "{error}");
    }

    fn details(id: u32, url: &str) -> Vec<u8> {
        format!(r#"{{"id":{id},"extraAssets":{{"charaGraph":{{"ascension":{{"1":"{url}"}}}}}}}}"#)
            .into_bytes()
    }

    #[test]
    fn portrait_url_accepts_only_https_atlas_assets_for_the_same_servant() {
        let good = "https://static.atlasacademy.io/NA/CharaGraph/100100/100100a.png";
        assert_eq!(portrait_url(&details(7, good), 7).unwrap().as_str(), good);

        let wrong_servant = portrait_url(&details(8, good), 7).unwrap_err();
        assert!(
            wrong_servant.contains("different servant"),
            "{wrong_servant}"
        );

        for url in [
            "http://static.atlasacademy.io/a.png",
            "https://example.com/a.png",
        ] {
            let error = portrait_url(&details(7, url), 7).unwrap_err();
            assert!(error.contains("unsupported portrait URL"), "{error}");
        }

        let missing = portrait_url(br#"{"id":7}"#, 7).unwrap_err();
        assert!(missing.contains("no first ascension"), "{missing}");
        let invalid = portrait_url(b"not json", 7).unwrap_err();
        assert!(invalid.contains("invalid portrait details"), "{invalid}");
    }

    #[test]
    fn read_bounded_rejects_declared_and_actual_oversized_bodies() {
        assert_eq!(read_bounded(&b"abcd"[..], Some(4), 4).unwrap(), b"abcd");
        assert_eq!(read_bounded(&b"abcd"[..], None, 4).unwrap(), b"abcd");
        assert!(read_bounded(&b"ab"[..], Some(5), 4).is_err());
        assert!(read_bounded(&b"abcde"[..], None, 4).is_err());
    }

    #[test]
    fn decode_portrait_rejects_non_images() {
        let error = decode_portrait(b"not an image".to_vec()).unwrap_err();
        assert!(error.contains("portrait"), "{error}");
    }
}
