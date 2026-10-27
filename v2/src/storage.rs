//! Only the source dropdown's storeValue is persisted. No parsed-date cache/history.
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};
static LOCK: Mutex<()> = Mutex::new(());
const LIMIT: u64 = 65536;
fn path(root: &Path) -> Result<PathBuf, String> {
    let file = root.join("timezone").join("storage.json");
    for (i, c) in file
        .ancestors()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .enumerate()
    {
        if i == 0 {
            continue;
        }
        match std::fs::symlink_metadata(c) {
            Ok(m) => {
                if m.file_type().is_symlink() {
                    return Err("Timezone storage contains a symlink".into());
                }
                if c == file {
                    if !m.is_file() || m.len() > LIMIT {
                        return Err("Timezone storage is not a bounded regular file".into());
                    }
                } else if !m.is_dir() {
                    return Err("Timezone storage parent is not a directory".into());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(file)
}
fn read(file: &Path) -> Result<Value, String> {
    let mut bytes = vec![];
    match std::fs::File::open(file) {
        Ok(f) => {
            f.take(LIMIT + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(json!({})),
        Err(e) => return Err(e.to_string()),
    }
    if bytes.len() as u64 > LIMIT {
        return Err("Timezone storage exceeds 64 KiB".into());
    }
    let v: Value = serde_json::from_slice(&bytes)
        .map_err(|_| "Corrupt timezone storage; input remains editable")?;
    if !v.is_object()
        || v.get("schemaVersion").is_some_and(|v| v != 1)
        || v.get("value").is_some_and(|v| !v.is_string())
    {
        return Err("Invalid timezone storage schema".into());
    }
    Ok(v)
}
pub fn load(root: &Path) -> Result<Option<String>, String> {
    let _lock = LOCK.lock().map_err(|_| "Storage lock poisoned")?;
    let f = path(root)?;
    let v = read(&f)?;
    Ok(v["value"].as_str().map(str::to_owned))
}
pub fn save(root: &Path, zone: &str) -> Result<(), String> {
    let _lock = LOCK.lock().map_err(|_| "Storage lock poisoned")?;
    let f = path(root)?;
    let mut v = read(&f)?;
    v["schemaVersion"] = json!(1);
    v["value"] = json!(zone);
    let b = serde_json::to_vec(&v).map_err(|e| e.to_string())?;
    if b.len() as u64 > LIMIT {
        return Err("Timezone storage exceeds 64 KiB".into());
    }
    let parent = f.parent().unwrap();
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    path(root)?;
    let mut t = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    t.write_all(&b).map_err(|e| e.to_string())?;
    t.as_file().sync_all().map_err(|e| e.to_string())?;
    t.persist(&f).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    std::fs::File::open(parent)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())?;
    Ok(())
}
