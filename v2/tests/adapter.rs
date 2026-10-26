use date_converter_v2::adapter::Adapter;
use serde_json::{json, Value};
use tezbar_extension_sdk::{Extension, ExtensionPaths, TestContext};
fn context() -> TestContext {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".qa/adapter-tests")
        .join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&p).unwrap();
    TestContext::new(
        ExtensionPaths {
            package_dir: p.clone(),
            data_dir: p.join("data"),
            temp_dir: p.join("tmp"),
        },
        &["clipboard.write", "clipboard.pasteText", "settings.open"],
    )
}
#[tokio::test]
async fn fixed_clock_default_rows_and_retry() {
    let h = context();
    let a = Adapter::fixed(1730426187456, chrono_tz::UTC);
    let mut state = json!({});
    let p = json!({"sessionId":"fixed","preferences":{},"argumentValues":{}});
    let r = a
        .handle(&h.context, "index", "extension:run-command", &p, &mut state)
        .await
        .unwrap();
    assert_eq!(r["root"]["children"].as_array().unwrap().len(), 4);
    assert_eq!(
        r["root"]["children"][0]["props"]["subtitle"],
        "Now - just now"
    );
    assert!(h.calls().is_empty());
    let id = &r["root"]["children"][0]["props"]["actionIds"][0];
    let f = json!({"__viewToken":r["root"]["props"]["__draftGuard"],"__searchText":"","__draftDirty":false,"__searchDirty":false,"__accessoryValue":"UTC"});
    h.reply(Ok(json!({"ok":false})));
    let p = json!({"sessionId":"fixed","actionId":id,"formValues":f});
    let retry = h
        .run(a.handle(
            &h.context,
            "index",
            "extension:invoke-action",
            &p,
            &mut state,
        ))
        .await
        .unwrap();
    assert!(retry["actions"][0]["title"]
        .as_str()
        .unwrap()
        .contains("May Repeat"));
    assert_eq!(h.calls()[0]["params"]["text"], "1730426187456");
    assert!(a
        .handle(
            &h.context,
            "index",
            "extension:invoke-action",
            &p,
            &mut state
        )
        .await
        .is_err());
    let _unused: Value = state;
}
