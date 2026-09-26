use std::collections::HashSet;

use serde::Deserialize;

use crate::damage::DamageInput;
use crate::model::{AttributeType, CardType, ClassType};

const BUNDLED_GAME_DATA: &str = include_str!("../data/game_data.json");

#[derive(Debug, Clone, Deserialize)]
pub struct GameData {
    pub version: String,
    pub region: String,
    pub retrieved_at: String,
    pub source: String,
    pub servants: Vec<ServantRecord>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServantRecord {
    pub id: u32,
    pub name: String,
    pub level: u32,
    pub attack: u32,
    pub class: ClassType,
    pub attribute: AttributeType,
    pub source_url: String,
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

    pub fn servant(&self, id: u32) -> Option<&ServantRecord> {
        self.servants.iter().find(|servant| servant.id == id)
    }

    fn validate(&self) -> Result<(), String> {
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
            if servant.id == 0
                || servant.name.trim().is_empty()
                || servant.attack == 0
                || servant.source_url.trim().is_empty()
            {
                return Err("Game data contains a servant with missing required values.".into());
            }
            if servant.level == 0 {
                return Err(format!(
                    "{} has an invalid level in game data.",
                    servant.name
                ));
            }
            if servant.class == ClassType::Beast {
                return Err(format!(
                    "{} uses Beast class affinity, which must be specified per encounter.",
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
