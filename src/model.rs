#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardType {
    Buster,
    Arts,
    Quick,
    Extra,
}

impl CardType {
    /// Base card multiplier used by this early calculator model.
    pub const fn base_multiplier(self) -> f64 {
        match self {
            Self::Buster => 1.5,
            Self::Arts => 1.0,
            Self::Quick => 0.8,
            Self::Extra => 1.0,
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
#[derive(Debug, Copy, Clone)]
pub enum AttributeType {
    Earth,
    Sky,
    Man,
    Star,
    Beast,
}

impl AttributeType {
    pub const fn attribute_multiplier(self) -> f64 {
        match self {
            Self::Earth => 1.0,
            Self::Sky => 1.0,
            Self::Man => 1.0,
            Self::Star => 1.0,
            Self::Beast => 1.0,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

impl ClassType {
    pub const fn class_default_multiplier(self) -> f64 {
        match self {
            Self::Saber => 1.0,
            Self::Archer => 0.9,
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
