mod atlas;
mod normalize;
mod store;

use crate::loader::GameData;

pub use normalize::NormalizationReport;

use atlas::AtlasClient;
use store::SnapshotStore;

pub struct StartupData {
    pub game_data: Result<GameData, String>,
    pub status: String,
}

pub struct UpdateReport {
    pub game_data: GameData,
    pub skipped_rows: usize,
}

pub struct ServantDataService;

impl ServantDataService {
    /// Loads a saved snapshot when available and otherwise uses the bundled seed.
    /// This method never makes a network request.
    pub fn load_local_or_bundled() -> StartupData {
        let store = match SnapshotStore::in_app_data_directory() {
            Ok(store) => store,
            Err(cache_error) => {
                return bundled_fallback(format!(
                    "Local servant data is unavailable ({cache_error}); using bundled data."
                ));
            }
        };

        match store.load() {
            Ok(Some(game_data)) => StartupData {
                game_data: Ok(game_data),
                status: "Loaded locally saved servant data.".into(),
            },
            Ok(None) => bundled_fallback(
                "Using bundled servant data. Choose Update servant data to fetch Atlas Academy."
                    .into(),
            ),
            Err(cache_error) => bundled_fallback(format!(
                "Saved servant data could not be read ({cache_error}); using bundled data."
            )),
        }
    }

    /// Downloads, normalizes, validates, and saves one complete snapshot.
    /// The existing snapshot is left untouched unless every step succeeds.
    pub fn update() -> Result<UpdateReport, String> {
        let store = SnapshotStore::in_app_data_directory()?;
        let client = AtlasClient::new()?;
        let payload = client.fetch_basic_export()?;
        let report = normalize::normalize_atlas_export(&payload)?;
        store.save(&report.game_data)?;

        Ok(UpdateReport {
            game_data: report.game_data,
            skipped_rows: report.skipped_rows,
        })
    }
}

fn bundled_fallback(status: String) -> StartupData {
    let game_data = GameData::bundled();
    let status = match &game_data {
        Ok(_) => status,
        Err(error) => format!("{status} Bundled data also failed to load: {error}"),
    };
    StartupData { game_data, status }
}
