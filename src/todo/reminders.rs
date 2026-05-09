use std::io::Write;

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};

use super::{TodoState, notifications};

pub fn parse_local_datetime_to_utc(input: &str) -> Option<DateTime<Utc>> {
    let naive = NaiveDateTime::parse_from_str(input.trim(), "%Y-%m-%d %H:%M").ok()?;
    let local = Local.from_local_datetime(&naive).single()?;
    Some(local.with_timezone(&Utc))
}

pub fn parse_local_date_to_utc_end_of_day(input: &str) -> Option<DateTime<Utc>> {
    let date = NaiveDate::parse_from_str(input.trim(), "%Y-%m-%d").ok()?;
    let naive = date.and_hms_opt(23, 59, 0)?;
    let local = Local.from_local_datetime(&naive).single()?;
    Some(local.with_timezone(&Utc))
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
        let Some(at) = item.reminder_at else {
            continue;
        };
        if item.reminder_sent {
            continue;
        }
        if at > now {
            continue;
        }

        let title = "Todo 提醒 (Todo Reminder)";
        let body = item.title.as_str();
        notifications::send_system_notification(title, body);
        item.reminder_sent = true;
        changed = true;
    }

    changed
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
