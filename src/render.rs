use crate::display::{make_table, print_rule, print_table_indented, right_cell, status_cell, terminal_width};
use crate::jira::Entry;
use crate::time::{format_hours, week_label, week_sort_key};
use chrono::{Local, NaiveDate};
use comfy_table::{Attribute, Cell, Color, ColumnConstraint, Width};
use console::style;
use std::collections::{BTreeMap, HashMap};

pub fn render(entries: &[Entry]) {
    if entries.is_empty() {
        println!("No worklogs found.");
        return;
    }

    let today = Local::now().date_naive();
    let today_str = today.format("%Y-%m-%d").to_string();
    let w = terminal_width();

    let mut by_month: BTreeMap<String, Vec<&Entry>> = BTreeMap::new();
    for e in entries {
        by_month.entry(e.started[..7].to_string()).or_default().push(e);
    }

    for (month_key, month_entries) in &by_month {
        let month_name = month_display_name(month_key);

        println!();
        print_rule(&month_name, true);

        let mut by_week: HashMap<String, Vec<&Entry>> = HashMap::new();
        for e in month_entries {
            by_week.entry(week_label(&e.started)).or_default().push(e);
        }

        let mut sorted_weeks: Vec<_> = by_week.iter().collect();
        sorted_weeks.sort_by_key(|(_, es)| week_sort_key(&es[0].started));

        let mut month_seconds = 0i64;

        for (label, group_entries) in &sorted_weeks {
            println!();
            print_rule(label, true);

            let mut by_day: BTreeMap<String, Vec<&Entry>> = BTreeMap::new();
            for e in *group_entries {
                by_day.entry(e.started.clone()).or_default().push(e);
            }

            let mut group_seconds = 0i64;

            for (day_str, day_entries) in &by_day {
                let d = NaiveDate::parse_from_str(day_str, "%Y-%m-%d").unwrap();
                let mut day_label = d.format("%A, %b %d").to_string();
                if day_str == &today_str {
                    day_label += "  (today)";
                }

                let mut table = make_table(w);
                table.set_header(vec![
                    Cell::new("Key"),
                    Cell::new("Summary"),
                    Cell::new("Description"),
                    Cell::new("Status"),
                    right_cell("Time"),
                ]);
                set_default_view_constraints(&mut table);

                let mut day_seconds = 0i64;
                for e in day_entries {
                    table.add_row(vec![
                        Cell::new(&e.key).fg(Color::Cyan).add_attribute(Attribute::Bold),
                        Cell::new(&e.summary),
                        Cell::new(&e.description),
                        status_cell(&e.status),
                        right_cell(format_hours(e.seconds)),
                    ]);
                    day_seconds += e.seconds;
                }
                group_seconds += day_seconds;

                let hours = format_hours(day_seconds);
                if day_seconds >= 28800 {
                    println!("\n  {}  {}", style(&day_label).bold(), style(&hours).bold().green());
                } else {
                    println!("\n  {}  {}", style(&day_label).bold(), style(&hours).dim());
                }
                print_table_indented(&table);
            }

            let week_hours = format_hours(group_seconds);
            if group_seconds >= 144000 {
                println!(
                    "  {}  {}",
                    style("Week total:").bold(),
                    style(&week_hours).bold().green()
                );
            } else {
                println!(
                    "  {}  {}",
                    style("Week total:").bold(),
                    style(&week_hours).bold().cyan()
                );
            }
            month_seconds += group_seconds;
        }

        println!();
        print_rule("", false);
        println!(
            "\n  {}  {} across {} worklog entries\n",
            style(format!("Total for {month_name}:")).bold(),
            style(format_hours(month_seconds)).bold().green(),
            style(month_entries.len().to_string()).bold(),
        );
    }
}

fn set_default_view_constraints(table: &mut comfy_table::Table) {
    table
        .column_mut(0)
        .unwrap()
        .set_constraint(ColumnConstraint::LowerBoundary(Width::Fixed(8)));
    table
        .column_mut(1)
        .unwrap()
        .set_constraint(ColumnConstraint::UpperBoundary(Width::Fixed(40)));
    table
        .column_mut(2)
        .unwrap()
        .set_constraint(ColumnConstraint::UpperBoundary(Width::Fixed(40)));
    table
        .column_mut(3)
        .unwrap()
        .set_constraint(ColumnConstraint::LowerBoundary(Width::Fixed(14)));
    table
        .column_mut(4)
        .unwrap()
        .set_constraint(ColumnConstraint::LowerBoundary(Width::Fixed(6)));
}

pub fn month_display_name(month_key: &str) -> String {
    NaiveDate::parse_from_str(&format!("{month_key}-01"), "%Y-%m-%d")
        .map(|d| d.format("%B %Y").to_string())
        .unwrap_or_else(|_| month_key.to_string())
}
