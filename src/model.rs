use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CardType {
    Buster,
    Arts,
    Quick,
    Extra,
}

impl CardType {
    /// Damage multiplier for the first card in a command chain.
    pub const fn base_multiplier(self) -> f64 {
        match self {
            Self::Buster => 1.5,
            Self::Arts => 1.0,
            Self::Quick => 0.8,
            Self::Extra => 1.0,
        }
    }

    /// A first-position Buster card adds 0.5 outside the card buff multiplier.
    pub const fn first_card_bonus(self) -> f64 {
        match self {
            Self::Buster => 0.5,
            _ => 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Servant {
    //servant stats
    name: String,
    level: u32,
    max_hp: u32,
    attack: u32,
    card: CardType,
    class: ClassType,
    attribute: AttributeType,
    noble_phantasm: NoblePhantasmType,
    skill_1: SkillType,
    skill_2: SkillType,
    skill_3: SkillType,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeType {
    Earth,
    Sky,
    Man,
    Star,
    Beast,
}

impl AttributeType {
    pub const ALL: [Self; 5] = [Self::Earth, Self::Sky, Self::Man, Self::Star, Self::Beast];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Earth => "Earth",
            Self::Sky => "Sky",
            Self::Man => "Man",
            Self::Star => "Star",
            Self::Beast => "Beast",
        }
    }

    /// Returns FGO's attack-side attribute affinity against a target attribute.
    pub const fn affinity_against(self, target: Self) -> f64 {
        use AttributeType::*;

        match (self, target) {
            (Man, Sky) | (Sky, Earth) | (Earth, Man) | (Star, Beast) | (Beast, Star) => 1.1,
            (Man, Earth) | (Earth, Sky) | (Sky, Man) => 0.9,
            _ => 1.0,
        }
    }
}

impl Servant {
    pub fn new(
        name: String,
        level: u32,
        max_hp: u32,
        attack: u32,
        card: CardType,
        class: ClassType,
        attribute: AttributeType,
        noble_phantasm: NoblePhantasmType,
        skill_1: SkillType,
        skill_2: SkillType,
        skill_3: SkillType,
    ) -> Self {
        Servant {
            name,
            level,
            max_hp,
            attack,
            card,
            class,
            attribute,
            noble_phantasm,
            skill_1,
            skill_2,
            skill_3,
        }
    }

    pub fn attack(&self) -> u32 {
        self.attack
    }

    pub fn card(&self) -> CardType {
        self.card
    }

    pub fn class(&self) -> ClassType {
        self.class
    }

    pub fn attribute(&self) -> AttributeType {
        self.attribute
    }
}
#[derive(Debug, Clone)]
pub struct NoblePhantasmType {
    name: String,
    level: u32,
    damage_multiplier: f64,
    card_type: CardType,
}

impl NoblePhantasmType {
    pub fn new(name: String, level: u32, damage_multiplier: f64, card_type: CardType) -> Self {
        NoblePhantasmType {
            name,
            level,
            damage_multiplier,
            card_type,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkillType {
    name: String,
    level: u32,
    effect: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassType {
    Saber,
    Archer,
    Lancer,
    Rider,
    Caster,
    Assassin,
    Berserker,
    Ruler,
    Avenger,
    MoonCancer,
    AlterEgo,
    Foreigner,
    Pretender,
    Beast,
    Shielder,
}

impl ClassType {
    pub const SELECTABLE: [Self; 14] = [
        Self::Saber,
        Self::Archer,
        Self::Lancer,
        Self::Rider,
        Self::Caster,
        Self::Assassin,
        Self::Berserker,
        Self::Shielder,
        Self::Ruler,
        Self::AlterEgo,
        Self::Avenger,
        Self::MoonCancer,
        Self::Foreigner,
        Self::Pretender,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Saber => "Saber",
            Self::Archer => "Archer",
            Self::Lancer => "Lancer",
            Self::Rider => "Rider",
            Self::Caster => "Caster",
            Self::Assassin => "Assassin",
            Self::Berserker => "Berserker",
            Self::Ruler => "Ruler",
            Self::Avenger => "Avenger",
            Self::MoonCancer => "Moon Cancer",
            Self::AlterEgo => "Alter Ego",
            Self::Foreigner => "Foreigner",
            Self::Pretender => "Pretender",
            Self::Beast => "Beast",
            Self::Shielder => "Shielder",
        }
    }

    /// Returns the directional class affinity multiplier from attacker to target.
    /// The generic Beast class is intentionally neutral here because individual
    /// Beast encounters use different rows in the source game data.
    pub const fn affinity_against(self, target: Self) -> f64 {
        const RELATIONS: [[u16; 14]; 14] = [
            [
                1000, 500, 2000, 1000, 1000, 1000, 2000, 1000, 500, 1000, 1000, 1000, 1000, 1000,
            ],
            [
                2000, 1000, 500, 1000, 1000, 1000, 2000, 1000, 500, 1000, 1000, 1000, 1000, 1000,
            ],
            [
                500, 2000, 1000, 1000, 1000, 1000, 2000, 1000, 500, 1000, 1000, 1000, 1000, 1000,
            ],
            [
                1000, 1000, 1000, 1000, 2000, 500, 2000, 1000, 500, 1000, 1000, 1000, 1000, 1000,
            ],
            [
                1000, 1000, 1000, 500, 1000, 2000, 2000, 1000, 500, 1000, 1000, 1000, 1000, 1000,
            ],
            [
                1000, 1000, 1000, 2000, 500, 1000, 2000, 1000, 500, 1000, 1000, 1000, 1000, 1000,
            ],
            [
                1500, 1500, 1500, 1500, 1500, 1500, 1500, 1000, 1500, 1500, 1500, 1500, 500, 1500,
            ],
            [
                1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000,
            ],
            [
                1000, 1000, 1000, 1000, 1000, 1000, 2000, 1000, 1000, 1000, 500, 2000, 1000, 1000,
            ],
            [
                500, 500, 500, 1500, 1500, 1500, 2000, 1000, 1000, 1000, 1000, 1000, 2000, 500,
            ],
            [
                1000, 1000, 1000, 1000, 1000, 1000, 2000, 1000, 2000, 1000, 1000, 500, 1000, 1000,
            ],
            [
                1000, 1000, 1000, 1000, 1000, 1000, 2000, 1000, 500, 1000, 2000, 1000, 1000, 1000,
            ],
            [
                1000, 1000, 1000, 1000, 1000, 1000, 2000, 1000, 1000, 500, 1000, 1000, 2000, 2000,
            ],
            [
                1500, 1500, 1500, 500, 500, 500, 2000, 1000, 1000, 2000, 1000, 1000, 500, 1000,
            ],
        ];

        let Some(attacker_index) = self.affinity_index() else {
            return 1.0;
        };
        let Some(target_index) = target.affinity_index() else {
            return 1.0;
        };

        RELATIONS[attacker_index][target_index] as f64 / 1000.0
    }

    const fn affinity_index(self) -> Option<usize> {
        match self {
            Self::Saber => Some(0),
            Self::Archer => Some(1),
            Self::Lancer => Some(2),
            Self::Rider => Some(3),
            Self::Caster => Some(4),
            Self::Assassin => Some(5),
            Self::Berserker => Some(6),
            Self::Shielder => Some(7),
            Self::Ruler => Some(8),
            Self::AlterEgo => Some(9),
            Self::Avenger => Some(10),
            Self::MoonCancer => Some(11),
            Self::Foreigner => Some(12),
            Self::Pretender => Some(13),
            Self::Beast => None,
        }
    }

    pub const fn class_default_multiplier(self) -> f64 {
        match self {
            Self::Saber => 1.0,
            Self::Archer => 0.95,
            Self::Lancer => 1.05,
            Self::Rider => 1.0,
            Self::Caster => 0.9,
            Self::Assassin => 0.9,
            Self::Berserker => 1.1,
            Self::Ruler => 1.1,
            Self::Avenger => 1.1,
            Self::MoonCancer => 1.0,
            Self::AlterEgo => 1.0,
            Self::Foreigner => 1.0,
            Self::Beast => 1.0,
            Self::Pretender => 1.0,
            Self::Shielder => 1.0,
        }
    }
}

impl SkillType {
    pub fn new(name: String, level: u32, effect: String) -> Self {
        Self {
            name,
            level,
            effect,
        }
    }
}

/// A physical normal card or one of the servant's supported NP variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedCard {
    Normal(usize),
    NoblePhantasm(usize),
}

impl SelectedCard {
    pub fn card_type(self, servant: &crate::loader::ServantRecord) -> Option<CardType> {
        match self {
            Self::Normal(index) => servant.deck.get(index).copied(),
            Self::NoblePhantasm(index) => servant.noble_phantasms.get(index).map(|np| np.card_type),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnSelection {
    pub slots: [SelectedCard; 3],
    pub np_level: u8,
}

impl TurnSelection {
    pub fn default_for(servant: &crate::loader::ServantRecord) -> Option<Self> {
        if servant.deck.len() != 5 {
            return None;
        }
        let slots = if servant.noble_phantasms.is_empty() {
            [
                SelectedCard::Normal(0),
                SelectedCard::Normal(1),
                SelectedCard::Normal(2),
            ]
        } else {
            [
                SelectedCard::NoblePhantasm(0),
                SelectedCard::Normal(0),
                SelectedCard::Normal(1),
            ]
        };
        Some(Self { slots, np_level: 1 })
    }

    pub fn validate(&self, servant: &crate::loader::ServantRecord) -> Result<(), String> {
        if servant.deck.len() != 5 || servant.deck.contains(&CardType::Extra) {
            return Err(
                "The servant's deck is unavailable. Update servant data to load it.".into(),
            );
        }
        if !(1..=5).contains(&self.np_level) {
            return Err("NP level must be between 1 and 5.".into());
        }
        let mut used = [false; 5];
        let mut np_used = false;
        for slot in self.slots {
            match slot {
                SelectedCard::Normal(index) => {
                    if index >= 5 || used[index] {
                        return Err("Select three distinct available cards.".into());
                    }
                    used[index] = true;
                }
                SelectedCard::NoblePhantasm(index) => {
                    if np_used
                        || index >= servant.noble_phantasms.len()
                        || servant.np_status != crate::loader::NpStatus::Damaging
                    {
                        return Err("Select at most one supported damaging NP.".into());
                    }
                    np_used = true;
                }
            }
        }
        Ok(())
    }
}
