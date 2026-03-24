#!/usr/bin/env python3
"""Jira timesheet CLI — shows your worklogs grouped by week."""

import json
import subprocess
import sys
import webbrowser
import warnings
from collections import defaultdict
from datetime import datetime, timedelta
from pathlib import Path

warnings.filterwarnings("ignore", category=Warning, module="urllib3")

REQUIRED = ["requests", "rich", "keyring"]


def _ensure_deps():
    missing = []
    for pkg in REQUIRED:
        try:
            __import__(pkg)
        except ImportError:
            missing.append(pkg)
    if missing:
        print(f"Installing missing dependencies: {', '.join(missing)}")
        subprocess.check_call([sys.executable, "-m", "pip", "install", *missing])
        print()


_ensure_deps()

import keyring
import requests
from rich import box
from rich.console import Console
from rich.padding import Padding
from rich.prompt import Prompt
from rich.rule import Rule
from rich.table import Table
from rich.text import Text

CONFIG_PATH = Path.home() / ".config" / "jelper" / "config.json"
KEYRING_SERVICE = "jelper"
KEYRING_KEY = "api_token"

JIRA_URL = ""
JIRA_EMAIL = ""
JIRA_TOKEN = ""


def load_config():
    global JIRA_URL, JIRA_EMAIL, JIRA_TOKEN
    if CONFIG_PATH.exists():
        data = json.loads(CONFIG_PATH.read_text())
        JIRA_URL = data.get("jira_url", "")
        JIRA_EMAIL = data.get("jira_email", "")
    JIRA_TOKEN = keyring.get_password(KEYRING_SERVICE, KEYRING_KEY) or ""


def save_config(url, email, token):
    CONFIG_PATH.parent.mkdir(parents=True, exist_ok=True)
    CONFIG_PATH.write_text(json.dumps({"jira_url": url, "jira_email": email}, indent=2))
    CONFIG_PATH.chmod(0o600)
    keyring.set_password(KEYRING_SERVICE, KEYRING_KEY, token)


STATUS_STYLES = {
    "In Progress": ("yellow", "⏳"),
    "Review": ("blue", "👁 "),
    "Done": ("green", "✓ "),
    "To Do": ("white", "○ "),
    "Selected for Development": ("cyan", "🎯"),
}

console = Console()


def setup():
    global JIRA_URL, JIRA_EMAIL, JIRA_TOKEN
    console.print("\n[bold]Jelper setup[/bold] — configure your Jira credentials\n")

    url = Prompt.ask("Jira URL", default=JIRA_URL or "https://your-org.atlassian.net")
    email = Prompt.ask("Jira email", default=JIRA_EMAIL or "")
    console.print("\nOpening browser to create an API token...")
    webbrowser.open("https://id.atlassian.com/manage-profile/security/api-tokens")
    token = Prompt.ask("Paste your API token", password=True)

    console.print("\nVerifying credentials...", end=" ")
    try:
        r = requests.get(
            f"{url}/rest/api/3/myself",
            auth=(email, token),
            headers={"Accept": "application/json"},
            timeout=15,
        )
        r.raise_for_status()
        name = r.json().get("displayName", email)
        console.print(f"[green]OK[/green] — logged in as [bold]{name}[/bold]\n")
    except requests.HTTPError as e:
        console.print(f"[red]Failed[/red] ({e})\n")
        sys.exit(1)
    except requests.RequestException as e:
        console.print(f"[red]Connection error[/red] ({e})\n")
        sys.exit(1)

    save_config(url, email, token)
    JIRA_URL, JIRA_EMAIL, JIRA_TOKEN = url, email, token
    console.print("[green]Configuration saved.[/green]\n")


def jira_get(path, params=None):
    if not JIRA_EMAIL or not JIRA_TOKEN:
        console.print("[red]Run jelper --reconfigure to set up credentials.[/red]")
        sys.exit(1)
    r = requests.get(
        f"{JIRA_URL}/rest/api/3{path}",
        params=params,
        auth=(JIRA_EMAIL, JIRA_TOKEN),
        headers={"Accept": "application/json"},
        timeout=15,
    )
    r.raise_for_status()
    return r.json()


def get_current_user():
    return jira_get("/myself")


def get_issues(account_id):
    jql = f'assignee = "{account_id}" ORDER BY updated DESC'
    fields = "summary,status,timespent,worklog,project"
    issues = []
    params = {"jql": jql, "fields": fields, "maxResults": 100}
    while True:
        data = jira_get("/search/jql", params)
        issues.extend(data["issues"])
        if data.get("isLast", True):
            break
        params = {**params, "nextPageToken": data["nextPageToken"]}
    return issues


def extract_adf_text(adf):
    if not adf or not isinstance(adf, dict):
        return ""
    texts = []
    for node in adf.get("content", []):
        if node.get("type") == "text":
            texts.append(node.get("text", ""))
        else:
            child = extract_adf_text(node)
            if child:
                texts.append(child)
    return " ".join(texts).strip()


def extract_entries(issues, account_id):
    entries = []
    for issue in issues:
        key = issue["key"]
        fields = issue["fields"]
        summary = fields["summary"]
        status = fields["status"]["name"]
        worklogs = fields.get("worklog", {}).get("worklogs", [])
        # If total > 20, fetch all worklogs
        if fields.get("worklog", {}).get("total", 0) > 20:
            wl_data = jira_get(f"/issue/{key}/worklog")
            worklogs = wl_data["worklogs"]
        for wl in worklogs:
            if wl["author"]["accountId"] != account_id:
                continue
            started_raw = wl["started"][:10]
            seconds = wl["timeSpentSeconds"]
            description = extract_adf_text(wl.get("comment", {}))
            entries.append(
                {
                    "key": key,
                    "summary": summary,
                    "status": status,
                    "seconds": seconds,
                    "started": started_raw,
                    "url": f"{JIRA_URL}/browse/{key}",
                    "description": description,
                }
            )
    return entries


def week_bounds(date_str):
    d = datetime.strptime(date_str, "%Y-%m-%d")
    monday = d - timedelta(days=d.weekday())
    sunday = monday + timedelta(days=6)
    return monday, sunday


def week_label(date_str):
    monday, sunday = week_bounds(date_str)
    return f"Week of {monday.strftime('%b %d')} – {sunday.strftime('%b %d, %Y')}"


def week_sort_key(date_str):
    d = datetime.strptime(date_str, "%Y-%m-%d")
    return d - timedelta(days=d.weekday())


def format_hours(seconds):
    h = seconds / 3600
    if h == int(h):
        return f"{int(h)}h"
    return f"{h:.1f}h"


def status_cell(status):
    style, icon = STATUS_STYLES.get(status, ("white", "  "))
    return Text(f"{icon} {status}", style=style)


def render(entries):
    if not entries:
        console.print("[yellow]No worklogs found.[/yellow]")
        return

    groups = defaultdict(list)
    for e in entries:
        groups[week_label(e["started"])].append(e)

    sorted_groups = sorted(
        groups.items(),
        key=lambda x: week_sort_key(x[1][0]["started"]),
        reverse=False,
    )

    total_seconds = 0
    today_str = datetime.now().strftime("%Y-%m-%d")

    for label, group_entries in sorted_groups:
        console.print()
        console.print(Rule(f"[bold]{label}[/bold]", style="dim"))

        days = defaultdict(list)
        for e in group_entries:
            days[e["started"]].append(e)

        group_seconds = 0
        for day_str in sorted(days.keys()):
            day_entries = days[day_str]
            d = datetime.strptime(day_str, "%Y-%m-%d")
            day_label = d.strftime("%A, %b %d")
            if day_str == today_str:
                day_label += "  [bold magenta](today)[/bold magenta]"

            table = Table(
                box=box.ROUNDED,
                show_lines=True,
                show_footer=False,
                pad_edge=False,
                expand=True,
            )
            table.add_column("Key", no_wrap=True, min_width=8)
            table.add_column("Summary", max_width=40)
            table.add_column("Description", max_width=40)
            table.add_column("Status", no_wrap=True, min_width=14)
            table.add_column("Time", justify="right", no_wrap=True, min_width=6)

            day_seconds = 0
            for e in sorted(day_entries, key=lambda x: x["started"]):
                key_text = Text(e["key"], style=f"bold cyan link {e['url']}")
                table.add_row(
                    key_text,
                    e["summary"],
                    e["description"] or "",
                    status_cell(e["status"]),
                    format_hours(e["seconds"]),
                )
                day_seconds += e["seconds"]
            group_seconds += day_seconds

            day_total_style = "bold green" if day_seconds >= 28800 else "dim"
            console.print(
                f"\n  [bold]{day_label}[/bold]  [{day_total_style}]{format_hours(day_seconds)}[/{day_total_style}]"
            )
            console.print(Padding(table, (0, 2)))

        week_style = "bold green" if group_seconds >= 144000 else "bold cyan"
        console.print(
            f"  [bold]Week total:[/bold] [{week_style}]{format_hours(group_seconds)}[/{week_style}]"
        )
        total_seconds += group_seconds

    console.print()
    console.print(Rule(style="dim"))
    month = datetime.now().strftime("%B")
    console.print(
        f"\n  [bold]Grand total for {month}:[/bold] [bold green]{format_hours(total_seconds)}[/bold green] across [bold]{len(entries)}[/bold] worklog entries\n"
    )


def main():
    reconfigure = "--reconfigure" in sys.argv
    load_config()

    if reconfigure or not (JIRA_URL and JIRA_EMAIL and JIRA_TOKEN):
        setup()

    with console.status("Fetching from Jira..."):
        user = get_current_user()
        account_id = user["accountId"]
        issues = get_issues(account_id)
        entries = extract_entries(issues, account_id)

    render(entries)


if __name__ == "__main__":
    main()
