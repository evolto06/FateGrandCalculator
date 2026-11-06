//! The audited NP lists, loaded from `data/audited_nps.json`.
//!
//! Only NPs whose Atlas data has been checked by hand get their special
//! mechanics imported. Auditing another NP means adding it to the data file,
//! not changing code.

use std::collections::HashMap;
use std::sync::LazyLock;

use serde::Deserialize;

use super::enemy_status::EnemyStatus;

const AUDITED_NPS: &str = include_str!("../../data/audited_nps.json");

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuditedNps {
    multi_component: Vec<MultiComponentNp>,
    enemy_status: Vec<EnemyStatusNp>,
    trait_labels: HashMap<i32, String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MultiComponentNp {
    servant_id: u32,
    np_ids: Vec<u32>,
    note: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnemyStatusNp {
    servant_id: u32,
    np_id: u32,
    status: EnemyStatus,
}

/// A malformed file leaves every list empty, so no NP gets an unaudited
/// mechanic; the tests below make sure the bundled file parses.
static AUDITED: LazyLock<AuditedNps> =
    LazyLock::new(|| serde_json::from_str(AUDITED_NPS).unwrap_or_default());

/// True for the audited multi-component NP variants.
pub fn is_multi_component_variant(servant_id: u32, np_id: u32) -> bool {
    AUDITED
        .multi_component
        .iter()
        .any(|np| np.servant_id == servant_id && np.np_ids.contains(&np_id))
}

/// The note shown for a servant's audited multi-component NP.
pub fn multi_component_note(servant_id: u32) -> Option<&'static str> {
    AUDITED
        .multi_component
        .iter()
        .find(|np| np.servant_id == servant_id)
        .map(|np| np.note.as_str())
}

/// The enemy status an audited NP checks at damage time.
pub fn enemy_status(servant_id: u32, np_id: u32) -> Option<EnemyStatus> {
    AUDITED
        .enemy_status
        .iter()
        .find(|np| np.servant_id == servant_id && np.np_id == np_id)
        .map(|np| np.status)
}

/// Display name for an audited Atlas trait (individuality) ID.
pub fn trait_label(target: i32) -> Option<&'static str> {
    AUDITED.trait_labels.get(&target).map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_list_parses() {
        let parsed: AuditedNps = serde_json::from_str(AUDITED_NPS).unwrap();
        assert_eq!(parsed.multi_component.len(), 3);
        assert_eq!(parsed.enemy_status.len(), 7);
        assert_eq!(parsed.trait_labels.len(), 28);
    }

    #[test]
    fn lookups_match_the_audited_entries() {
        assert!(is_multi_component_variant(201300, 201302));
        assert!(!is_multi_component_variant(201300, 504401));
        assert!(multi_component_note(305400).unwrap().starts_with("Bhima"));
        assert_eq!(enemy_status(604900, 604901), Some(EnemyStatus::Bind));
        assert_eq!(enemy_status(604900, 200301), None);
        assert_eq!(trait_label(2000), Some("Divine"));
        assert_eq!(trait_label(-2000), None);
    }
}
