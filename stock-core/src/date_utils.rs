use chrono::{Datelike, NaiveDate, Timelike};

pub fn roc_to_ad_date(value: &str) -> Option<String> {
    let value = value.trim();
    if let Ok(dt) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return Some(dt.format("%Y-%m-%d").to_string());
    }
    let parts: Vec<&str> = value.split('/').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;
    let ad_year = if year < 1911 { year + 1911 } else { year };
    let dt = NaiveDate::from_ymd_opt(ad_year, month, day)?;
    Some(dt.format("%Y-%m-%d").to_string())
}

pub fn roc_date_compact_to_ad(value: &str) -> Option<String> {
    let value = value.trim();
    if value.len() < 7 {
        return None;
    }
    let year: i32 = value[..3].parse().ok()?;
    let month: u32 = value[3..5].parse().ok()?;
    let day: u32 = value[5..7].parse().ok()?;
    let ad_year = year + 1911;
    let dt = NaiveDate::from_ymd_opt(ad_year, month, day)?;
    Some(dt.format("%Y-%m-%d").to_string())
}

pub fn is_trading_time() -> bool {
    let now = chrono::Local::now();
    let weekday = now.weekday();
    if weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun {
        return false;
    }
    let hour = now.hour();
    let minute = now.minute();
    let total_minutes = hour as i32 * 60 + minute as i32;
    (9 * 60..=13 * 60 + 30).contains(&total_minutes)
}
