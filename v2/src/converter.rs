//! Source index.tsx semantics, using chrono-node 2.7.8/dayjs 1.11.13 translations.
use crate::{date, natural};
use chrono::{Datelike, Offset, SecondsFormat, Timelike};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub const DEFAULT: &str = "dddd, MMMM D, YYYY hh:mm:ss";
pub const FORMATS: [(&str, &str); 4] = [
    ("human", "Human Date"),
    ("unix-ms", "Unix Timestamp (ms)"),
    ("unix-s", "Unix Timestamp (seconds)"),
    ("iso", "ISO Date"),
];
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preferences {
    pub default_format: String,
    pub secondary_format: String,
    pub copy_action: String,
    pub display: String,
    pub human: String,
}
impl Preferences {
    pub fn read(p: &Value) -> Result<Self, String> {
        fn text(p: &Value, k: &str, d: &str) -> Result<String, String> {
            match p.get(k) {
                None => Ok(d.into()),
                Some(Value::String(s)) => {
                    date::checked_text(s)?;
                    Ok(s.clone())
                }
                _ => Err(format!("Preference {k} must be text")),
            }
        }
        let v = Self {
            default_format: text(p, "defaultFormat", "unix-ms")?,
            secondary_format: text(p, "secondaryFormat", "iso")?,
            copy_action: text(p, "copyAction", "both")?,
            display: text(p, "displayFormat", DEFAULT)?,
            human: text(p, "humanFormat", DEFAULT)?,
        };
        if !["copy", "paste", "both"].contains(&v.copy_action.as_str()) {
            return Err(
                "Invalid copy action preference; correct it and reopen this command".into(),
            );
        }
        Ok(v)
    }
    pub fn order(&self) -> Vec<(&'static str, &'static str)> {
        let mut f = FORMATS.to_vec();
        f.sort_by_key(|(id, _)| {
            if *id == self.default_format {
                0
            } else if *id == self.secondary_format {
                1
            } else {
                2
            }
        });
        f
    }
}
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ResultDate {
    pub ms: i64,
    pub label: String,
    pub title: String,
    pub subtitle: String,
    pub values: Vec<(String, String)>,
}
pub fn iso(ms: i64) -> Result<String, String> {
    Ok(date::local(ms, chrono_tz::UTC)?.to_rfc3339_opts(SecondsFormat::Millis, true))
}
/// Translate dayjs 1.11.13 instance .tz(): its source-local Date reparse and
/// 15-minute source-offset rounding are observable around DST transitions.
pub fn zoned(
    ms: i64,
    source: Tz,
    target: Tz,
) -> Result<chrono::DateTime<chrono::FixedOffset>, String> {
    use chrono::TimeZone;
    for zone in [source, target] {
        let civil = date::local(ms, zone)?;
        if !(100..=9999).contains(&civil.year()) {
            return Err("Outside supported civil years 0100..9999 (engine-specific early-year reparsing or overflow)".into());
        }
        if civil.offset().fix().local_minus_utc() % 60 != 0 {
            return Err("Historical subminute timezone offsets are outside native support".into());
        }
    }
    let source_date = date::local(ms, source)?;
    let naive = date::local(ms, target)?.naive_local();
    let local_parse = date::resolve(source, naive.with_nanosecond(0).unwrap(), false)?;
    let diff = ((ms as f64 - local_parse as f64) / 60000.0 + 0.5).floor() as i32;
    let offset = -((-f64::from(source_date.offset().fix().local_minus_utc() / 60) / 15.0 + 0.5)
        .floor() as i32)
        * 15
        - diff;
    if offset == 0 {
        return Ok(date::local(ms, chrono_tz::UTC)?.fixed_offset());
    }
    let offset = if offset.abs() <= 16 {
        offset * 60
    } else {
        offset
    };
    let civil = date::local(local_parse, source)?
        .naive_local()
        .with_nanosecond(source_date.nanosecond())
        .unwrap();
    chrono::FixedOffset::east_opt(offset * 60)
        .and_then(|tz| tz.from_local_datetime(&civil).single())
        .ok_or("Invalid converted timezone offset".into())
}
pub fn display(ms: i64, source: Tz, target: Tz, p: &Preferences) -> Result<String, String> {
    date::format_fixed(
        zoned(ms, source, target)?,
        if trim(&p.display).is_empty() {
            DEFAULT
        } else {
            trim(&p.display)
        },
        false,
    )
}
pub fn format(
    ms: i64,
    source: Tz,
    target: Tz,
    id: &str,
    p: &Preferences,
) -> Result<String, String> {
    let d = zoned(ms, source, target)?;
    match id {
        "human" => date::format_fixed(
            d,
            if trim(&p.human).is_empty() {
                DEFAULT
            } else {
                trim(&p.human)
            },
            false,
        ),
        "unix-ms" => Ok(d.timestamp_millis().to_string()),
        "unix-s" => Ok(d.timestamp_millis().div_euclid(1000).to_string()),
        "iso" => iso(d.timestamp_millis()),
        _ => Err("Unknown format".into()),
    }
}
pub fn trim(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
}
fn machine(s: &str, tz: Tz) -> Result<Option<(i64, String)>, String> {
    let bytes = s.as_bytes();
    // Only the source's uppercase ISO prefix is eligible; Date handles rollover and offsets.
    if let Some(c) = regex::Regex::new(r"^([0-9]{4})-([0-9]{2})-([0-9]{2})T([0-9]{2}):([0-9]{2}):([0-9]{2})(?:\.([0-9]+))?([Zz]|([+-])([0-9]{2}):?([0-9]{2}))?$").unwrap().captures(s) {
        let n=|i:usize|c.get(i).and_then(|v|v.as_str().parse::<i64>().ok()).unwrap_or(0);
        let fraction=c.get(7).map(|v|v.as_str().to_string()).unwrap_or_else(||"000".into());
        let fraction=format!("{}000",fraction);let ms=fraction[..3].parse::<i64>().unwrap_or(0);
        let (y,m,d,h,min,sec)=(n(1),n(2),n(3),n(4),n(5),n(6));
        if (1..=12).contains(&m)&&(1..=31).contains(&d)&&h<=24&&min<=59&&sec<=59&&n(10)<=23&&n(11)<=59&&!(h==24&&(min!=0||sec!=0||ms!=0)) {
            if let Ok(local)=date::normalized(y as i32,m as i32,d as i32,h,min,sec,ms,false) {
                let instant=if c.get(8).is_none(){date::resolve(tz,local,false).ok()}else{Some(local.and_utc().timestamp_millis()-(n(10)*60+n(11))*60000*if c.get(9).is_some_and(|v|v.as_str()=="-"){-1}else{1})};
                if let Some(instant)=instant{return Ok(Some((instant,"ISO Date".into())))}
            }
        }
    }
    if (s.len() == 19 || s.len() == 16) && bytes.iter().all(u8::is_ascii_digit) {
        let ms = s.parse::<u64>().map_err(|_| "Timestamp overflow")?
            / if s.len() == 19 { 1000000 } else { 1000 };
        return Ok(Some((
            ms as i64,
            if s.len() == 19 {
                "Unix Timestamp (ns)"
            } else {
                "Unix Timestamp (μs)"
            }
            .into(),
        )));
    }
    let hex = regex::Regex::new(r"(?i)^0x[0-9a-f]+$").unwrap().is_match(s);
    let mut t = if hex { &s[2..] } else { s };
    let neg = t.starts_with('-');
    t = t.trim_start_matches(['+', '-']);
    let digits = t
        .chars()
        .take_while(|c| c.is_digit(if hex { 16 } else { 10 }))
        .collect::<String>();
    if digits.is_empty() {
        return Ok(None);
    }
    let mut value = 0.0;
    for c in digits.chars() {
        value = value * if hex { 16.0 } else { 10.0 }
            + f64::from(c.to_digit(if hex { 16 } else { 10 }).unwrap());
    }
    if neg {
        value = -value
    }
    if value > 1000000.0 {
        let seconds = value <= 2147483648.0;
        if seconds {
            value *= 1000.0
        }
        if value.is_finite() && value <= 8.64e15 {
            return Ok(Some((
                value.trunc() as i64,
                if seconds {
                    "Unix Timestamp (seconds)"
                } else {
                    "Unix Timestamp (ms)"
                }
                .into(),
            )));
        }
    }
    Ok(None)
}
/// javascript-time-ago default roundMinute style; English long labels, numeric always.
pub fn time_ago(ms: i64, now: i64) -> String {
    let elapsed = (now as f64 - ms as f64) / 1000.0;
    let amount = elapsed.abs();
    let units = [
        ("minute", 60.0),
        ("hour", 3600.0),
        ("day", 86400.0),
        ("week", 604800.0),
        ("month", 30.44 * 86400.0),
        ("year", 146097.0 / 400.0 * 86400.0),
    ];
    let mut selected = None;
    let mut prev = 60.0;
    for (i, (unit, size)) in units.into_iter().enumerate() {
        let threshold = if i == 0 { 30.0 } else { size - prev * 0.5 };
        if amount < threshold {
            break;
        }
        selected = Some((unit, size));
        prev = size
    }
    let Some((unit, size)) = selected else {
        return if elapsed < 0.0 {
            "in a moment"
        } else {
            "just now"
        }
        .into();
    };
    let n = (amount / size + 0.5).floor() as u64;
    let raw = n.to_string();
    let n = raw
        .chars()
        .enumerate()
        .flat_map(|(i, c)| {
            let mut s = String::new();
            if i > 0 && (raw.len() - i) % 3 == 0 {
                s.push(',')
            }
            s.push(c);
            s.chars().collect::<Vec<_>>()
        })
        .collect::<String>();
    let plural = if n == "1" { "" } else { "s" };
    if elapsed < 0.0 {
        format!("in {n} {unit}{plural}")
    } else {
        format!("{n} {unit}{plural} ago")
    }
}
#[derive(Clone, Debug)]
pub struct ParsedDate {
    pub ms: i64,
    pub label: String,
}
pub fn parse_dates(query: &str, now: i64, source: Tz) -> Result<Vec<ParsedDate>, String> {
    date::checked_text(query)?;
    let mut dates = if query.is_empty() {
        let n = date::local(now, source)?.naive_local();
        let midnight = n
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();
        let today = date::resolve(source, midnight, false)?;
        vec![
            (now, "Now".into()),
            (today, "Today".into()),
            (today + 86400000, "Tomorrow".into()),
            (today - 86400000, "Yesterday".into()),
        ]
    } else {
        let s = trim(query);
        let mut v = machine(s, source)?.into_iter().collect::<Vec<_>>();
        v.extend(natural::parse_all(s, now, source)?);
        v
    };
    let mut seen = std::collections::HashSet::new();
    dates.retain(|(ms, _)| seen.insert(*ms));
    Ok(dates
        .into_iter()
        .map(|(ms, label)| ParsedDate { ms, label })
        .collect())
}
pub fn render_dates(
    dates: &[ParsedDate],
    now: i64,
    source: Tz,
    target: Tz,
    p: &Preferences,
) -> Result<Vec<ResultDate>, String> {
    dates
        .iter()
        .map(|parsed| {
            let ms = parsed.ms;
            let label = parsed.label.clone();
            let title = display(ms, source, target, p)?;
            let values = p
                .order()
                .into_iter()
                .map(|(id, _)| Ok((id.into(), format(ms, source, target, id, p)?)))
                .collect::<Result<Vec<_>, String>>()?;
            Ok(ResultDate {
                ms,
                subtitle: format!("{label} - {}", time_ago(ms, now)),
                label,
                title,
                values,
            })
        })
        .collect()
}
pub fn results(
    query: &str,
    now: i64,
    source: Tz,
    target: Tz,
    p: &Preferences,
) -> Result<Vec<ResultDate>, String> {
    render_dates(&parse_dates(query, now, source)?, now, source, target, p)
}
