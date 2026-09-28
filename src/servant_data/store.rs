use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::loader::GameData;

const SNAPSHOT_SCHEMA_VERSION: u32 = 2;
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
        if envelope.schema_version != 1 && envelope.schema_version != SNAPSHOT_SCHEMA_VERSION {
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
            for field in ["deck", "np_status", "noble_phantasms"] {
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
        assert_eq!(
            migrated.servants[0].np_status,
            crate::loader::NpStatus::Unavailable
        );
        fs::remove_file(&store.path).unwrap();
    }
    #[test]
    fn successful_save_uses_v2_and_invalid_save_preserves_existing_bytes() {
        let store = test_store();
        let mut data = GameData::bundled().unwrap();
        store.save(&data).unwrap();
        let original = fs::read(&store.path).unwrap();
        let envelope: serde_json::Value = serde_json::from_slice(&original).unwrap();
        assert_eq!(envelope["schema_version"], 2);
        assert_eq!(store.load().unwrap().unwrap().servants[0].deck.len(), 5);
        data.servants[0].deck.pop();
        assert!(store.save(&data).is_err());
        assert_eq!(fs::read(&store.path).unwrap(), original);
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
}
