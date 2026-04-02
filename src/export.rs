use crate::jira::Entry;
use anyhow::Result;
use std::collections::BTreeMap;

pub fn to_toon(entries: &[Entry]) -> String {
    if entries.is_empty() {
        return "entries[0]:\n".to_string();
    }

    const FIELDS: &[&str] = &["key", "summary", "status", "seconds", "started", "url", "description"];
    const NEEDS_QUOTING: &[char] = &[',', ':', '|', '[', ']', '{', '}', '"', '\\', '\n', '\r', '\t'];

    let quote = |s: &str| -> String {
        if s.is_empty() || s != s.trim() || s.chars().any(|c| NEEDS_QUOTING.contains(&c)) {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t");
            format!("\"{escaped}\"")
        } else {
            s.to_string()
        }
    };

    let mut lines = vec![format!("entries[{}]{{{}}}:", entries.len(), FIELDS.join(","))];

    for e in entries {
        let values = [
            quote(&e.key),
            quote(&e.summary),
            quote(&e.status),
            quote(&e.seconds.to_string()),
            quote(&e.started),
            quote(&e.url),
            quote(&e.description),
        ];
        lines.push(format!("  {}", values.join(",")));
    }

    lines.join("\n") + "\n"
}

pub fn tasks_json(entries: &[Entry]) {
    let mut by_month: BTreeMap<String, Vec<&Entry>> = BTreeMap::new();
    for e in entries {
        by_month.entry(e.started[..7].to_string()).or_default().push(e);
    }

    let result: BTreeMap<&str, Vec<serde_json::Value>> = by_month
        .iter()
        .map(|(month_key, month_entries)| {
            let mut by_task: BTreeMap<String, Vec<&Entry>> = BTreeMap::new();
            for e in month_entries {
                by_task.entry(e.key.clone()).or_default().push(*e);
            }

            let mut task_list: Vec<_> = by_task.iter().collect();
            task_list.sort_by(|(_, a), (_, b)| {
                let min_a = a.iter().map(|e| e.started.as_str()).min().unwrap_or("");
                let min_b = b.iter().map(|e| e.started.as_str()).min().unwrap_or("");
                min_a.cmp(min_b)
            });

            let tasks: Vec<serde_json::Value> = task_list
                .into_iter()
                .map(|(key, es)| {
                    let started = es.iter().map(|e| e.started.as_str()).min().unwrap_or("");
                    let sample = es[0];
                    serde_json::json!({
                        "key": key,
                        "summary": sample.summary,
                        "status": sample.status,
                        "url": sample.url,
                        "started": started,
                        "seconds": es.iter().map(|e| e.seconds).sum::<i64>(),
                    })
                })
                .collect();

            (month_key.as_str(), tasks)
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}

pub fn to_xlsx(entries: &[Entry], path: &str) -> Result<()> {
    use rust_xlsxwriter::{Color, Format, FormatAlign, FormatUnderline, Url, Workbook};

    let mut workbook = Workbook::new();

    let title_format = Format::new().set_bold().set_font_size(14.0);
    let header_format = Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x1F4E79))
        .set_align(FormatAlign::Center);
    let total_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0xD9E1F2));
    let link_format = Format::new()
        .set_font_color(Color::RGB(0x0563C1))
        .set_underline(FormatUnderline::Single)
        .set_align(FormatAlign::Top);
    let wrap_format = Format::new().set_text_wrap().set_align(FormatAlign::Top);
    let top_format = Format::new().set_align(FormatAlign::Top);

    let mut by_month: BTreeMap<String, Vec<&Entry>> = BTreeMap::new();
    for e in entries {
        by_month.entry(e.started[..7].to_string()).or_default().push(e);
    }

    for (month_key, month_entries) in &by_month {
        use chrono::NaiveDate;
        let month_name = NaiveDate::parse_from_str(&format!("{month_key}-01"), "%Y-%m-%d")
            .map(|d| d.format("%B %Y").to_string())
            .unwrap_or_else(|_| month_key.clone());

        let ws = workbook.add_worksheet();
        ws.set_name(&month_name)?;

        ws.write_with_format(0, 0, format!("Timesheet \u{2014} {month_name}"), &title_format)?;

        for (col, header) in ["Task", "Summary", "Started", "Hours"].iter().enumerate() {
            ws.write_with_format(2, col as u16, *header, &header_format)?;
        }

        let mut by_task: BTreeMap<String, Vec<&Entry>> = BTreeMap::new();
        for e in month_entries {
            by_task.entry(e.key.clone()).or_default().push(*e);
        }

        let mut task_list: Vec<_> = by_task.iter().collect();
        task_list.sort_by(|(_, a), (_, b)| {
            let min_a = a.iter().map(|e| e.started.as_str()).min().unwrap_or("");
            let min_b = b.iter().map(|e| e.started.as_str()).min().unwrap_or("");
            min_a.cmp(min_b)
        });

        let mut row = 3u32;
        let mut month_seconds = 0i64;

        for (key, task_entries) in &task_list {
            let total = task_entries.iter().map(|e| e.seconds).sum::<i64>();
            let started = task_entries.iter().map(|e| e.started.as_str()).min().unwrap_or("");
            let sample = task_entries[0];
            let hours = (total as f64 / 3600.0 * 100.0).round() / 100.0;

            let url = Url::new(&sample.url).set_text(key.as_str());
            ws.write_url_with_format(row, 0, url, &link_format)?;
            ws.write_with_format(row, 1, &sample.summary, &wrap_format)?;
            ws.write_with_format(row, 2, started, &top_format)?;
            ws.write_with_format(row, 3, hours, &top_format)?;

            month_seconds += total;
            row += 1;
        }

        let total_hours = (month_seconds as f64 / 3600.0 * 100.0).round() / 100.0;
        ws.write_with_format(row, 0, "Total", &total_format)?;
        ws.write_with_format(row, 2, format!("{} tasks", task_list.len()), &total_format)?;
        ws.write_with_format(row, 3, total_hours, &total_format)?;

        ws.set_column_width(0, 12.0)?;
        ws.set_column_width(1, 55.0)?;
        ws.set_column_width(2, 12.0)?;
        ws.set_column_width(3, 8.0)?;
    }

    workbook.save(path)?;
    Ok(())
}
