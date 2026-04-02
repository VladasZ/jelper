use crate::display::{make_table, print_rule, print_table_indented, right_cell, status_cell, terminal_width};
use crate::jira::Entry;
use crate::render::month_display_name;
use crate::time::format_hours;
use comfy_table::{Attribute, Cell, Color, ColumnConstraint, Width};
use std::collections::BTreeMap;

pub fn render(entries: &[Entry]) {
    if entries.is_empty() {
        println!("No worklogs found.");
        return;
    }

    let w = terminal_width();

    let mut by_month: BTreeMap<String, Vec<&Entry>> = BTreeMap::new();
    for e in entries {
        by_month.entry(e.started[..7].to_string()).or_default().push(e);
    }

    for (month_key, month_entries) in &by_month {
        let month_name = month_display_name(month_key);

        println!();
        print_rule(&month_name, true);

        let mut by_task: BTreeMap<String, Vec<&Entry>> = BTreeMap::new();
        for e in month_entries {
            by_task.entry(e.key.clone()).or_default().push(*e);
        }

        let mut task_list: Vec<(String, Vec<&Entry>)> = by_task.into_iter().collect();
        task_list.sort_by_key(|(_, es)| {
            es.iter().map(|e| e.started.clone()).min().unwrap_or_default()
        });

        let mut table = make_table(w);
        table.set_header(vec![
            Cell::new("Key"),
            Cell::new("Summary"),
            Cell::new("Status"),
            Cell::new("Started"),
            right_cell("Time"),
        ]);
        table
            .column_mut(0)
            .unwrap()
            .set_constraint(ColumnConstraint::LowerBoundary(Width::Fixed(8)));
        table
            .column_mut(1)
            .unwrap()
            .set_constraint(ColumnConstraint::UpperBoundary(Width::Fixed(40)));
        table
            .column_mut(3)
            .unwrap()
            .set_constraint(ColumnConstraint::LowerBoundary(Width::Fixed(10)));
        table
            .column_mut(4)
            .unwrap()
            .set_constraint(ColumnConstraint::LowerBoundary(Width::Fixed(6)));

        let mut month_seconds = 0i64;

        for (key, task_entries) in &task_list {
            let total = task_entries.iter().map(|e| e.seconds).sum::<i64>();
            let started = task_entries.iter().map(|e| e.started.as_str()).min().unwrap_or("");
            let sample = task_entries[0];

            table.add_row(vec![
                Cell::new(key.as_str()).fg(Color::Cyan).add_attribute(Attribute::Bold),
                Cell::new(&sample.summary),
                status_cell(&sample.status),
                Cell::new(started),
                right_cell(format_hours(total)),
            ]);
            month_seconds += total;
        }

        table.add_row(vec![
            Cell::new("Total").add_attribute(Attribute::Bold),
            Cell::new(format!("{} tasks", task_list.len())).fg(Color::DarkGrey),
            Cell::new(""),
            Cell::new(""),
            right_cell(format_hours(month_seconds))
                .fg(Color::Green)
                .add_attribute(Attribute::Bold),
        ]);

        println!();
        print_table_indented(&table);
    }
}
