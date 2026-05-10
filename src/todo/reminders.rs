use std::io::Write;

use chrono::{
    DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, TimeZone, Utc, Weekday,
};

use super::{TodoState, model::ReminderRepeat, notifications};

pub fn parse_local_datetime_to_utc(input: &str) -> Option<DateTime<Utc>> {
    parse_human_datetime_to_utc(input)
}

pub fn parse_local_reminder_to_utc_and_repeat(
    input: &str,
) -> Option<(DateTime<Utc>, Option<ReminderRepeat>)> {
    parse_human_reminder_to_utc_and_repeat(input)
}

pub fn parse_local_date_to_utc_end_of_day(input: &str) -> Option<DateTime<Utc>> {
    let date = parse_human_date(input)?;
    let naive = date.and_hms_opt(23, 59, 0)?;
    let local = Local.from_local_datetime(&naive).single()?;
    Some(local.with_timezone(&Utc))
}

fn parse_human_datetime_to_utc(input: &str) -> Option<DateTime<Utc>> {
    parse_human_reminder_to_utc_and_repeat(input).map(|(dt, _)| dt)
}

fn parse_human_reminder_to_utc_and_repeat(
    input: &str,
) -> Option<(DateTime<Utc>, Option<ReminderRepeat>)> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }

    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M") {
        let local = Local.from_local_datetime(&naive).single()?;
        return Some((local.with_timezone(&Utc), None));
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y/%m/%d %H:%M") {
        let local = Local.from_local_datetime(&naive).single()?;
        return Some((local.with_timezone(&Utc), None));
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y.%m.%d %H:%M") {
        let local = Local.from_local_datetime(&naive).single()?;
        return Some((local.with_timezone(&Utc), None));
    }

    if let Some(dt) = parse_relative_datetime(s) {
        return Some((dt, None));
    }

    if let Some((date, rest)) = parse_relative_day_prefix(s) {
        if let Some((h, m)) = parse_human_time(rest) {
            return local_date_time_to_utc(date, h, m).map(|dt| (dt, None));
        }
        return local_date_time_to_utc(date, 9, 0).map(|dt| (dt, None));
    }

    if let Some((date, rest, is_every)) = parse_weekday_prefix_with_every(s) {
        let (h, m) = parse_human_time(rest).unwrap_or((9, 0));
        let Some(mut dt) = local_date_time_to_utc(date, h, m) else {
            return None;
        };
        if dt <= Utc::now() {
            dt = dt + Duration::days(7);
        }
        let repeat = if is_every {
            let weekday = date.weekday().num_days_from_monday() as u8;
            Some(ReminderRepeat::Weekly {
                weekday,
                hour: h as u8,
                minute: m as u8,
            })
        } else {
            None
        };
        return Some((dt, repeat));
    }

    if let Some((date, time_s)) = split_date_and_time(s) {
        let date = parse_date_token(date)?;
        let (h, m) = parse_human_time(time_s)?;
        return local_date_time_to_utc(date, h, m).map(|dt| (dt, None));
    }

    if let Some((h, m)) = parse_human_time(s) {
        let now_local = Local::now();
        let today = NaiveDate::from_ymd_opt(now_local.year(), now_local.month(), now_local.day())?;
        let Some(mut dt) = local_date_time_to_utc(today, h, m) else {
            return None;
        };
        if dt <= Utc::now() {
            dt = dt + Duration::days(1);
        }
        return Some((dt, None));
    }

    None
}

fn split_date_and_time(s: &str) -> Option<(&str, &str)> {
    let mut it = s.split_whitespace();
    let date = it.next()?;
    let time = it.next()?;
    if it.next().is_some() {
        return None;
    }
    Some((date, time))
}

fn parse_relative_day_prefix(s: &str) -> Option<(NaiveDate, &str)> {
    let s = s.trim_start();
    let lower = s.to_ascii_lowercase();
    let (days, rest) = if let Some(rest) = s.strip_prefix("今天") {
        (0, rest)
    } else if let Some(rest) = s.strip_prefix("明天") {
        (1, rest)
    } else if let Some(rest) = s.strip_prefix("后天") {
        (2, rest)
    } else if lower.starts_with("today") {
        (0, &s[5..])
    } else if lower.starts_with("tomorrow") {
        (1, &s[8..])
    } else {
        return None;
    };

    let now_local = Local::now();
    let base = NaiveDate::from_ymd_opt(now_local.year(), now_local.month(), now_local.day())?;
    Some((base + Duration::days(days), rest.trim()))
}

const MON_ALIASES: &[&str] = &["周一", "星期一", "礼拜一", "mon", "monday"];
const TUE_ALIASES: &[&str] = &["周二", "星期二", "礼拜二", "tue", "tues", "tuesday"];
const WED_ALIASES: &[&str] = &["周三", "星期三", "礼拜三", "wed", "wednesday"];
const THU_ALIASES: &[&str] = &[
    "周四",
    "星期四",
    "礼拜四",
    "thu",
    "thur",
    "thurs",
    "thursday",
];
const FRI_ALIASES: &[&str] = &["周五", "星期五", "礼拜五", "fri", "friday"];
const SAT_ALIASES: &[&str] = &["周六", "星期六", "礼拜六", "sat", "saturday"];
const SUN_ALIASES: &[&str] = &[
    "周日",
    "周天",
    "星期日",
    "星期天",
    "礼拜日",
    "礼拜天",
    "sun",
    "sunday",
];

const WEEKDAY_ALIAS_GROUPS: &[(Weekday, &[&str])] = &[
    (Weekday::Mon, MON_ALIASES),
    (Weekday::Tue, TUE_ALIASES),
    (Weekday::Wed, WED_ALIASES),
    (Weekday::Thu, THU_ALIASES),
    (Weekday::Fri, FRI_ALIASES),
    (Weekday::Sat, SAT_ALIASES),
    (Weekday::Sun, SUN_ALIASES),
];

fn parse_weekday_prefix_with_every(s: &str) -> Option<(NaiveDate, &str, bool)> {
    let s = s.trim_start();
    let (weekday, rest, is_every) = parse_weekday_token_and_rest_with_every(s)?;

    let now_local = Local::now();
    let base = NaiveDate::from_ymd_opt(now_local.year(), now_local.month(), now_local.day())?;
    let date = next_weekday_date_from(base, weekday, true);
    Some((date, rest, is_every))
}

fn parse_weekday_token_and_rest(s: &str) -> Option<(Weekday, &str)> {
    parse_weekday_token_and_rest_with_every(s).map(|(wd, rest, _)| (wd, rest))
}

fn parse_weekday_token_and_rest_with_every(s: &str) -> Option<(Weekday, &str, bool)> {
    let s = s.trim_start();
    let lower = s.to_ascii_lowercase();

    let mut start_index = 0usize;
    let mut is_every = false;
    if s.starts_with('每') {
        start_index = '每'.len_utf8();
        is_every = true;
    } else if lower.starts_with("every ") {
        start_index = "every ".len();
        is_every = true;
    }

    let tail = s[start_index..].trim_start();
    let lower_tail = tail.to_ascii_lowercase();

    for (weekday, aliases) in WEEKDAY_ALIAS_GROUPS {
        for alias in *aliases {
            if lower_tail.starts_with(alias) {
                return Some((*weekday, tail[alias.len()..].trim(), is_every));
            }
        }
    }

    None
}

fn next_weekday_date_from(base: NaiveDate, target: Weekday, include_today: bool) -> NaiveDate {
    let current = base.weekday();
    let cur = current.num_days_from_monday() as i32;
    let tar = target.num_days_from_monday() as i32;
    let mut delta = (tar - cur + 7) % 7;
    if delta == 0 && !include_today {
        delta = 7;
    }
    base + Duration::days(delta as i64)
}

fn local_date_time_to_utc(date: NaiveDate, hour: u32, minute: u32) -> Option<DateTime<Utc>> {
    let naive = date.and_hms_opt(hour, minute, 0)?;
    let local = Local.from_local_datetime(&naive).single()?;
    Some(local.with_timezone(&Utc))
}

fn parse_relative_datetime(s: &str) -> Option<DateTime<Utc>> {
    let raw = s.trim();
    let rest = raw.strip_prefix('+')?;
    let (n, unit) = parse_number_prefix(rest)?;
    let dur = match unit {
        "m" | "min" | "mins" | "minute" | "minutes" | "分钟" => Duration::minutes(n as i64),
        "h" | "hr" | "hrs" | "hour" | "hours" | "小时" => Duration::hours(n as i64),
        "d" | "day" | "days" | "天" => Duration::days(n as i64),
        _ => return None,
    };
    Some(Utc::now() + dur)
}

fn parse_number_prefix(s: &str) -> Option<(u64, &str)> {
    let s = s.trim();
    let mut end = 0usize;
    for (i, ch) in s.char_indices() {
        if ch.is_ascii_digit() {
            end = i + ch.len_utf8();
        } else {
            break;
        }
    }
    if end == 0 {
        return None;
    }
    let n: u64 = s[..end].parse().ok()?;
    let unit = s[end..].trim();
    Some((n, unit))
}

fn parse_human_date(input: &str) -> Option<NaiveDate> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }

    let now_local = Local::now();
    let today = NaiveDate::from_ymd_opt(now_local.year(), now_local.month(), now_local.day())?;

    if s == "今天" || s.eq_ignore_ascii_case("today") {
        return Some(today);
    }
    if s == "明天" || s.eq_ignore_ascii_case("tomorrow") {
        return Some(today + Duration::days(1));
    }
    if s == "后天" {
        return Some(today + Duration::days(2));
    }

    if let Some((weekday, _rest)) = parse_weekday_token_and_rest(s) {
        return Some(next_weekday_date_from(today, weekday, true));
    }

    parse_date_token(s)
}

fn parse_date_token(token: &str) -> Option<NaiveDate> {
    let t = token.trim();
    if let Ok(date) = NaiveDate::parse_from_str(t, "%Y-%m-%d") {
        return Some(date);
    }
    if let Ok(date) = NaiveDate::parse_from_str(t, "%Y/%m/%d") {
        return Some(date);
    }
    if let Ok(date) = NaiveDate::parse_from_str(t, "%Y.%m.%d") {
        return Some(date);
    }

    let now_local = Local::now();
    let year = now_local.year();
    if let Some((m, d)) = parse_month_day_token(t) {
        return NaiveDate::from_ymd_opt(year, m, d);
    }

    None
}

fn parse_month_day_token(t: &str) -> Option<(u32, u32)> {
    if let Some((m, d)) = t.split_once('-') {
        let m: u32 = m.trim().parse().ok()?;
        let d: u32 = d.trim().parse().ok()?;
        return Some((m, d));
    }
    if let Some((m, d)) = t.split_once('/') {
        let m: u32 = m.trim().parse().ok()?;
        let d: u32 = d.trim().parse().ok()?;
        return Some((m, d));
    }
    if let Some((m, rest)) = t.split_once('月') {
        let m: u32 = m.trim().parse().ok()?;
        let d = rest.trim_end_matches('日').trim();
        let d: u32 = d.parse().ok()?;
        return Some((m, d));
    }
    None
}

fn parse_human_time(input: &str) -> Option<(u32, u32)> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }

    if let Some((h, m)) = parse_colon_time(s) {
        return Some((h, m));
    }

    parse_chinese_time(s)
}

fn parse_colon_time(s: &str) -> Option<(u32, u32)> {
    let (h, m) = s.split_once(':')?;
    let h: u32 = h.trim().parse().ok()?;
    let m: u32 = m.trim().parse().ok()?;
    if h > 23 || m > 59 {
        return None;
    }
    Some((h, m))
}

fn parse_chinese_time(s: &str) -> Option<(u32, u32)> {
    let mut rest = s.trim();
    let mut pm = false;

    if let Some(r) = rest.strip_prefix("下午") {
        pm = true;
        rest = r.trim();
    } else if let Some(r) = rest.strip_prefix("晚上") {
        pm = true;
        rest = r.trim();
    } else if let Some(r) = rest.strip_prefix("中午") {
        pm = true;
        rest = r.trim();
    } else if let Some(r) = rest.strip_prefix("上午") {
        rest = r.trim();
    } else if let Some(r) = rest.strip_prefix("凌晨") {
        rest = r.trim();
    }

    let (hour, minute) = if let Some((h_s, after)) = rest.split_once('点') {
        let h: u32 = h_s.trim().parse().ok()?;
        let after = after.trim();
        if after.is_empty() {
            (h, 0)
        } else if after.starts_with('半') {
            (h, 30)
        } else {
            let after = after.trim_end_matches('分').trim();
            let m: u32 = after.parse().ok()?;
            (h, m)
        }
    } else {
        return None;
    };

    if hour > 23 || minute > 59 {
        return None;
    }

    let hour = if pm && hour < 12 { hour + 12 } else { hour };
    Some((hour, minute))
}

pub fn write_reminders_ics(state: &TodoState) {
    let Some(folder) = state.get_save_folder_path() else {
        return;
    };
    let path = folder.join("reminders.ics");

    let now = Utc::now();
    let mut events: Vec<(DateTime<Utc>, String, String)> = Vec::new();

    for item in &state.items {
        if item.deleted_at.is_some() {
            continue;
        }
        if item.completed {
            continue;
        }
        let Some(at) = item.reminder_at else {
            continue;
        };
        if at <= now {
            continue;
        }

        let uid = item.id.to_string();
        let title = escape_ics_text(&item.title);
        let description = escape_ics_text(
            &item
                .description
                .clone()
                .unwrap_or_else(|| "Todo Reminder".to_string()),
        );
        events.push((at, uid, format!("{title}\n{description}")));
    }

    let mut ics = String::new();
    ics.push_str("BEGIN:VCALENDAR\r\n");
    ics.push_str("VERSION:2.0\r\n");
    ics.push_str("PRODID:-//eframe_template//Todo Reminders//EN\r\n");
    ics.push_str("CALSCALE:GREGORIAN\r\n");
    ics.push_str("METHOD:PUBLISH\r\n");

    let dtstamp = format_ics_datetime(now);
    for (at, uid, summary) in events {
        ics.push_str("BEGIN:VEVENT\r\n");
        ics.push_str(&format!("UID:{uid}\r\n"));
        ics.push_str(&format!("DTSTAMP:{dtstamp}\r\n"));
        ics.push_str(&format!("DTSTART:{}\r\n", format_ics_datetime(at)));
        ics.push_str(&format!("SUMMARY:{}\r\n", escape_ics_text(&summary)));
        ics.push_str("BEGIN:VALARM\r\n");
        ics.push_str("TRIGGER:-PT0M\r\n");
        ics.push_str("ACTION:DISPLAY\r\n");
        ics.push_str(&format!("DESCRIPTION:{}\r\n", escape_ics_text(&summary)));
        ics.push_str("END:VALARM\r\n");
        ics.push_str("END:VEVENT\r\n");
    }

    ics.push_str("END:VCALENDAR\r\n");

    let Ok(mut tmp) = tempfile::NamedTempFile::new_in(&folder) else {
        return;
    };
    if tmp.write_all(ics.as_bytes()).is_err() {
        return;
    }
    if tmp.flush().is_err() {
        return;
    }

    let _ = tmp.persist(path);
}

pub fn next_reminder_from_repeat(
    repeat: ReminderRepeat,
    after_utc: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    compute_next_reminder_from_repeat(Some(repeat), after_utc)
}

pub fn poll_due_reminders_and_notify(state: &mut TodoState) -> bool {
    let now = Utc::now();
    let mut changed = false;

    for item in &mut state.items {
        if item.deleted_at.is_some() {
            continue;
        }
        if item.completed {
            continue;
        }
        if item.reminder_at.is_none() {
            if let Some(next) = compute_next_reminder_from_repeat(item.reminder_repeat, now) {
                item.reminder_at = Some(next);
                item.reminder_sent = false;
                changed = true;
            }
            continue;
        }
        let at = item.reminder_at.unwrap_or(now);
        if at > now {
            continue;
        }
        if item.reminder_sent {
            continue;
        }

        let title = "Todo 提醒 (Todo Reminder)";
        let body = item.title.as_str();
        notifications::send_system_notification(title, body);
        if let Some(next) = compute_next_reminder_from_repeat(item.reminder_repeat, now) {
            item.reminder_at = Some(next);
            item.reminder_sent = false;
        } else {
            item.reminder_sent = true;
        }
        changed = true;
    }

    changed
}

fn compute_next_reminder_from_repeat(
    repeat: Option<ReminderRepeat>,
    after_utc: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    match repeat? {
        ReminderRepeat::Weekly {
            weekday,
            hour,
            minute,
        } => next_weekly_reminder_utc(weekday, hour, minute, after_utc),
    }
}

fn next_weekly_reminder_utc(
    weekday: u8,
    hour: u8,
    minute: u8,
    after_utc: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    let target = weekday_from_u8(weekday)?;
    let after_local = after_utc.with_timezone(&Local);
    let base = NaiveDate::from_ymd_opt(after_local.year(), after_local.month(), after_local.day())?;
    let date = next_weekday_date_from(base, target, true);
    let Some(mut next) = local_date_time_to_utc(date, hour as u32, minute as u32) else {
        return None;
    };
    if next <= after_utc {
        next = next + Duration::days(7);
    }
    Some(next)
}

fn weekday_from_u8(n: u8) -> Option<Weekday> {
    match n {
        0 => Some(Weekday::Mon),
        1 => Some(Weekday::Tue),
        2 => Some(Weekday::Wed),
        3 => Some(Weekday::Thu),
        4 => Some(Weekday::Fri),
        5 => Some(Weekday::Sat),
        6 => Some(Weekday::Sun),
        _ => None,
    }
}

fn format_ics_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%Y%m%dT%H%M%SZ").to_string()
}

fn escape_ics_text(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace(',', "\\,")
        .replace(';', "\\;")
        .replace('\r', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_time_variants() {
        assert_eq!(parse_human_time("20:30"), Some((20, 30)));
        assert_eq!(parse_human_time("9:05"), Some((9, 5)));
        assert_eq!(parse_human_time("9点"), Some((9, 0)));
        assert_eq!(parse_human_time("9点半"), Some((9, 30)));
        assert_eq!(parse_human_time("下午3点"), Some((15, 0)));
        assert_eq!(parse_human_time("晚上11点30"), Some((23, 30)));
    }

    #[test]
    fn parse_date_variants() {
        assert_eq!(
            parse_date_token("2026-05-10"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap())
        );
        assert_eq!(
            parse_date_token("2026/05/10"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap())
        );
        assert_eq!(
            parse_date_token("2026.05.10"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap())
        );
    }

    #[test]
    fn weekday_mapping_and_next() {
        let (wd, _) = parse_weekday_token_and_rest("周一").unwrap();
        assert_eq!(wd, Weekday::Mon);
        let (wd, _) = parse_weekday_token_and_rest("每周二").unwrap();
        assert_eq!(wd, Weekday::Tue);
        let (wd, _) = parse_weekday_token_and_rest("every monday").unwrap();
        assert_eq!(wd, Weekday::Mon);

        let base = NaiveDate::from_ymd_opt(2026, 5, 11).unwrap();
        assert_eq!(base.weekday(), Weekday::Mon);
        assert_eq!(
            next_weekday_date_from(base, Weekday::Mon, true),
            NaiveDate::from_ymd_opt(2026, 5, 11).unwrap()
        );
        assert_eq!(
            next_weekday_date_from(base, Weekday::Tue, true),
            NaiveDate::from_ymd_opt(2026, 5, 12).unwrap()
        );
    }

    #[test]
    fn weekday_every_flag() {
        let (wd, _, every) = parse_weekday_token_and_rest_with_every("每周二 9:00").unwrap();
        assert_eq!(wd, Weekday::Tue);
        assert!(every);

        let (wd, _, every) = parse_weekday_token_and_rest_with_every("周二 9:00").unwrap();
        assert_eq!(wd, Weekday::Tue);
        assert!(!every);
    }
}
