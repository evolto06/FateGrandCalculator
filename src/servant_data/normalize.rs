use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use serde_json::Value;

use crate::loader::{GameData, ServantRecord};
use crate::model::{AttributeType, ClassType};

const REGION: &str = "NA";
const SOURCE_URL: &str = "https://api.atlasacademy.io/export/NA/basic_servant.json";

pub struct NormalizationReport {
    pub game_data: GameData,
    pub skipped_rows: usize,
}

#[derive(Deserialize)]
struct AtlasBasicServant {
    id: Option<u32>,
    name: Option<String>,
    #[serde(rename = "className")]
    class_name: Option<String>,
    attribute: Option<String>,
    #[serde(rename = "atkMax")]
    max_attack: Option<u32>,
    #[serde(rename = "lvMax", alias = "maxLevel")]
    max_level: Option<u32>,
}

pub fn normalize_atlas_export(payload: &Value) -> Result<NormalizationReport, String> {
    let rows = payload.as_array().ok_or_else(|| {
        "Atlas returned an unexpected servant export shape; expected a JSON array.".to_owned()
    })?;
    if rows.is_empty() {
        return Err("Atlas returned an empty servant export.".into());
    }

    let mut servants = Vec::with_capacity(rows.len());
    let mut skipped_rows: usize = 0;
    let mut ids = HashSet::with_capacity(rows.len());

    for row in rows {
        let Ok(source) = serde_json::from_value::<AtlasBasicServant>(row.clone()) else {
            skipped_rows += 1;
            continue;
        };

        let Some(id) = source.id.filter(|id| *id > 0) else {
            skipped_rows += 1;
            continue;
        };
        let Some(name) = source
            .name
            .map(|name| name.trim().to_owned())
            .filter(|name| !name.is_empty())
        else {
            skipped_rows += 1;
            continue;
        };
        let Some(class) = source.class_name.as_deref().and_then(parse_class) else {
            // The calculator intentionally excludes Beast and non-playable classes.
            skipped_rows += 1;
            continue;
        };
        let Some(attribute) = source.attribute.as_deref().and_then(parse_attribute) else {
            skipped_rows += 1;
            continue;
        };
        let Some(attack) = source.max_attack.filter(|attack| *attack > 0) else {
            skipped_rows += 1;
            continue;
        };
        if source.max_level == Some(0) {
            skipped_rows += 1;
            continue;
        }

        if !ids.insert(id) {
            return Err(format!(
                "Atlas returned duplicate servant ID {id}; the update was rejected."
            ));
        }

        servants.push(ServantRecord {
            id,
            name,
            level: source.max_level,
            attack,
            class,
            attribute,
            source_url: format!("https://api.atlasacademy.io/nice/{REGION}/servant/{id}"),
        });
    }

    if servants.is_empty() {
        return Err(
            "Atlas export contained no usable playable servants; the saved data was not changed."
                .into(),
        );
    }
    // A large skip ratio likely means Atlas changed its export schema.
    if skipped_rows.saturating_mul(5) > rows.len() {
        return Err(format!(
            "Atlas returned {} incomplete or unsupported entries out of {}; the update was rejected to protect the saved data.",
            skipped_rows,
            rows.len()
        ));
    }

    let retrieved_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("Could not determine the snapshot retrieval time: {error}"))?
        .as_secs()
        .to_string();
    let game_data = GameData {
        version: "atlas-basic-export-v1".into(),
        region: REGION.into(),
        retrieved_at,
        source: SOURCE_URL.into(),
        servants,
    };
    game_data.validate()?;

    Ok(NormalizationReport {
        game_data,
        skipped_rows,
    })
}

fn parse_class(value: &str) -> Option<ClassType> {
    match compact(value).as_str() {
        "saber" => Some(ClassType::Saber),
        "archer" => Some(ClassType::Archer),
        "lancer" => Some(ClassType::Lancer),
        "rider" => Some(ClassType::Rider),
        "caster" => Some(ClassType::Caster),
        "assassin" => Some(ClassType::Assassin),
        "berserker" => Some(ClassType::Berserker),
        "ruler" => Some(ClassType::Ruler),
        "avenger" => Some(ClassType::Avenger),
        "mooncancer" => Some(ClassType::MoonCancer),
        "alterego" => Some(ClassType::AlterEgo),
        "foreigner" => Some(ClassType::Foreigner),
        "pretender" => Some(ClassType::Pretender),
        "shielder" => Some(ClassType::Shielder),
        // Beast is encounter-specific in this calculator and is filtered here.
        _ => None,
    }
}

fn parse_attribute(value: &str) -> Option<AttributeType> {
    match value.trim().to_ascii_lowercase().as_str() {
        "earth" => Some(AttributeType::Earth),
        "sky" => Some(AttributeType::Sky),
        "human" | "man" => Some(AttributeType::Man),
        "star" => Some(AttributeType::Star),
        "beast" => Some(AttributeType::Beast),
        // Void and future unsupported values are left out rather than guessed.
        _ => None,
    }
}

fn compact(value: &str) -> String {
    value
        .chars()
        .filter(|character| !matches!(character, '_' | '-' | ' '))
        .flat_map(char::to_lowercase)
        .collect()
}
