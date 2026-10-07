use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::loader::GameData;

const SNAPSHOT_SCHEMA_VERSION: u32 = 6;
const MAX_SNAPSHOT_BYTES: u64 = 64 * 1024 * 1024;

pub struct SnapshotStore {
    path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct SnapshotEnvelope {
    schema_version: u32,
    game_data: GameData,
}

impl SnapshotStore {
    pub fn in_app_data_directory() -> Result<Self, String> {
        let project_dirs = ProjectDirs::from("org", "FateGrandCalculator", "FateGrandCalculator")
            .ok_or_else(|| {
            "Could not find the operating system's application data directory.".to_owned()
        })?;
        Ok(Self {
            path: project_dirs.data_local_dir().join("servant-data-v1.json"),
        })
    }

    pub fn load(&self) -> Result<Option<GameData>, String> {
        let file = match File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(format!(
                    "Could not open the local servant snapshot: {error}"
                ));
            }
        };

        let size = file
            .metadata()
            .map_err(|error| format!("Could not inspect the local servant snapshot: {error}"))?
            .len();
        if size > MAX_SNAPSHOT_BYTES {
            return Err("The local servant snapshot is larger than the allowed size.".into());
        }

        let mut json = String::with_capacity(size as usize);
        file.take(MAX_SNAPSHOT_BYTES + 1)
            .read_to_string(&mut json)
            .map_err(|error| format!("Could not read the local servant snapshot: {error}"))?;
        if json.len() as u64 > MAX_SNAPSHOT_BYTES {
            return Err(
                "The local servant snapshot grew beyond the allowed size while being read.".into(),
            );
        }
        let envelope: SnapshotEnvelope = serde_json::from_str(&json)
            .map_err(|error| format!("The local servant snapshot is malformed: {error}"))?;
        if !matches!(envelope.schema_version, 1..=SNAPSHOT_SCHEMA_VERSION) {
            return Err(format!(
                "The local servant snapshot uses unsupported format version {}.",
                envelope.schema_version
            ));
        }
        envelope.game_data.validate()?;
        Ok(Some(envelope.game_data))
    }

    pub fn save(&self, game_data: &GameData) -> Result<(), String> {
        game_data.validate()?;
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "The local servant snapshot path is invalid.".to_owned())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create the local data directory: {error}"))?;

        let envelope = SnapshotEnvelope {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            game_data: game_data.clone(),
        };
        let bytes = serde_json::to_vec_pretty(&envelope)
            .map_err(|error| format!("Could not encode the servant snapshot: {error}"))?;
        let temp_path = temporary_path(&self.path);

        let write_result = write_staged_snapshot(&temp_path, &bytes)
            .and_then(|()| replace_snapshot(&temp_path, &self.path));
        if write_result.is_err() {
            let _ = fs::remove_file(&temp_path);
        }
        write_result.map_err(|error| format!("Could not safely save servant data: {error}"))
    }
}

fn temporary_path(snapshot_path: &Path) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let file_name = format!("servant-data-{}-{nonce}.tmp", std::process::id());
    snapshot_path.with_file_name(file_name)
}

fn write_staged_snapshot(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(not(windows))]
fn replace_snapshot(staged: &Path, destination: &Path) -> std::io::Result<()> {
    fs::rename(staged, destination)
}

#[cfg(windows)]
fn replace_snapshot(staged: &Path, destination: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };

    let staged: Vec<u16> = staged.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let success = unsafe {
        MoveFileExW(
            staged.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if success == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_store() -> SnapshotStore {
        SnapshotStore {
            path: std::env::temp_dir().join(format!(
                "fgc-snapshot-{}-{}.json",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            )),
        }
    }
    #[test]
    fn reads_v1_without_fabricating_new_gameplay_data() {
        let store = test_store();
        let mut data = serde_json::to_value(GameData::bundled().unwrap()).unwrap();
        for servant in data["servants"].as_array_mut().unwrap() {
            for field in ["deck", "np_status", "noble_phantasms", "max_hp"] {
                servant.as_object_mut().unwrap().remove(field);
            }
        }
        fs::write(
            &store.path,
            serde_json::to_vec(&serde_json::json!({"schema_version":1,"game_data":data})).unwrap(),
        )
        .unwrap();
        let migrated = store.load().unwrap().unwrap();
        assert!(migrated.servants[0].deck.is_empty());
        assert_eq!(migrated.servants[0].max_hp, None);
        assert_eq!(
            migrated.servants[0].np_status,
            crate::loader::NpStatus::Unavailable
        );
        fs::remove_file(&store.path).unwrap();
    }
    #[test]
    fn successful_save_uses_v6_and_invalid_save_preserves_existing_bytes() {
        let store = test_store();
        let mut data = GameData::bundled().unwrap();
        store.save(&data).unwrap();
        let original = fs::read(&store.path).unwrap();
        let envelope: serde_json::Value = serde_json::from_slice(&original).unwrap();
        assert_eq!(envelope["schema_version"], 6);
        assert_eq!(store.load().unwrap().unwrap().servants[0].deck.len(), 5);
        data.servants[0].deck.pop();
        assert!(store.save(&data).is_err());
        assert_eq!(fs::read(&store.path).unwrap(), original);
        fs::remove_file(&store.path).unwrap();
    }
    #[test]
    fn v2_np_snapshot_migrates_without_inventing_overcharge_and_saves_v6() {
        let store = test_store();
        let mut data = serde_json::to_value(GameData::bundled().unwrap()).unwrap();
        for servant in data["servants"].as_array_mut().unwrap() {
            for np in servant["noble_phantasms"].as_array_mut().unwrap() {
                let multipliers = np["components"][0]["overcharge"][0]["multipliers"].clone();
                np.as_object_mut().unwrap().remove("components");
                np["multipliers"] = multipliers;
            }
        }
        fs::write(
            &store.path,
            serde_json::to_vec(&serde_json::json!({"schema_version":2,"game_data":data})).unwrap(),
        )
        .unwrap();
        let original = fs::read(&store.path).unwrap();
        let migrated = store.load().unwrap().unwrap();
        assert_eq!(fs::read(&store.path).unwrap(), original);
        for np in migrated
            .servants
            .iter()
            .flat_map(|servant| &servant.noble_phantasms)
        {
            assert_eq!(np.available_overcharges(), [1]);
            assert_eq!(np.base_multiplier(5, 2), None);
        }
        store.save(&migrated).unwrap();
        let saved: serde_json::Value =
            serde_json::from_slice(&fs::read(&store.path).unwrap()).unwrap();
        assert_eq!(saved["schema_version"], 6);
        assert!(
            saved["game_data"]["servants"][0]["noble_phantasms"][0]
                .get("multipliers")
                .is_none()
        );
        assert_eq!(
            store.load().unwrap().unwrap().servants[0].noble_phantasms[0].available_overcharges(),
            [1]
        );
        fs::remove_file(&store.path).unwrap();
    }
    #[test]
    fn rejects_unknown_snapshot_versions_without_overwriting() {
        let store = test_store();
        let original = serde_json::to_vec(
            &serde_json::json!({"schema_version":99,"game_data":GameData::bundled().unwrap()}),
        )
        .unwrap();
        fs::write(&store.path, &original).unwrap();
        assert!(store.load().unwrap_err().contains("unsupported format"));
        assert_eq!(fs::read(&store.path).unwrap(), original);
        fs::remove_file(&store.path).unwrap();
    }

    #[test]
    fn legacy_v3_defaults_to_ordinary_damage_without_rewriting_the_cache() {
        let store = test_store();
        let mut data = serde_json::to_value(GameData::bundled().unwrap()).unwrap();
        for servant in data["servants"].as_array_mut().unwrap() {
            servant.as_object_mut().unwrap().remove("max_hp");
            for np in servant["noble_phantasms"].as_array_mut().unwrap() {
                for component in np["components"].as_array_mut().unwrap() {
                    for row in component["overcharge"]
                        .as_array_mut()
                        .unwrap()
                        .iter_mut()
                        .filter(|row| !row.is_null())
                    {
                        row.as_object_mut().unwrap().remove("low_hp");
                    }
                }
            }
        }
        let bytes =
            serde_json::to_vec(&serde_json::json!({"schema_version":3,"game_data":data})).unwrap();
        fs::write(&store.path, &bytes).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert!(
            loaded
                .servants
                .iter()
                .all(|servant| servant.max_hp.is_none())
        );
        assert!(
            loaded
                .servants
                .iter()
                .flat_map(|servant| &servant.noble_phantasms)
                .all(|np| !np.requires_attacker_hp())
        );
        assert_eq!(fs::read(&store.path).unwrap(), bytes);
        fs::remove_file(&store.path).unwrap();
    }

    #[test]
    fn v4_round_trips_source_units_and_rejects_malformed_scaling() {
        use crate::np_mechanics::{components::NpDamageComponent, low_hp::LowHpScaling};
        let store = test_store();
        let mut data = GameData::bundled().unwrap();
        data.servants.truncate(1);
        data.servants[0].max_hp = Some(12345);
        let np = &mut data.servants[0].noble_phantasms[0];
        np.components = vec![NpDamageComponent::legacy([6.0; 5])];
        np.components[0].overcharge[0].as_mut().unwrap().low_hp = Some(LowHpScaling {
            source_base_rates: [6000; 5],
            coefficients: [7000; 5],
        });
        store.save(&data).unwrap();
        let bytes = fs::read(&store.path).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded.servants[0].max_hp, Some(12345));
        assert_eq!(
            loaded.servants[0].noble_phantasms[0].components,
            data.servants[0].noble_phantasms[0].components
        );
        for malformed in [
            serde_json::json!({"coefficients": [7000,7000,7000,7000,7000]}),
            serde_json::json!({"source_base_rates": [6001,6001,6001,6001,6001], "coefficients": [7000,7000,7000,7000,7000]}),
            serde_json::json!({"source_base_rates": [6000,6000,6000,6000,6000], "coefficients": [-1,7000,7000,7000,7000]}),
            serde_json::json!({"source_base_rates": [6000,6000,6000,6000,6000], "coefficients": [7000,7000,7000,7000,7000], "unknown_modifier":1}),
        ] {
            let mut invalid: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            invalid["game_data"]["servants"][0]["noble_phantasms"][0]["components"][0]["overcharge"]
                [0]["low_hp"] = malformed;
            fs::write(&store.path, serde_json::to_vec(&invalid).unwrap()).unwrap();
            assert!(store.load().is_err());
        }
        fs::remove_file(&store.path).unwrap();
    }

    #[test]
    fn legacy_v4_defaults_to_absent_status_without_rewriting() {
        let store = test_store();
        let mut data = serde_json::to_value(GameData::bundled().unwrap()).unwrap();
        for servant in data["servants"].as_array_mut().unwrap() {
            for np in servant["noble_phantasms"].as_array_mut().unwrap() {
                for component in np["components"].as_array_mut().unwrap() {
                    for row in component["overcharge"]
                        .as_array_mut()
                        .unwrap()
                        .iter_mut()
                        .filter(|row| !row.is_null())
                    {
                        row.as_object_mut().unwrap().remove("enemy_status");
                    }
                }
            }
        }
        let bytes =
            serde_json::to_vec(&serde_json::json!({"schema_version":4,"game_data":data})).unwrap();
        fs::write(&store.path, &bytes).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert!(
            loaded
                .servants
                .iter()
                .flat_map(|servant| &servant.noble_phantasms)
                .all(|np| np.enemy_status_condition().is_none())
        );
        assert_eq!(fs::read(&store.path).unwrap(), bytes);
        fs::remove_file(&store.path).unwrap();
    }

    #[test]
    fn v5_status_round_trip_validates_source_units_flags_and_preserves_failed_saves() {
        use crate::np_mechanics::enemy_status::{EnemyStatus, EnemyStatusScaling};
        let store = test_store();
        let mut data = GameData::bundled().unwrap();
        data.servants.truncate(1);
        data.servants[0].noble_phantasms.truncate(1);
        data.servants[0].noble_phantasms[0].components =
            vec![crate::np_mechanics::components::NpDamageComponent::legacy(
                [6.0; 5],
            )];
        let status = EnemyStatusScaling {
            condition: EnemyStatus::Poison,
            source_corrections: [2000, 2125, 2250, 2375, 2500],
            include_ignore_individuality: false,
        };
        data.servants[0].noble_phantasms[0].components[0].overcharge[0]
            .as_mut()
            .unwrap()
            .enemy_status = Some(status.clone());
        store.save(&data).unwrap();
        let bytes = fs::read(&store.path).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(
            loaded.servants[0].noble_phantasms[0].components,
            data.servants[0].noble_phantasms[0].components
        );
        assert_eq!(
            loaded.servants[0].noble_phantasms[0].enemy_status_condition(),
            Some(EnemyStatus::Poison)
        );
        let envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(envelope["schema_version"], 6);
        assert_eq!(
            envelope["game_data"]["servants"][0]["noble_phantasms"][0]["components"][0]["overcharge"]
                [0]["enemy_status"]["source_corrections"],
            serde_json::json!([2000, 2125, 2250, 2375, 2500])
        );

        // Validate before staging or replacing: a malformed refresh cannot
        // overwrite the last valid snapshot.
        data.servants[0].noble_phantasms[0].components[0].overcharge[0]
            .as_mut()
            .unwrap()
            .enemy_status
            .as_mut()
            .unwrap()
            .include_ignore_individuality = true;
        assert!(store.save(&data).is_err());
        assert_eq!(fs::read(&store.path).unwrap(), bytes);

        for malformed in [
            serde_json::json!({"condition":"poison","source_corrections":[2000,2000,2000,2000,2000]}),
            serde_json::json!({"condition":"poison","source_corrections":[0,2000,2000,2000,2000],"include_ignore_individuality":false}),
            serde_json::json!({"condition":"poison","source_corrections":[-1,2000,2000,2000,2000],"include_ignore_individuality":false}),
            serde_json::json!({"condition":"poison","source_corrections":[2000,2000,2000,2000,2000],"include_ignore_individuality":true}),
            serde_json::json!({"condition":"poison","source_corrections":[2000,2000,2000,2000,2000],"include_ignore_individuality":false,"unverified_flag":1}),
            serde_json::json!({"condition":"unsupported","source_corrections":[2000,2000,2000,2000,2000],"include_ignore_individuality":false}),
        ] {
            let mut invalid = envelope.clone();
            invalid["game_data"]["servants"][0]["noble_phantasms"][0]["components"][0]["overcharge"]
                [0]["enemy_status"] = malformed;
            let corrupt_bytes = serde_json::to_vec(&invalid).unwrap();
            fs::write(&store.path, &corrupt_bytes).unwrap();
            assert!(store.load().is_err());
            assert_eq!(fs::read(&store.path).unwrap(), corrupt_bytes);
        }
        fs::remove_file(&store.path).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn replacement_failure_preserves_the_last_valid_snapshot_and_removes_staging() {
        use std::os::windows::fs::OpenOptionsExt;
        let mut store = test_store();
        let directory = store.path.with_extension("dir");
        store.path = directory.join("snapshot.json");
        let data = GameData::bundled().unwrap();
        store.save(&data).unwrap();
        let bytes = fs::read(&store.path).unwrap();
        let lock = OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&store.path)
            .unwrap();
        assert!(store.save(&data).is_err());
        drop(lock);
        assert_eq!(fs::read(&store.path).unwrap(), bytes);
        assert!(store.load().unwrap().is_some());
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        fs::remove_file(&store.path).unwrap();
        fs::remove_dir(&directory).unwrap();
    }

    #[test]
    fn legacy_versions_default_to_absent_trait_bonus_and_v6_validates_metadata() {
        use crate::np_mechanics::trait_bonus::{TraitBonusScaling, TraitCondition};
        let store = test_store();
        let mut data = GameData::bundled().unwrap();
        data.servants.truncate(1);
        data.servants[0].noble_phantasms.truncate(1);
        let mut legacy = serde_json::to_value(&data).unwrap();
        for np in legacy["servants"][0]["noble_phantasms"]
            .as_array_mut()
            .unwrap()
        {
            for component in np["components"].as_array_mut().unwrap() {
                for row in component["overcharge"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .filter(|row| !row.is_null())
                {
                    row.as_object_mut().unwrap().remove("trait_bonus");
                }
            }
        }
        for version in 1..=5 {
            let bytes = serde_json::to_vec(
                &serde_json::json!({"schema_version":version,"game_data":legacy}),
            )
            .unwrap();
            fs::write(&store.path, &bytes).unwrap();
            let loaded = store.load().unwrap().unwrap();
            assert!(
                loaded.servants[0].noble_phantasms[0]
                    .trait_bonus_condition()
                    .is_none()
            );
            assert_eq!(
                loaded.servants[0].noble_phantasms[0].base_multiplier(1, 1),
                data.servants[0].noble_phantasms[0].base_multiplier(1, 1)
            );
            assert_eq!(fs::read(&store.path).unwrap(), bytes);
        }
        data.servants[0].noble_phantasms[0].components =
            vec![crate::np_mechanics::components::NpDamageComponent::legacy(
                [6.0; 5],
            )];
        data.servants[0].noble_phantasms[0].components[0].overcharge[0]
            .as_mut()
            .unwrap()
            .trait_bonus = Some(TraitBonusScaling {
            condition: TraitCondition::from_source_target(2002).unwrap(),
            source_corrections: [1500, 1600, 1700, 1800, 1900],
        });
        store.save(&data).unwrap();
        let bytes = fs::read(&store.path).unwrap();
        let envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(envelope["schema_version"], 6);
        assert_eq!(
            store.load().unwrap().unwrap().servants[0].noble_phantasms[0].components,
            data.servants[0].noble_phantasms[0].components
        );
        data.servants[0].noble_phantasms[0].defense_pierce = true;
        assert!(store.save(&data).is_err());
        assert_eq!(fs::read(&store.path).unwrap(), bytes);
        let mut invalid = envelope.clone();
        invalid["game_data"]["servants"][0]["noble_phantasms"][0]["defense_pierce"] =
            serde_json::json!(true);
        fs::write(&store.path, serde_json::to_vec(&invalid).unwrap()).unwrap();
        assert!(store.load().is_err());
        fs::write(&store.path, &bytes).unwrap();
        data.servants[0].noble_phantasms[0].defense_pierce = false;
        data.servants[0].noble_phantasms[0].components[0].overcharge[0]
            .as_mut()
            .unwrap()
            .trait_bonus
            .as_mut()
            .unwrap()
            .source_corrections[0] = 0;
        assert!(store.save(&data).is_err());
        assert_eq!(fs::read(&store.path).unwrap(), bytes);
        for malformed in [
            serde_json::json!({"condition":{"source_target":-2002},"source_corrections":[1500,1500,1500,1500,1500]}),
            serde_json::json!({"condition":{"source_target":999999},"source_corrections":[1500,1500,1500,1500,1500]}),
            serde_json::json!({"condition":{"source_target":2002},"source_corrections":[0,1500,1500,1500,1500]}),
            serde_json::json!({"condition":{"source_target":2002},"source_corrections":[-1,1500,1500,1500,1500]}),
            serde_json::json!({"condition":{"source_target":2002},"source_corrections":[1500,1500,1500,1500,1500],"unknown":1}),
        ] {
            let mut invalid = envelope.clone();
            invalid["game_data"]["servants"][0]["noble_phantasms"][0]["components"][0]["overcharge"]
                [0]["trait_bonus"] = malformed;
            let corrupt = serde_json::to_vec(&invalid).unwrap();
            fs::write(&store.path, &corrupt).unwrap();
            assert!(store.load().is_err());
            assert_eq!(fs::read(&store.path).unwrap(), corrupt);
        }
        fs::remove_file(&store.path).unwrap();
    }
}
