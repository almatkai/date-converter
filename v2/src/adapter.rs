//! Authoritative session actions outlive SDK staged state; consume before host awaits.
use crate::{
    converter::{self, ParsedDate, Preferences, ResultDate},
    date, storage,
};
use chrono::Utc;
use chrono_tz::Tz;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tezbar_extension_sdk::{
    no_view, unchanged, view, BoxFuture, CoreResult, Extension, ExtensionContext,
};
#[derive(Clone)]
struct Prepared {
    method: &'static str,
    text: String,
    remaining_paste: bool,
}
#[derive(Clone)]
enum Action {
    Copy(String),
    Timezone,
    Retry,
    Settings,
    Parse,
}
struct Authority {
    draft: String,
    source: Tz,
    target: Tz,
    target_error: Option<String>,
    now: i64,
    dates: Vec<ParsedDate>,
    rows: Vec<ResultDate>,
    prefs: Preferences,
    message: Option<String>,
    token: String,
    actions: HashMap<String, Action>,
    prepared: Option<Prepared>,
}
type Shared = Arc<Mutex<Authority>>;
#[derive(Default)]
pub struct Adapter {
    sessions: Mutex<HashMap<String, (Shared, ExtensionContext)>>,
    fixed: Option<(i64, Tz)>,
}
impl Adapter {
    pub fn fixed(now: i64, tz: Tz) -> Self {
        Self {
            fixed: Some((now, tz)),
            ..Self::default()
        }
    }
    fn clock(&self) -> Result<(i64, Tz), String> {
        self.fixed
            .map(Ok)
            .unwrap_or_else(|| Ok((Utc::now().timestamp_millis(), date::system_zone()?)))
    }
    fn session(&self, id: &str) -> Result<Shared, String> {
        self.sessions
            .lock()
            .map_err(|_| "Session lock poisoned")?
            .get(id)
            .map(|(a, _)| a.clone())
            .ok_or("Session no longer active".into())
    }
    async fn prune(&self) -> Result<(), String> {
        let owners = self
            .sessions
            .lock()
            .map_err(|_| "Session lock poisoned")?
            .iter()
            .map(|(id, (a, c))| (id.clone(), a.clone(), c.clone()))
            .collect::<Vec<_>>();
        for (id, a, c) in owners {
            let cancelled = tokio::select! {biased;_=c.cancelled()=>true,_=async{}=>false};
            if cancelled {
                let mut map = self.sessions.lock().map_err(|_| "Session lock poisoned")?;
                if map.get(&id).is_some_and(|(v, _)| Arc::ptr_eq(&a, v)) {
                    map.remove(&id);
                }
            }
        }
        Ok(())
    }
    fn reformat(&self, a: &mut Authority) {
        if let Some(e) = &a.target_error {
            a.rows.clear();
            a.message = Some(e.clone());
            return;
        }
        // Collect atomically: never expose a prefix of successfully formatted rows.
        match converter::render_dates(&a.dates, a.now, a.source, a.target, &a.prefs) {
            Ok(rows) => {
                a.rows = rows;
                a.message = None;
            }
            Err(e) => {
                a.rows.clear();
                a.message = Some(e);
            }
        }
    }
    fn parse(&self, a: &mut Authority) {
        a.rows.clear();
        a.dates.clear();
        match self.clock().and_then(|(now, source)| {
            a.now = now;
            a.source = source;
            converter::parse_dates(&a.draft, now, source)
        }) {
            Ok(dates) => {
                a.dates = dates;
                self.reformat(a);
            }
            Err(e) => a.message = Some(e),
        }
    }
    fn render(&self, a: &mut Authority) -> CoreResult {
        a.actions.clear();
        a.token = uuid::Uuid::new_v4().to_string();
        let token = a.token.clone();
        let mut actions = vec![];
        let mut rows = vec![];
        let mut add = |title: String, act: Action| {
            let id = format!("{token}:{}", actions.len());
            actions.push(json!({"id":id,"title":title,"kind":"callback"}));
            a.actions.insert(id.clone(), act);
            id
        };
        if a.prepared.is_some() {
            let id = add(
                "Retry Exact Prepared Text (May Repeat Previous Effect)".into(),
                Action::Retry,
            );
            rows.push(json!({"type":"List.Item","props":{"id":"uncertain","title":"Delivery outcome is unknown; explicit retry may repeat it","actionIds":[id]}}));
        } else {
            for r in &a.rows {
                let ids = r
                    .values
                    .iter()
                    .map(|(id, text)| {
                        let title = converter::FORMATS.iter().find(|(k, _)| k == id).unwrap().1;
                        add(format!("Copy as {title}"), Action::Copy(text.clone()))
                    })
                    .collect::<Vec<_>>();
                rows.push(json!({"type":"List.Item","props":{"id":converter::iso(r.ms)?,"title":r.title,"subtitle":format!("{} - {}",r.label,converter::time_ago(r.ms,self.clock().map(|n| n.0).unwrap_or(a.now))),"actionIds":ids}}));
            }
        }
        if rows.is_empty() {
            let (title, description) = match &a.message {
                Some(message) => ("Unable to Convert Date", message.as_str()),
                None => (
                    "Invalid Date",
                    "Failed to parse your date in a human or machine format.",
                ),
            };
            rows.push(
                json!({"type":"List.EmptyView","props":{"title":title,"description":description}}),
            );
        }
        if a.message.is_some() && a.prepared.is_none() {
            add("Retry Date Conversion".into(), Action::Parse);
        }
        // Preferences are immutable in SDK sessions; this recovery action explains reopen.
        if a.message.is_some() {
            add("Open Extension Preferences".into(), Action::Settings);
        }
        let tz_id = add("Select Target Timezone".into(), Action::Timezone);
        let mut zones = vec![
            json!({"type":"List.Dropdown.Item","props":{"title":format!("{} - local",a.source),"value":a.source.name()}}),
        ];
        let names: Vec<String> = serde_json::from_str(include_str!("../resources/timezones.json"))
            .map_err(|e| e.to_string())?;
        zones.extend(
            names
                .into_iter()
                .filter(|s| s != a.source.name())
                .map(|s| json!({"type":"List.Dropdown.Item","props":{"title":s,"value":s}})),
        );
        let mut out = view(
            "Convert Date",
            json!({"type":"List","props":{"searchText":a.draft,"searchBarPlaceholder":"Enter a human or machine readable date","__hasServerSearch":true,"__draftGuard":token,"__timezone":a.target.name(),"searchBarAccessory":{"type":"List.Dropdown","props":{"tooltip":"Select Target Timezone","value":a.target.name(),"actionId":tz_id},"children":zones}},"children":rows}),
            actions,
        );
        out["refreshMs"] = Value::Null;
        if let Some(m) = &a.message {
            out["message"] = json!(m)
        }
        Ok(out)
    }
    async fn effect(&self, ctx: &ExtensionContext, shared: &Shared, e: Prepared) -> CoreResult {
        {
            let mut a = shared.lock().map_err(|_| "Session lock poisoned")?;
            a.actions.clear();
            a.prepared = Some(e.clone());
        }
        let mut effect = e;
        let mut result = ctx
            .host(effect.method, json!({"text":effect.text}))
            .await
            .and_then(|v| ack(effect.method, &v));
        if result.is_ok() && effect.remaining_paste {
            effect.method = "host.clipboard.pasteText";
            effect.remaining_paste = false;
            shared.lock().map_err(|_| "Session lock poisoned")?.prepared = Some(effect.clone());
            result = ctx
                .host(effect.method, json!({"text":effect.text}))
                .await
                .and_then(|v| ack(effect.method, &v));
        }
        let mut a = shared.lock().map_err(|_| "Session lock poisoned")?;
        if result.is_ok() {
            a.prepared = None;
            a.message = None; /* Clipboard.copy callback keeps view; paste owns hide/focus/paste. */
            self.render(&mut a)
        } else {
            a.message=Some("The host did not confirm delivery. Retry sends the exact prepared bytes; it may repeat the effect.".into());
            self.render(&mut a)
        }
    }
}
struct OpenGuard<'a> {
    adapter: &'a Adapter,
    id: String,
    keep: bool,
}
impl Drop for OpenGuard<'_> {
    fn drop(&mut self) {
        if !self.keep {
            if let Ok(mut map) = self.adapter.sessions.lock() {
                map.remove(&self.id);
            }
        }
    }
}
pub fn ack(method: &str, v: &Value) -> Result<(), String> {
    if method == "host.window.visibility" {
        return if v.is_null() {
            Ok(())
        } else {
            Err("Invalid visibility acknowledgement".into())
        };
    }
    let o = v.as_object().ok_or("Invalid host acknowledgement")?;
    if o.get("ok") != Some(&Value::Bool(true))
        || o.keys().any(|k| k != "ok" && k != "message")
        || o.get("message").is_some_and(|v| !v.is_string())
    {
        Err("Invalid host acknowledgement".into())
    } else {
        Ok(())
    }
}
impl Extension for Adapter {
    fn id(&self) -> &'static str {
        "com.tezbar.native.date-converter"
    }
    fn version(&self) -> &'static str {
        "2.0.0"
    }
    fn commands(&self) -> &'static [&'static str] {
        &["index"]
    }
    fn handle<'a>(
        &'a self,
        ctx: &'a ExtensionContext,
        command: &'a str,
        op: &'a str,
        p: &'a Value,
        _state: &'a mut Value,
    ) -> BoxFuture<'a, CoreResult> {
        Box::pin(async move {
            if command != "index" {
                return Err("Unknown command".into());
            }
            let id = p["sessionId"].as_str().ok_or("Missing sessionId")?;
            if op == "extension:dispose-session" {
                self.sessions
                    .lock()
                    .map_err(|_| "Session lock poisoned")?
                    .remove(id);
                return Ok(no_view("Closed"));
            }
            if op == "extension:run-command" {
                self.prune().await?;
                let (now, source) = self.clock()?;
                let prefs = Preferences::read(&p["preferences"])?;
                let mut a = Authority {
                    draft: p["searchText"].as_str().unwrap_or("").into(),
                    now,
                    source,
                    target: source,
                    target_error: None,
                    dates: vec![],
                    rows: vec![],
                    prefs,
                    message: None,
                    token: String::new(),
                    actions: HashMap::new(),
                    prepared: None,
                };
                let storage_error = match storage::load(&ctx.paths.data_dir) {
                    Ok(Some(s)) => match s.parse::<Tz>() {
                        Ok(t) => {
                            a.target = t;
                            None
                        }
                        Err(_) => {
                            let error = "Stored timezone is unsupported. Select a target timezone to recover.".to_string();
                            a.target_error = Some(error.clone());
                            Some(error)
                        }
                    },
                    Ok(None) => None,
                    Err(e) => Some(e),
                };
                self.parse(&mut a);
                if storage_error.is_some() {
                    a.rows.clear();
                    a.message = storage_error
                }
                let shared = Arc::new(Mutex::new(a));
                {
                    let mut map = self.sessions.lock().map_err(|_| "Session lock poisoned")?;
                    if map.len() >= 128 {
                        return Err("Adapter session capacity reached".into());
                    }
                    if map.contains_key(id) {
                        return Err("Duplicate session".into());
                    }
                    map.insert(id.into(), (shared.clone(), ctx.clone()));
                }
                let mut guard = OpenGuard {
                    adapter: self,
                    id: id.into(),
                    keep: false,
                };
                let out = self.render(&mut *shared.lock().map_err(|_| "Session lock poisoned")?);
                guard.keep = out.is_ok();
                return out;
            }
            let shared = self.session(id)?;
            match op {
                "extension:refresh-session" => {
                    let mut a = shared.lock().map_err(|_| "Session lock poisoned")?;
                    if a.prepared.is_some() || a.message.is_some() {
                        self.render(&mut a)
                    } else {
                        Ok(unchanged())
                    }
                }
                "extension:load-more" => Ok(unchanged()),
                "extension:search-text-changed" => {
                    let mut a = shared.lock().map_err(|_| "Session lock poisoned")?;
                    a.actions.clear();
                    a.rows.clear();
                    let draft = p["searchText"].as_str().ok_or("Missing searchText")?;
                    a.draft = draft.into();
                    if a.prepared.is_none() {
                        self.parse(&mut a)
                    }
                    self.render(&mut a)
                }
                "extension:invoke-action" => {
                    let act = {
                        let mut a = shared.lock().map_err(|_| "Session lock poisoned")?;
                        let f = &p["formValues"];
                        if f["__viewToken"].as_str() != Some(a.token.as_str())
                            || f["__searchText"].as_str() != Some(a.draft.as_str())
                            || f["__searchDirty"] != false
                        {
                            return Err("Draft changed; wait for current results".into());
                        }
                        let action_id = p["actionId"].as_str().ok_or("Missing actionId")?;
                        let act = a
                            .actions
                            .get(action_id)
                            .cloned()
                            .ok_or("Action is stale or already consumed")?;
                        if !matches!(act, Action::Timezone)
                            && (f["__draftDirty"] != false
                                || f["__accessoryValue"].as_str() != Some(a.target.name()))
                        {
                            return Err("Timezone draft changed; wait for current results".into());
                        }
                        a.actions.clear();
                        act
                    };
                    match act {
                        Action::Copy(text) => {
                            let copy = shared
                                .lock()
                                .map_err(|_| "Session lock poisoned")?
                                .prefs
                                .copy_action
                                .clone();
                            self.effect(
                                ctx,
                                &shared,
                                Prepared {
                                    method: if copy == "paste" {
                                        "host.clipboard.pasteText"
                                    } else {
                                        "host.clipboard.write"
                                    },
                                    text,
                                    remaining_paste: copy == "both",
                                },
                            )
                            .await
                        }
                        Action::Retry => {
                            let e = shared
                                .lock()
                                .map_err(|_| "Session lock poisoned")?
                                .prepared
                                .clone()
                                .ok_or("No prepared text")?;
                            self.effect(ctx, &shared, e).await
                        }
                        Action::Timezone => {
                            let s = p["formValues"]["value"]
                                .as_str()
                                .ok_or("Missing timezone")?;
                            let names: Vec<String> =
                                serde_json::from_str(include_str!("../resources/timezones.json"))
                                    .map_err(|e| e.to_string())?;
                            let mut a = shared.lock().map_err(|_| "Session lock poisoned")?;
                            if s != a.source.name() && !names.iter().any(|v| v == s) {
                                return Err("Unknown timezone".into());
                            }
                            let target = s
                                .parse::<Tz>()
                                .map_err(|_| "Timezone unsupported by native IANA data")?;
                            a.target = target;
                            a.target_error = None;
                            self.reformat(&mut a);
                            if let Err(e) = storage::save(&ctx.paths.data_dir, s) {
                                a.message = Some(match a.message.take() {
                                    Some(m) => format!("{m}; {e}"),
                                    None => e,
                                });
                            }
                            self.render(&mut a)
                        }
                        Action::Parse => {
                            let mut a = shared.lock().map_err(|_| "Session lock poisoned")?;
                            self.parse(&mut a);
                            self.render(&mut a)
                        }
                        Action::Settings => {
                            let r = ctx
                                .host(
                                    "host.settings.open",
                                    json!({"surface":"settings-extensions"}),
                                )
                                .await
                                .and_then(|v| ack("host.settings.open", &v));
                            let mut a = shared.lock().map_err(|_| "Session lock poisoned")?;
                            a.message = Some(
                                if r.is_ok() {
                                    "Reopen this command after changing preferences."
                                } else {
                                    "Failed to open preferences; retry."
                                }
                                .into(),
                            );
                            self.render(&mut a)
                        }
                    }
                }
                _ => Err("Unsupported operation".into()),
            }
        })
    }
}
