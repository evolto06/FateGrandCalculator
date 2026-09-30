use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::damage::DamageInput;
use crate::model::{AttributeType, CardType, ClassType};
pub use crate::np_mechanics::space_eresh::AffectionScaling;

const BUNDLED_GAME_DATA: &str = include_str!("../data/game_data.json");

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameData {
    pub version: String,
    pub region: String,
    pub retrieved_at: String,
    pub source: String,
    pub servants: Vec<ServantRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServantRecord {
    pub id: u32,
    pub name: String,
    pub level: Option<u32>,
    pub attack: u32,
    pub class: ClassType,
    pub attribute: AttributeType,
    pub source_url: String,
    #[serde(default)]
    pub deck: Vec<CardType>,
    #[serde(default)]
    pub deck_note: Option<String>,
    #[serde(default)]
    pub noble_phantasms: Vec<NoblePhantasmRecord>,
    #[serde(default)]
    pub np_status: NpStatus,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NpStatus {
    Damaging,
    Support,
    #[default]
    Unavailable,
    Unsupported,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NoblePhantasmRecord {
    pub id: u32,
    pub name: String,
    pub card_type: CardType,
    pub multipliers: [f64; 5],
    #[serde(default)]
    pub defense_pierce: bool,
    #[serde(default)]
    pub affection: Option<AffectionScaling>,
    #[serde(default)]
    pub notes: Vec<String>,
}

impl ServantRecord {
    pub fn normal_card_input(
        &self,
        card_type: CardType,
        attack_buff: f64,
        card_buff: f64,
        enemy_defense: f64,
        enemy_class: ClassType,
        enemy_attribute: AttributeType,
    ) -> DamageInput {
        DamageInput {
            attack: self.attack,
            card_type,
            attacker_class: self.class,
            attack_buff,
            card_buff,
            enemy_defense,
            class_multiplier: self.class.affinity_against(enemy_class),
            attribute_multiplier: self.attribute.affinity_against(enemy_attribute),
        }
    }
}

impl GameData {
    pub fn bundled() -> Result<Self, String> {
        let data: Self = serde_json::from_str(BUNDLED_GAME_DATA)
            .map_err(|error| format!("Could not read bundled game data: {error}"))?;
        data.validate()?;
        Ok(data)
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        let data: Self = serde_json::from_str(json)
            .map_err(|error| format!("Could not read servant snapshot: {error}"))?;
        data.validate()?;
        Ok(data)
    }

    pub fn servant(&self, id: u32) -> Option<&ServantRecord> {
        self.servants.iter().find(|servant| servant.id == id)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.version.trim().is_empty()
            || self.region.trim().is_empty()
            || self.retrieved_at.trim().is_empty()
            || self.source.trim().is_empty()
        {
            return Err("Game data is missing version or source metadata.".into());
        }
        if self.servants.is_empty() {
            return Err("Game data contains no servants.".into());
        }

        let mut ids = HashSet::with_capacity(self.servants.len());
        for servant in &self.servants {
            if !servant.deck.is_empty()
                && (servant.deck.len() != 5 || servant.deck.contains(&CardType::Extra))
            {
                return Err(format!("{} has an invalid five-card deck.", servant.name));
            }
            if servant
                .deck_note
                .as_ref()
                .is_some_and(|note| note.trim().is_empty())
                || (!servant.deck.is_empty() && servant.deck_note.is_some())
            {
                return Err(format!(
                    "{} has inconsistent card availability data.",
                    servant.name
                ));
            }
            if (servant.np_status == NpStatus::Damaging) != !servant.noble_phantasms.is_empty() {
                return Err(format!("{} has inconsistent NP metadata.", servant.name));
            }
            let mut np_ids = HashSet::new();
            for np in &servant.noble_phantasms {
                if np.id == 0
                    || !np_ids.insert(np.id)
                    || np.name.trim().is_empty()
                    || np.card_type == CardType::Extra
                    || np.multipliers.iter().any(|v| !v.is_finite() || *v <= 0.0)
                    || np.affection.as_ref().is_some_and(|scale| !scale.is_valid())
                {
                    return Err(format!("{} has invalid NP damage data.", servant.name));
                }
            }
            if servant.id == 0
                || servant.name.trim().is_empty()
                || servant.attack == 0
                || servant.source_url.trim().is_empty()
            {
                return Err("Game data contains a servant with missing required values.".into());
            }
            if servant.level == Some(0) {
                return Err(format!(
                    "{} has an invalid level in game data.",
                    servant.name
                ));
            }
            if !ids.insert(servant.id) {
                return Err(format!(
                    "Game data contains duplicate servant ID {}.",
                    servant.id
                ));
            }
        }

        Ok(())
    }
}
