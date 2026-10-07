use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use serde_json::Value;

use crate::loader::{GameData, ServantRecord};
use crate::model::{AttributeType, ClassType};
use crate::np_mechanics::components::{NpDamageComponent, NpDamageValues, NpTarget};
use crate::np_mechanics::enemy_status::{EnemyStatus, EnemyStatusScaling};
use crate::np_mechanics::low_hp::LowHpScaling;
use crate::np_mechanics::trait_bonus::{TraitBonusScaling, TraitCondition};
use crate::np_mechanics::{self, NpDamageKind};

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
    #[serde(rename = "hpMax")]
    max_hp: Option<u32>,
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
        if source.max_level == Some(0) || source.max_hp == Some(0) {
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
            max_hp: source.max_hp,
            class,
            attribute,
            deck: Vec::new(),
            deck_note: None,
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
        "beast" => Some(ClassType::Beast),
        "beasteresh" => Some(ClassType::BeastEresh),
        "beasti" => Some(ClassType::BeastI),
        "beastii" => Some(ClassType::BeastII),
        "beastiiil" => Some(ClassType::BeastIIIL),
        "beastiiir" => Some(ClassType::BeastIIIR),
        "beastiv" => Some(ClassType::BeastIV),
        "loregrandcaster" => Some(ClassType::LoreGrandCaster),
        "uolgamarieflarecollection" => Some(ClassType::OlgaMarieFlareCollection),
        "uolgamarieaquacollection" => Some(ClassType::OlgaMarieAquaCollection),
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
    // Atlas hpMax is the natural maximum-level statistic (hpGrowth[lvMax - 1]
    // in the audited responses), before player-entered Fous or grail levels.
    if let Some(hp) = row.get("hpMax") {
        servant.max_hp = Some(
            hp.as_u64()
                .and_then(|hp| u32::try_from(hp).ok())
                .filter(|hp| *hp > 0)
                .ok_or_else(|| {
                    format!("{} has an invalid maximum HP in Atlas data.", servant.name)
                })?,
        );
    }
    let cards = row
        .get("cards")
        .and_then(Value::as_array)
        .filter(|cards| cards.len() == 5)
        .ok_or_else(|| {
            format!(
                "{} has no valid five-card deck in Atlas data.",
                servant.name
            )
        })?;
    match cards.iter().map(parse_card).collect::<Option<Vec<_>>>() {
        Some(deck) => {
            servant.deck = deck;
            servant.deck_note = None;
        }
        None if servant.class == ClassType::BeastIV
            && cards
                .iter()
                .all(|card| card.as_str() == Some("10") || card.as_u64() == Some(10)) =>
        {
            servant.deck.clear();
            servant.deck_note = Some("Atlas lists only special card type 10 for Beast IV. Three-card damage cannot be calculated for this servant.".into());
        }
        None => {
            return Err(format!(
                "{} has unsupported card types in Atlas data.",
                servant.name
            ));
        }
    }
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
        let function_types: Vec<_> = damage_functions
            .iter()
            .map(|(_, damage)| damage["funcType"].as_str().unwrap_or(""))
            .collect();
        let damage_kind = np_mechanics::classify_functions(&function_types, servant.id);
        // Only audited variants may use the verified consecutive component path.
        let np_id = np
            .get("id")
            .and_then(Value::as_u64)
            .and_then(|id| u32::try_from(id).ok());
        let multi = damage_kind == NpDamageKind::MultipleComponents;
        if multi
            && !(np_id
                .is_some_and(|id| np_mechanics::multi_import::is_verified_variant(servant.id, id))
                && np_mechanics::multi_import::is_verified_shape(&damage_functions))
        {
            unsupported = true;
            continue;
        }
        if !multi
            && !matches!(
                damage_kind,
                NpDamageKind::Standard
                    | NpDamageKind::TraitBase
                    | NpDamageKind::DefensePierce
                    | NpDamageKind::SpaceEresh
                    | NpDamageKind::HpRatioLow
                    | NpDamageKind::StateIndividualFix
            )
        {
            unsupported = true;
            continue;
        }
        let (position, damage) = damage_functions[0];
        let function_type = damage["funcType"].as_str().unwrap_or("");
        let affection_np = damage_kind == NpDamageKind::SpaceEresh;
        let low_hp_np = damage_kind == NpDamageKind::HpRatioLow;
        let status_np = damage_kind == NpDamageKind::StateIndividualFix;
        if low_hp_np && !is_supported_damage_shape(np, damage) {
            unsupported = true;
            continue;
        }
        let status_condition = if status_np {
            let Some(condition) = np_id.and_then(|id| verified_enemy_status(servant.id, id)) else {
                unsupported = true;
                continue;
            };
            let expected_target = if matches!(condition, EnemyStatus::Bind | EnemyStatus::Charm) {
                "enemyAll"
            } else {
                "enemy"
            };
            if !is_supported_damage_shape(np, damage)
                || damage["funcTargetType"].as_str() != Some(expected_target)
            {
                unsupported = true;
                continue;
            }
            Some(condition)
        } else {
            None
        };
        let mut components = Vec::new();
        let mut bad_data = false;
        let mut bad_condition = false;
        for (_, function) in &damage_functions {
            let target = match function.get("funcTargetType").and_then(Value::as_str) {
                Some("enemy") => Some(NpTarget::Enemy),
                Some("enemyAll") => Some(NpTarget::EnemyAll),
                None if !multi && !low_hp_np && !status_np => None,
                _ => {
                    bad_condition = true;
                    break;
                }
            };
            let mut overcharge: [Option<NpDamageValues>; 5] = std::array::from_fn(|_| None);
            for (oc, output) in overcharge.iter_mut().enumerate() {
                let key = if oc == 0 {
                    "svals".to_owned()
                } else {
                    format!("svals{}", oc + 1)
                };
                if let Some(values) = function.get(&key) {
                    match import_damage_values(values, function_type, multi, status_condition) {
                        Ok(row) => *output = Some(row),
                        Err(ImportValuesError::Unsupported) => {
                            if oc == 0 {
                                bad_condition = true;
                            }
                        }
                        Err(ImportValuesError::Incomplete) => {
                            if oc == 0 {
                                bad_data = true;
                            }
                        }
                    }
                } else if oc == 0 {
                    bad_data = true;
                }
            }
            components.push(NpDamageComponent { target, overcharge });
        }
        if bad_condition {
            unsupported = true;
            continue;
        }
        if bad_data {
            incomplete = true;
            continue;
        }
        // Base rates stay usable when the conditional contract is unsupported.
        // Enable the bonus only when every supplied source row is audited and
        // keeps the same condition across NP levels and Overcharge.
        let trait_condition = if damage_kind == NpDamageKind::TraitBase {
            import_trait_bonus(np, damage, &mut components[0])
        } else {
            None
        };
        // A row is usable only when every component explicitly supplies it.
        for oc in 0..5 {
            let complete = components
                .iter()
                .all(|component| component.overcharge[oc].is_some());
            let active = complete
                && (0..5).all(|level| {
                    components.iter().any(|component| {
                        let row = component.overcharge[oc].as_ref().unwrap();
                        row.rates[level] == 1000 && row.multipliers[level] > 0.0
                    })
                });
            if !active {
                for component in &mut components {
                    component.overcharge[oc] = None;
                }
            }
        }
        if components
            .iter()
            .any(|component| component.overcharge[0].is_none())
        {
            incomplete = true;
            continue;
        }
        let affection = if affection_np {
            let Some(scaling) = np_mechanics::space_eresh::import_scaling(damage, servant.id)
            else {
                incomplete = true;
                continue;
            };
            for oc in 1..5 {
                if components[0].overcharge[oc].is_some() {
                    let mut other = damage.clone();
                    other["svals"] = damage[format!("svals{}", oc + 1)].clone();
                    let same = np_mechanics::space_eresh::import_scaling(&other, servant.id)
                        .is_some_and(|other| {
                            other.base == scaling.base
                                && other.per_level == scaling.per_level
                                && other.max_level == scaling.max_level
                                && other.ignore_defense_at == scaling.ignore_defense_at
                        });
                    if !same {
                        components[0].overcharge[oc] = None;
                    }
                }
            }
            Some(scaling)
        } else {
            None
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
        if function_type == "damageNpIndividual" {
            notes.push(if trait_condition.is_some() {
                "Set the matching enemy trait at NP damage time. Preceding trait-granting effects must be accounted for manually; other NP effects are not simulated."
            } else {
                "Conditional NP trait bonus metadata is unsupported or unavailable; this is base damage. Update servant data for audited bonuses."
            }.into());
        }
        if affection_np {
            notes.push("Set the affection level at the moment NP damage lands. Higher Overcharge can raise the gauge before damage; adjust the level manually.".into());
        }
        if low_hp_np {
            notes.push("Set attacker HP at NP damage time, including preceding HP loss or healing. HP changes and other NP effects are not simulated.".into());
        }
        if status_np {
            notes.push("Set the matching enemy status at NP damage time. Status application and success are not simulated; preceding effects must actually land, and effects after damage cannot activate this NP's bonus.".into());
        }
        if damage_kind == NpDamageKind::DefensePierce {
            notes.push(
                "Ignores positive enemy Defense; Defense Down still increases damage.".into(),
            );
        }
        if position > 0 {
            notes.push(if affection_np {
                "Other NP effects before damage are not applied automatically; enter applicable buffs manually."
            } else {
                "NP effects before damage are not applied automatically; enter applicable buffs manually."
            }.into());
        }
        if functions.len() > 1 {
            notes.push(
                "Other NP effects, including changes to later cards, are not simulated.".into(),
            );
        }
        if components[0].overcharge.iter().skip(1).any(Option::is_none) {
            notes.push("Some Overcharge rows are unavailable. Update servant data to fetch complete values; missing rows are never inferred.".into());
        }
        if multi {
            match servant.id {
                201300 => notes.push("Arash's self-sacrifice and its effects on later cards are not simulated.".into()),
                504400 => notes.push("Chen Gong's NP assumes an eligible ally is available; ally sacrifice and its effects on later cards are not simulated.".into()),
                305400 => notes.push("Bhima removes enemy buffs before damage; enter the resulting enemy Defense manually. Buff removal and later effects are not simulated.".into()),
                _ => {}
            }
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
            components,
            affection,
            defense_pierce: damage_kind == NpDamageKind::DefensePierce,
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

#[derive(Debug)]
enum ImportValuesError {
    Incomplete,
    Unsupported,
}

fn import_damage_values(
    values: &Value,
    function_type: &str,
    multi: bool,
    status_condition: Option<EnemyStatus>,
) -> Result<NpDamageValues, ImportValuesError> {
    let values = values
        .as_array()
        .filter(|values| values.len() == 5)
        .ok_or(ImportValuesError::Incomplete)?;
    let mut row = NpDamageValues::guaranteed([0.0; 5]);
    let low_hp = function_type == "damageNpHpratioLow";
    let mut scaling = LowHpScaling {
        source_base_rates: [0; 5],
        coefficients: [0; 5],
    };
    let mut status_scaling = status_condition.map(|condition| EnemyStatusScaling {
        condition,
        source_corrections: [0; 5],
        include_ignore_individuality: condition.includes_ignored_individuality(),
    });
    for (level, value) in values.iter().enumerate() {
        let object = value.as_object().ok_or(ImportValuesError::Incomplete)?;
        // The ordinary trait and affection fields have separate, explicit handling.
        let allowed: &[&str] = match function_type {
            "damageNpHpratioLow" => &["Value", "Rate", "Target"],
            "damageNpStateIndividualFix" => &[
                "Value",
                "Rate",
                "Target",
                "Correction",
                "IncludeIgnoreIndividuality",
            ],
            "damageNpIndividual" => &[
                "Value",
                "Rate",
                "Target",
                "Correction",
                "IncludeIgnoreIndividuality",
                "IgnoreIndividuality",
            ],
            "damageNpBattlePointPhase" => &["Value", "Rate", "Value2", "Correction", "Target"],
            _ => &["Value", "Rate", "HideMiss", "HideNoEffect", "CheckDead"],
        };
        if object.keys().any(|key| !allowed.contains(&key.as_str())) {
            return Err(ImportValuesError::Unsupported);
        }
        if let Some(scaling) = &mut status_scaling {
            if value.get("Target").and_then(Value::as_u64)
                != Some(u64::from(scaling.condition.source_target()))
            {
                return Err(ImportValuesError::Unsupported);
            }
            let include = match value.get("IncludeIgnoreIndividuality") {
                None => false,
                Some(flag) => match flag.as_u64() {
                    Some(0) => false,
                    Some(1) => true,
                    _ => return Err(ImportValuesError::Unsupported),
                },
            };
            if include != scaling.include_ignore_individuality
                || value.get("Rate").and_then(Value::as_u64) != Some(1000)
            {
                return Err(ImportValuesError::Unsupported);
            }
            for key in ["Value", "Correction"] {
                let source = value
                    .get(key)
                    .and_then(Value::as_u64)
                    .and_then(|source| u32::try_from(source).ok())
                    .filter(|source| *source > 0)
                    .ok_or(ImportValuesError::Incomplete)?;
                if key == "Correction" {
                    scaling.source_corrections[level] = source;
                }
            }
        }
        if low_hp {
            let source_value = value
                .get("Value")
                .and_then(Value::as_u64)
                .and_then(|value| u32::try_from(value).ok())
                .filter(|value| *value > 0)
                .ok_or(ImportValuesError::Incomplete)?;
            let coefficient = value
                .get("Target")
                .and_then(Value::as_u64)
                .and_then(|value| u32::try_from(value).ok())
                .ok_or(ImportValuesError::Incomplete)?;
            if value.get("Rate").and_then(Value::as_u64) != Some(1000) {
                return Err(ImportValuesError::Unsupported);
            }
            scaling.source_base_rates[level] = source_value;
            scaling.coefficients[level] = coefficient;
        }
        let multiplier = value
            .get("Value")
            .and_then(Value::as_f64)
            .filter(|value| value.is_finite() && *value >= 0.0)
            .ok_or(ImportValuesError::Incomplete)?
            / 1000.0;
        let rate = match value.get("Rate") {
            None => 1000,
            Some(rate) => rate
                .as_u64()
                .filter(|rate| matches!(rate, 0 | 1000))
                .ok_or(ImportValuesError::Unsupported)? as u16,
        };
        if (!multi && multiplier <= 0.0) || (rate == 0 && multiplier != 0.0) {
            return Err(ImportValuesError::Incomplete);
        }
        let check_dead = match value.get("CheckDead") {
            None => false,
            Some(flag) => match flag.as_u64() {
                Some(0) => false,
                Some(1) => true,
                _ => return Err(ImportValuesError::Unsupported),
            },
        };
        row.multipliers[level] = multiplier;
        row.rates[level] = rate;
        row.check_dead[level] = check_dead;
    }
    if low_hp {
        row.low_hp = Some(scaling);
    }
    row.enemy_status = status_scaling;
    Ok(row)
}

/// An audited simple `damageNpIndividual` contract, separate from base parsing.
fn import_trait_bonus(
    np: &Value,
    function: &Value,
    component: &mut NpDamageComponent,
) -> Option<TraitCondition> {
    if !is_supported_damage_shape(np, function) {
        return None;
    }
    let mut condition = None;
    let mut scalings: [Option<TraitBonusScaling>; 5] = std::array::from_fn(|_| None);
    for (oc, scaling) in scalings.iter_mut().enumerate() {
        let key = if oc == 0 {
            "svals".to_owned()
        } else {
            format!("svals{}", oc + 1)
        };
        let Some(values) = function.get(&key) else {
            continue;
        };
        let values = values.as_array().filter(|values| values.len() == 5)?;
        let mut corrections = [0; 5];
        for (level, value) in values.iter().enumerate() {
            let object = value.as_object()?;
            if object.len() != 4
                || object
                    .keys()
                    .any(|key| !["Value", "Rate", "Target", "Correction"].contains(&key.as_str()))
                || value["Rate"].as_u64() != Some(1000)
                || value["Value"].as_u64().filter(|value| *value > 0).is_none()
            {
                return None;
            }
            let target = value["Target"]
                .as_i64()
                .and_then(|target| i32::try_from(target).ok())?;
            let row_condition = TraitCondition::from_source_target(target)?;
            if condition.is_some_and(|condition| condition != row_condition) {
                return None;
            }
            condition = Some(row_condition);
            corrections[level] = value["Correction"]
                .as_u64()
                .and_then(|value| u32::try_from(value).ok())
                .filter(|value| *value > 0)?;
        }
        *scaling = Some(TraitBonusScaling {
            condition: condition?,
            source_corrections: corrections,
        });
    }
    let condition = condition?;
    for (row, scaling) in component.overcharge.iter_mut().zip(scalings) {
        if let Some(row) = row {
            row.trait_bonus = scaling;
        }
    }
    Some(condition)
}

/// Only these NA variants and their verified matching policies were audited.
fn verified_enemy_status(servant_id: u32, np_id: u32) -> Option<EnemyStatus> {
    match (servant_id, np_id) {
        (200300, 200301) => Some(EnemyStatus::Poison),
        (504800, 504801) => Some(EnemyStatus::SkillSeal),
        (604900, 604901) => Some(EnemyStatus::Bind),
        (704900, 704901) => Some(EnemyStatus::DefenseUp),
        (1101100, 1101101) => Some(EnemyStatus::Charm),
        (1101400, 1101401) => Some(EnemyStatus::Curse),
        (2500400, 2500401) => Some(EnemyStatus::Burn),
        _ => None,
    }
}

/// Low-HP and status parameters are handled separately in the value rows.
/// Additional function conditions, scripts and unsupported targeting must not
/// become an apparently ordinary NP. Other effects remain manual notes.
fn is_supported_damage_shape(np: &Value, function: &Value) -> bool {
    let Some(object) = function.as_object() else {
        return false;
    };
    let fields = [
        "funcId",
        "funcType",
        "funcTargetType",
        "funcTargetTeam",
        "funcPopupText",
        "functvals",
        "overWriteTvalsList",
        "funcquestTvals",
        "funcGroup",
        "traitVals",
        "buffs",
        "script",
        "svals",
        "svals2",
        "svals3",
        "svals4",
        "svals5",
    ];
    object.keys().all(|key| fields.contains(&key.as_str()))
        && matches!(
            function["funcTargetType"].as_str(),
            Some("enemy" | "enemyAll")
        )
        && function
            .get("funcTargetTeam")
            .is_none_or(|team| team.as_str() == Some("playerAndEnemy"))
        && [
            "functvals",
            "overWriteTvalsList",
            "funcquestTvals",
            "funcGroup",
            "traitVals",
            "buffs",
        ]
        .iter()
        .all(|key| {
            function
                .get(*key)
                .is_none_or(|value| value.as_array().is_some_and(Vec::is_empty))
        })
        && [np, function].iter().all(|value| {
            value
                .get("script")
                .is_none_or(|script| script.as_object().is_some_and(serde_json::Map::is_empty))
        })
}
