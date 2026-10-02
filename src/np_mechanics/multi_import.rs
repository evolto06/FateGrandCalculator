//! Narrow import policy for the multi-component NPs verified for v0.1.3.

use serde_json::Value;

pub fn is_verified_variant(servant_id: u32, np_id: u32) -> bool {
    matches!(
        (servant_id, np_id),
        (201300, 201301 | 201302) | (504400, 504401) | (305400, 305401)
    )
}

pub fn is_verified_shape(functions: &[(usize, &Value)]) -> bool {
    let [(first_position, first), (second_position, second)] = functions else {
        return false;
    };
    *second_position == *first_position + 1
        && first["funcType"] == "damageNp"
        && second["funcType"] == "damageNp"
        && matches!(first["funcTargetType"].as_str(), Some("enemy" | "enemyAll"))
        && first["funcTargetType"] == second["funcTargetType"]
        && functions.iter().all(|(_, function)| {
            [
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
            }) && function
                .get("funcTargetTeam")
                .is_none_or(|value| value.as_str() == Some("playerAndEnemy"))
                && function
                    .get("script")
                    .is_none_or(|value| value.as_object().is_some_and(serde_json::Map::is_empty))
        })
}
