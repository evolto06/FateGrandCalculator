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

pub struct PortraitStream {
    selected: Option<(String, u32)>,
    generation: u64,
    view: PortraitView,
    sender: Sender<PortraitResult>,
    receiver: Receiver<PortraitResult>,
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
        let (sender, receiver) = mpsc::channel();
        Self {
            selected: None,
            generation: 0,
            view: PortraitView::Missing,
            sender,
            receiver,
        }
    }
}

impl PortraitStream {
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
        let context = context.clone();
        let worker = thread::Builder::new()
            .name("servant-portrait".into())
            .spawn(move || {
                let image = std::panic::catch_unwind(|| fetch_portrait(&region, id))
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
    let metadata = read_bounded(metadata_response, MAX_METADATA_BYTES)?;
    let details: AtlasPortrait = serde_json::from_slice(&metadata)
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

    let image_response = client
        .get(portrait_url)
        .send()
        .map_err(|error| format!("Could not stream the portrait: {error}"))?
        .error_for_status()
        .map_err(|error| format!("The portrait image was unavailable: {error}"))?;
    let bytes = read_bounded(image_response, MAX_PORTRAIT_BYTES)?;

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

fn read_bounded(response: Response, maximum: usize) -> Result<Vec<u8>, String> {
    if response
        .content_length()
        .is_some_and(|length| length > maximum as u64)
    {
        return Err("Atlas returned an oversized portrait response.".into());
    }

    let mut bytes = Vec::new();
    response
        .take((maximum + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Could not read the portrait response: {error}"))?;
    if bytes.len() > maximum {
        return Err("Atlas returned an oversized portrait response.".into());
    }
    Ok(bytes)
}
