use chrono::{Datelike, Duration, NaiveDate};

pub fn week_bounds(date_str: &str) -> (NaiveDate, NaiveDate) {
    let d = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
    let monday = d - Duration::days(d.weekday().num_days_from_monday() as i64);
    let sunday = monday + Duration::days(6);
    (monday, sunday)
}

pub fn week_label(date_str: &str) -> String {
    let (monday, sunday) = week_bounds(date_str);
    format!(
        "Week of {} \u{2013} {}",
        monday.format("%b %d"),
        sunday.format("%b %d, %Y")
    )
}

pub fn week_sort_key(date_str: &str) -> NaiveDate {
    week_bounds(date_str).0
}

pub fn format_hours(seconds: i64) -> String {
    let total_minutes = seconds / 60;
    if total_minutes % 60 == 0 {
        format!("{}h", total_minutes / 60)
    } else {
        format!("{:.1}h", seconds as f64 / 3600.0)
    }
}
