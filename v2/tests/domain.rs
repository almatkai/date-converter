use date_converter_v2::{
    converter::{self, Preferences},
    date, storage,
};
use serde_json::{json, Value};
#[test]
fn whole_source_parity() {
    let tests: Vec<Value> =
        serde_json::from_str(include_str!("../evidence/source-vectors.json")).unwrap();
    let exclusions: Vec<Value> =
        serde_json::from_str(include_str!("../qa/expected-exclusions.json")).unwrap();
    assert_eq!(tests.len(), 2239);
    assert_eq!(exclusions.len(), 31);
    let mut exact = 0;
    let mut excluded = 0;
    for (i, t) in tests.iter().enumerate() {
        let p = Preferences::read(&t["preferences"]).unwrap();
        let tz = t["tz"].as_str().unwrap().parse().unwrap();
        let target = t["target"]
            .as_str()
            .unwrap_or(t["tz"].as_str().unwrap())
            .parse()
            .unwrap();
        let actual = match converter::results(
            t["text"].as_str().unwrap(),
            t["now"].as_i64().unwrap(),
            tz,
            target,
            &p,
        ) {
            Ok(r) => json!({"rows":r}),
            Err(e) => json!({"error":e}),
        };
        if let Some(limit) = exclusions.iter().find(|e| e["index"] == i) {
            assert_eq!(&limit["case"], t, "exclusion input/source changed: {i}");
            assert_eq!(
                actual, limit["native"],
                "exclusion native result changed: {i}"
            );
            assert_ne!(actual, t["expected"], "remove obsolete exclusion: {i}");
            excluded += 1;
        } else {
            assert_eq!(actual, t["expected"], "{t}");
            exact += 1;
        }
    }
    assert_eq!((exact, excluded), (2208, 31));
}
#[test]
fn bounds_and_timezone_inventory() {
    let p = Preferences::read(&json!({})).unwrap();
    assert!(converter::results(&"x".repeat(4097), 0, chrono_tz::UTC, chrono_tz::UTC, &p).is_err());
    assert!(date::format(i64::MAX, chrono_tz::UTC, "YYYY", false).is_err());
    let zones: Vec<String> =
        serde_json::from_str(include_str!("../resources/timezones.json")).unwrap();
    for z in zones {
        assert!(z.parse::<chrono_tz::Tz>().is_ok(), "{z}")
    }
}
fn temp() -> tempfile::TempDir {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".qa/test-data");
    std::fs::create_dir_all(&root).unwrap();
    tempfile::tempdir_in(root).unwrap()
}
#[test]
fn private_store_guards_and_unknown_keys() {
    let d = temp();
    let root = d.path();
    assert_eq!(storage::load(root).unwrap(), None);
    storage::save(root, "UTC").unwrap();
    let path = root.join("timezone/storage.json");
    std::fs::write(&path, br#"{"value":"UTC","unknown":{"keep":true}}"#).unwrap();
    storage::save(root, "Europe/Berlin").unwrap();
    let v: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(v["unknown"]["keep"], true);
    for b in [
        b"null".as_slice(),
        br#"{"value":3}"#,
        br#"{"schemaVersion":2}"#,
        b"{corrupt",
    ] {
        std::fs::write(&path, b).unwrap();
        assert!(storage::load(root).is_err());
        assert!(storage::save(root, "UTC").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b)
    }
    std::fs::write(&path, vec![b' '; 65537]).unwrap();
    assert!(storage::load(root).is_err())
}
#[cfg(unix)]
#[test]
fn private_store_rejects_symlinks() {
    use std::os::unix::fs::symlink;
    let d = temp();
    let external = temp();
    symlink(external.path(), d.path().join("timezone")).unwrap();
    assert!(storage::save(d.path(), "UTC").is_err());
    assert!(!external.path().join("storage.json").exists());
}
#[test]
fn exact_closed_acks() {
    for method in [
        "host.clipboard.write",
        "host.clipboard.pasteText",
        "host.settings.open",
    ] {
        assert!(date_converter_v2::adapter::ack(method, &json!({"ok":true})).is_ok());
        for v in [
            Value::Null,
            json!(true),
            json!({"ok":false}),
            json!({"ok":true,"error":"bad"}),
            json!({"ok":true,"message":false}),
            json!({"ok":true,"extra":true}),
        ] {
            assert!(date_converter_v2::adapter::ack(method, &v).is_err())
        }
    }
    assert!(date_converter_v2::adapter::ack("host.window.visibility", &Value::Null).is_ok());
    assert!(
        date_converter_v2::adapter::ack("host.window.visibility", &json!({"ok":true})).is_err()
    );
}

#[test]
fn pinned_recurrence_and_inventory() {
    use chrono::{Offset, TimeZone};
    assert_eq!(tzdb_data::VERSION, "2026a");
    let zones: Vec<String> =
        serde_json::from_str(include_str!("../resources/timezones.json")).unwrap();
    for z in zones {
        let zone: chrono_tz::Tz = z.parse().unwrap();
        for year in [100, 1899, 2099, 2100, 2400, 9999] {
            let ms = chrono::Utc
                .with_ymd_and_hms(year, 7, 20, 12, 0, 0)
                .unwrap()
                .timestamp_millis();
            assert!(date::local(ms, zone).is_ok(), "{year}: {z}");
        }
    }
    // Last explicit chrono-tz transitions are insufficient for future summer DST.
    for (zone, winter, summer) in [
        (chrono_tz::America::New_York, -18000, -14400),
        (chrono_tz::Europe::Berlin, 3600, 7200),
        (chrono_tz::Australia::Lord_Howe, 39600, 37800),
    ] {
        for (month, expected) in [(1, winter), (7, summer)] {
            let ms = chrono::Utc
                .with_ymd_and_hms(2500, month, 20, 12, 0, 0)
                .unwrap()
                .timestamp_millis();
            assert_eq!(
                date::local(ms, zone)
                    .unwrap()
                    .offset()
                    .fix()
                    .local_minus_utc(),
                expected
            );
        }
    }
}
