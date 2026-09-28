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
            deck: Vec::new(),
            noble_phantasms: Vec::new(),
            np_status: crate::loader::NpStatus::Unavailable,
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

/// Adds compact gameplay metadata from a single Atlas nice-servant response.
pub fn enrich_servant(servant: &mut ServantRecord, row: &Value) -> Result<(), String> {
    use crate::loader::{NoblePhantasmRecord, NpStatus};
    if row.get("id").and_then(Value::as_u64) != Some(servant.id as u64) {
        return Err(format!(
            "Atlas returned the wrong servant for {}.",
            servant.name
        ));
    }
    let deck: Option<Vec<_>> = row
        .get("cards")
        .and_then(Value::as_array)
        .map(|cards| cards.iter().map(parse_card).collect())
        .flatten();
    let deck = deck.filter(|cards| cards.len() == 5).ok_or_else(|| {
        format!(
            "{} has no valid five-card deck in Atlas data.",
            servant.name
        )
    })?;
    servant.deck = deck;
    servant.noble_phantasms.clear();
    let Some(nps) = row
        .get("noblePhantasms")
        .and_then(Value::as_array)
        .filter(|nps| !nps.is_empty())
    else {
        servant.np_status = NpStatus::Unavailable;
        return Ok(());
    };
    let mut unsupported = false;
    let mut incomplete = false;
    for np in nps {
        let Some(functions) = np
            .get("functions")
            .and_then(Value::as_array)
            .filter(|f| !f.is_empty())
        else {
            incomplete = true;
            continue;
        };
        if functions
            .iter()
            .any(|function| function.get("funcType").and_then(Value::as_str).is_none())
        {
            incomplete = true;
            continue;
        }
        let damage_functions: Vec<_> = functions
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                f.get("funcType")
                    .and_then(Value::as_str)
                    .is_some_and(|t| t.starts_with("damage"))
            })
            .collect();
        if damage_functions.is_empty() {
            if np
                .get("effectFlags")
                .and_then(Value::as_array)
                .is_some_and(|flags| {
                    flags.iter().any(|flag| {
                        flag.as_str()
                            .is_some_and(|flag| flag.starts_with("attackEnemy"))
                    })
                })
            {
                unsupported = true;
            }
            continue;
        }
        if damage_functions.len() != 1 {
            unsupported = true;
            continue;
        }
        let (position, damage) = damage_functions[0];
        let function_type = damage["funcType"].as_str().unwrap_or("");
        // Conditional trait damage uses Value as its ordinary base; Correction is intentionally not applied.
        if !matches!(function_type, "damageNp" | "damageNpIndividual") {
            unsupported = true;
            continue;
        }
        let Some(values) = damage
            .get("svals")
            .and_then(Value::as_array)
            .filter(|v| v.len() == 5)
        else {
            incomplete = true;
            continue;
        };
        let multipliers: Option<Vec<f64>> = values
            .iter()
            .map(|v| {
                v.get("Value")
                    .and_then(Value::as_f64)
                    .filter(|v| v.is_finite() && *v > 0.0)
                    .map(|v| v / 1000.0)
            })
            .collect();
        let Some(multipliers) = multipliers else {
            incomplete = true;
            continue;
        };
        let Some(card_type) = np.get("card").and_then(parse_card) else {
            incomplete = true;
            continue;
        };
        let Some(id) = np
            .get("id")
            .and_then(Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
            .filter(|v| *v > 0)
        else {
            incomplete = true;
            continue;
        };
        let Some(name) = np
            .get("name")
            .and_then(Value::as_str)
            .filter(|n| !n.trim().is_empty())
        else {
            incomplete = true;
            continue;
        };
        let mut notes = Vec::new();
        if function_type != "damageNp" {
            notes.push("Conditional NP damage bonuses are excluded; this is base damage.".into());
        }
        if position > 0 {
            notes.push("NP effects before damage are not applied automatically; enter applicable buffs manually.".into());
        }
        if functions.len() > 1 {
            notes.push(
                "Other NP effects, including changes to later cards, are not simulated.".into(),
            );
        }
        if (2..=5).any(|oc| {
            damage
                .get(format!("svals{oc}"))
                .is_some_and(|other| other != &damage["svals"])
        }) {
            notes.push("Damage uses Overcharge 1; higher Overcharge effects are excluded.".into());
        }
        if servant
            .noble_phantasms
            .iter()
            .any(|existing| existing.id == id)
        {
            continue;
        }
        servant.noble_phantasms.push(NoblePhantasmRecord {
            id,
            name: format!(
                "{name} ({})",
                if np
                    .get("strengthStatus")
                    .and_then(Value::as_u64)
                    .unwrap_or(0)
                    >= 2
                {
                    "upgraded"
                } else {
                    "base"
                }
            ),
            card_type,
            multipliers: multipliers.try_into().expect("five values"),
            notes,
        });
    }
    servant.np_status = if !servant.noble_phantasms.is_empty() {
        if unsupported || incomplete {
            for np in &mut servant.noble_phantasms {
                np.notes.push("Some NP variants have unsupported or unavailable damage data and cannot be selected.".into());
            }
        }
        NpStatus::Damaging
    } else if unsupported {
        NpStatus::Unsupported
    } else if incomplete {
        NpStatus::Unavailable
    } else {
        NpStatus::Support
    };
    Ok(())
}

fn parse_card(value: &Value) -> Option<crate::model::CardType> {
    use crate::model::CardType;
    match value
        .as_str()
        .map(str::to_owned)
        .or_else(|| value.as_u64().map(|v| v.to_string()))
        .as_deref()
    {
        Some("1" | "arts") => Some(CardType::Arts),
        Some("2" | "buster") => Some(CardType::Buster),
        Some("3" | "quick") => Some(CardType::Quick),
        _ => None,
    }
}
