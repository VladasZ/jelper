use anyhow::Result;
use chrono::{Datelike, Duration, Local};
use clap::Parser;

mod config;
mod display;
mod export;
mod jira;
mod render;
mod render_tasks;
#[allow(dead_code)]
mod time;

#[derive(Parser)]
#[command(name = "jelper", version, about = "Jira timesheet CLI")]
struct Args {
    #[arg(long)]
    reconfigure: bool,
    #[arg(long)]
    url: Option<String>,
    #[arg(long)]
    email: Option<String>,
    #[arg(long)]
    token: Option<String>,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    toon: bool,
    #[arg(long)]
    tasks: bool,
    #[arg(long)]
    xlsx: bool,
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut cfg = config::load(args.url.as_deref(), args.email.as_deref(), args.token.as_deref());

    if args.reconfigure || cfg.jira_url.is_empty() || cfg.jira_email.is_empty() || cfg.token.is_empty() {
        cfg = config::setup().await?;
    }

    let client = jira::Client::new(&cfg.jira_url, &cfg.jira_email, &cfg.token);

    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message("Fetching from Jira...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let user = client.get_current_user().await?;
    let all_entries = client.get_entries(&user.account_id).await?;

    spinner.finish_and_clear();

    let now = Local::now().date_naive();
    let first_of_month = now.with_day(1).unwrap();
    let first_of_prev = (first_of_month - Duration::days(1)).with_day(1).unwrap();
    let cutoff = first_of_prev.format("%Y-%m").to_string();

    let entries: Vec<_> = all_entries
        .into_iter()
        .filter(|e| &e.started[..7] >= cutoff.as_str())
        .collect();

    if args.xlsx {
        let last_month_dt = first_of_month - Duration::days(1);
        let last_month = last_month_dt.format("%Y-%m").to_string();
        let xlsx_entries: Vec<_> = entries
            .iter()
            .filter(|e| e.started.starts_with(&last_month))
            .cloned()
            .collect();
        let path = last_month_dt.format("%B-%Y").to_string().to_lowercase() + ".xlsx";
        export::to_xlsx(&xlsx_entries, &path)?;
        println!("Saved: {path}");
    } else if args.tasks && args.json {
        export::tasks_json(&entries);
    } else if args.json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else if args.toon {
        print!("{}", export::to_toon(&entries));
    } else if args.tasks {
        render_tasks::render(&entries);
    } else {
        render::render(&entries);
    }

    #[cfg(target_os = "windows")]
    {
        println!("\nPress Enter to exit...");
        let mut buf = String::new();
        let _ = std::io::stdin().read_line(&mut buf);
    }

    Ok(())
}
