//! Native civil-date and dayjs 1.11.13 formatting primitives for Convert Date.
//! Date's implementation-dependent string fallback is deliberately bounded; see LIMITS.md.
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, Offset, Timelike};
use chrono_tz::Tz;

pub const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
pub const DAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
pub const MAX_TEXT: usize = 4096;
pub fn checked_text(s: &str) -> Result<(), String> {
    if s.len() > MAX_TEXT {
        Err("Input exceeds 4096 UTF-8 bytes".into())
    } else {
        Ok(())
    }
}
pub fn system_zone() -> Result<Tz, String> {
    if let Ok(s) = std::env::var("TZ") {
        return s
            .trim_start_matches(':')
            .parse()
            .map_err(|_| "Unsupported TZ; use an IANA timezone".into());
    }
    iana_time_zone::get_timezone()
        .map_err(|e| e.to_string())?
        .parse()
        .map_err(|_| "OS timezone is not in the pinned IANA database".into())
}
/// All offsets and recurrence come from embedded IANA 2026a, including beyond 2099.
/// chrono-tz is retained only as the case-sensitive zone identifier/inventory type.
fn zone(tz: Tz) -> Result<tz::TimeZoneRef<'static>, String> {
    tzdb_data::find_tz(tz.name().as_bytes())
        .copied()
        .ok_or_else(|| format!("Timezone {tz} missing from pinned IANA 2026a"))
}
pub fn local(ms: i64, tz: Tz) -> Result<DateTime<chrono::FixedOffset>, String> {
    let offset = zone(tz)?
        .find_local_time_type(ms.div_euclid(1000))
        .map_err(|e| e.to_string())?
        .ut_offset();
    let offset = chrono::FixedOffset::east_opt(offset).ok_or("Invalid IANA offset")?;
    DateTime::from_timestamp_millis(ms)
        .map(|d| d.with_timezone(&offset))
        .ok_or("Date outside native bounds".into())
}
pub fn bounded(d: NaiveDateTime) -> Result<NaiveDateTime, String> {
    if (-9999..=9999).contains(&d.year()) {
        Ok(d)
    } else {
        Err("Date outside supported civil years -9999..9999".into())
    }
}
/// JS compatible fold=earlier, gap=advance by the transition's displacement.
/// tz-rs evaluates the TZif extra rule as well as explicit historical transitions.
pub fn resolve(tz: Tz, n: NaiveDateTime, reject_gap: bool) -> Result<i64, String> {
    let found = tz::DateTime::find(
        n.year(),
        n.month() as u8,
        n.day() as u8,
        n.hour() as u8,
        n.minute() as u8,
        n.second() as u8,
        n.nanosecond(),
        zone(tz)?,
    )
    .map_err(|e| e.to_string())?;
    match found.into_inner().first() {
        Some(tz::datetime::FoundDateTimeKind::Normal(d)) => {
            Ok(d.unix_time() * 1000 + i64::from(n.nanosecond() / 1_000_000))
        }
        Some(tz::datetime::FoundDateTimeKind::Skipped {
            before_transition, ..
        }) if !reject_gap => Ok(n.and_utc().timestamp_millis()
            - i64::from(before_transition.local_time_type().ut_offset()) * 1000),
        Some(_) => Err("Invalid local date in a daylight-saving gap".into()),
        None => Err("Unresolvable local date".into()),
    }
}
// Keep the positional JS Date constructor fields visible in the translation.
#[allow(clippy::too_many_arguments)]
pub fn normalized(
    y: i32,
    m: i32,
    d: i32,
    h: i64,
    min: i64,
    s: i64,
    ms: i64,
    js_year: bool,
) -> Result<NaiveDateTime, String> {
    let y = if js_year && (0..=99).contains(&y) {
        1900 + y
    } else {
        y
    };
    let y = y
        .checked_add((m - 1).div_euclid(12))
        .ok_or("Date overflow")?;
    let m = (m - 1).rem_euclid(12) + 1;
    let base = NaiveDate::from_ymd_opt(y, m as u32, 1)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .ok_or("Date overflow")?;
    let delta = i64::from(d - 1)
        .checked_mul(86400000)
        .and_then(|v| v.checked_add(h.checked_mul(3600000)?))
        .and_then(|v| v.checked_add(min.checked_mul(60000)?))
        .and_then(|v| v.checked_add(s.checked_mul(1000)?))
        .and_then(|v| v.checked_add(ms))
        .ok_or("Date overflow")?;
    Ok(base
        .checked_add_signed(Duration::milliseconds(delta))
        .ok_or("Date overflow")?)
}
pub fn format(ms: i64, tz: Tz, fmt: &str, advanced: bool) -> Result<String, String> {
    format_fixed(local(ms, tz)?.fixed_offset(), fmt, advanced)
}
pub fn format_fixed(
    d: chrono::DateTime<chrono::FixedOffset>,
    fmt: &str,
    advanced: bool,
) -> Result<String, String> {
    checked_text(fmt)?;
    bounded(d.naive_local())?;
    let ms = d.timestamp_millis();
    let y = d.year();
    let mo = d.month() as usize;
    let day = d.day();
    let wd = d.weekday().num_days_from_sunday() as usize;
    let h = d.hour();
    let h12 = if h % 12 == 0 { 12 } else { h % 12 };
    let off = d.offset().fix().local_minus_utc() / 60;
    let sign = if off < 0 { '-' } else { '+' };
    let off = off.abs();
    let z = format!("{sign}{:02}:{:02}", off / 60, off % 60);
    let fmt = if fmt.is_empty() {
        "YYYY-MM-DDTHH:mm:ssZ"
    } else {
        fmt
    };
    let mut expanded = String::new();
    let mut i = 0;
    // localizedFormat is loaded only in index.tsx; English is dayjs's actual default,
    // independent of OS locale. Natural command has no plugins.
    while i < fmt.len() {
        let s = &fmt[i..];
        if s.starts_with('[') {
            if let Some(j) = s.find(']') {
                expanded.push_str(&s[..=j]);
                i += j + 1;
                continue;
            }
        }
        if advanced {
            if let Some((t, r)) = [
                ("LLLL", "dddd, MMMM D, YYYY h:mm A"),
                ("LLL", "MMMM D, YYYY h:mm A"),
                ("LTS", "h:mm:ss A"),
                ("LL", "MMMM D, YYYY"),
                ("LT", "h:mm A"),
                ("L", "MM/DD/YYYY"),
            ]
            .into_iter()
            .find(|(t, _)| s.starts_with(t))
            {
                expanded.push_str(r);
                i += t.len();
                continue;
            }
        }
        let c = s.chars().next().unwrap();
        expanded.push(c);
        i += c.len_utf8();
    }
    let mut out = String::new();
    let mut i = 0;
    while i < expanded.len() {
        let s = &expanded[i..];
        if s.starts_with('[') {
            if let Some(j) = s.find(']') {
                out.push_str(&s[1..j]);
                i += j + 1;
                continue;
            }
        }
        if advanced {
            if let Some(t) = [
                "gggg", "GGGG", "zzz", "wo", "ww", "WW", "Do", "kk", "Q", "X", "x", "k", "w", "W",
                "z",
            ]
            .into_iter()
            .find(|t| s.starts_with(t))
            {
                let r = match t {
                    "Q" => mo.div_ceil(3).to_string(),
                    "X" => ms.div_euclid(1000).to_string(),
                    "x" => ms.to_string(),
                    "k" => if h == 0 { 24 } else { h }.to_string(),
                    "kk" => format!("{:02}", if h == 0 { 24 } else { h }),
                    "Do" => format!(
                        "{day}{}",
                        if (11..=13).contains(&(day % 100)) {
                            "th"
                        } else {
                            match day % 10 {
                                1 => "st",
                                2 => "nd",
                                3 => "rd",
                                _ => "th",
                            }
                        }
                    ),
                    _ => {
                        return Err(format!(
                            "Original dayjs throws for missing plugin token {t}"
                        ))
                    }
                };
                out.push_str(&r);
                i += t.len();
                continue;
            }
        }
        if let Some(t) = [
            "YYYY", "MMMM", "dddd", "MMM", "ddd", "SSS", "YY", "MM", "DD", "dd", "HH", "hh", "mm",
            "ss", "ZZ", "M", "D", "d", "H", "h", "m", "s", "Z", "A", "a",
        ]
        .into_iter()
        .find(|t| s.starts_with(t))
        {
            let r = match t {
                "YYYY" => {
                    let s = y.to_string();
                    format!("{}{}", "0".repeat(4usize.saturating_sub(s.len())), s)
                }
                "YY" => format!("{:02}", y % 100),
                "MMMM" => MONTHS[mo - 1].into(),
                "MMM" => MONTHS[mo - 1][..3].into(),
                "MM" => format!("{mo:02}"),
                "M" => mo.to_string(),
                "DD" => format!("{day:02}"),
                "D" => day.to_string(),
                "dddd" => DAYS[wd].into(),
                "ddd" => DAYS[wd][..3].into(),
                "dd" => DAYS[wd][..2].into(),
                "d" => wd.to_string(),
                "HH" => format!("{h:02}"),
                "H" => h.to_string(),
                "hh" => format!("{h12:02}"),
                "h" => h12.to_string(),
                "mm" => format!("{:02}", d.minute()),
                "m" => d.minute().to_string(),
                "ss" => format!("{:02}", d.second()),
                "s" => d.second().to_string(),
                "SSS" => format!("{:03}", d.timestamp_subsec_millis()),
                "Z" => z.clone(),
                "ZZ" => z.replace(':', ""),
                "A" => if h < 12 { "AM" } else { "PM" }.into(),
                "a" => if h < 12 { "am" } else { "pm" }.into(),
                _ => unreachable!(),
            };
            out.push_str(&r);
            i += t.len();
        } else {
            let c = s.chars().next().unwrap();
            out.push(c);
            i += c.len_utf8()
        }
    }
    Ok(out)
}
