use eframe::egui::{self, Color32, RichText};
use fate_grand_calculator::damage::{
    DamageResult, TurnBuffs, TurnDamageResult, percent_to_modifier,
};
use fate_grand_calculator::loader::{GameData, NpStatus, ServantRecord};
use fate_grand_calculator::model::{
    AttackerHp, AttributeType, CardType, ClassType, SelectedCard, TurnSelection,
};
use fate_grand_calculator::np_mechanics::enemy_status::EnemyStatus;
use fate_grand_calculator::np_mechanics::trait_bonus::TraitCondition;

use super::theme::{self, *};

mod attack;
mod cards;
mod hp;
mod inputs;
mod np_settings;
mod result;
mod status_assumption;
mod trait_assumption;
mod widgets;

use self::{cards::*, hp::*, np_settings::*, status_assumption::*, trait_assumption::*};

pub use self::attack::*;
pub use self::inputs::*;
pub use self::result::*;
pub use self::widgets::*;

pub(crate) use self::hp::clear_hp_inputs;
#[cfg(test)]
pub(crate) use self::hp::test_hp_inputs;
pub(crate) use self::np_settings::clear_overcharge_feedback;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use self::{
    hp::has_hp_inputs, status_assumption::reset_enemy_status_for_catalog,
    trait_assumption::reset_trait_bonus_for_catalog,
};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
