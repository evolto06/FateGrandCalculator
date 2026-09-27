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
                status: format!(
                    "Loaded {} local servants · updated {}.",
                    game_data.servants.len(),
                    format_retrieved_at(&game_data.retrieved_at)
                ),
                game_data: Ok(game_data),
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

pub fn format_retrieved_at(timestamp: &str) -> String {
    let Ok(timestamp) = timestamp.parse::<i64>() else {
        return timestamp.to_owned();
    };
    let days = timestamp.div_euclid(86_400);
    let seconds = timestamp.rem_euclid(86_400);
    let Some(date) = days.checked_add(719_468) else {
        return timestamp.to_string();
    };

    let era = if date >= 0 { date } else { date - 146_096 } / 146_097;
    let day_of_era = date - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);

    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02} UTC",
        seconds / 3_600,
        (seconds % 3_600) / 60
    )
}

fn bundled_fallback(status: String) -> StartupData {
    let game_data = GameData::bundled();
    let status = match &game_data {
        Ok(_) => status,
        Err(error) => format!("{status} Bundled data also failed to load: {error}"),
    };
    StartupData { game_data, status }
}
