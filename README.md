<img src="jelper.png" width="80" align="left" style="margin-right: 16px"/>

# Jelper

A Jira timesheet CLI — shows your worklogs grouped by week and month.

## Installation

### Windows

1. Download `jelper-windows-installer.exe` from the [latest release](https://github.com/VladasZ/jelper/releases/latest)
2. Run the installer and follow the steps
3. Open a new terminal and type `jelper`

### macOS

```bash
brew tap VladasZ/tap
brew install jelper
```

## Usage

```
jelper               # show your timesheet (current + last month, grouped by week)
jelper --tasks       # show tasks grouped by total time per month
jelper --xlsx        # export last month's tasks to an Excel file (e.g. march-2026.xlsx)
jelper --json        # export worklogs as JSON
jelper --tasks --json  # export task summary as JSON
jelper --toon        # export worklogs as TOON
jelper --reconfigure # update saved credentials
```

### First launch

On first launch, Jelper will prompt for three things:

1. **Jira URL** — your organization's Jira address, e.g. `https://your-org.atlassian.net`
2. **Jira email** — the email you use to log into Jira
3. **API token** — your browser will open automatically to the Atlassian token page. Create a new token there, copy it, and paste it back into the terminal.

Jelper will verify the credentials before saving. On success, everything is stored locally and you won't be asked again.

The API token is stored in the OS keychain (Keychain on macOS, Credential Manager on Windows). The Jira URL and email are saved in plain text at `~/.config/jelper/config.json`.

### What it shows

Worklogs cover the current month and the previous month. The default view groups entries by month, then by week and day. Each entry shows the ticket key (clickable link), summary, worklog description, status, and time spent.

`--tasks` shows a summary table per month where each row is a unique task with its earliest start date and total hours logged.

`--xlsx` generates an Excel spreadsheet for last month with one sheet, a title, and a totals row.

## Credits

Icon by [Freepik](https://www.flaticon.com/authors/freepik) from [Flaticon](https://www.flaticon.com/free-icon/activity_11710486).
