# Jelper

<img src="jelper.png" width="80" align="left" style="margin-right: 16px"/>

A Jira timesheet CLI — shows your worklogs grouped by week.

## Installation

### Windows

1. Download `jelper-installer.exe` from the [latest release](https://github.com/VladasZ/jelper/releases/latest)
2. Run the installer and follow the steps
3. Open a new terminal and type `jelper`

### macOS

1. Download `jelper-macos` from the [latest release](https://github.com/VladasZ/jelper/releases/latest)
2. Open a terminal in your Downloads folder and run:
   ```bash
   chmod +x jelper-macos
   xattr -d com.apple.quarantine jelper-macos
   sudo mv jelper-macos /usr/local/bin/jelper
   ```
3. Type `jelper` in any terminal

## Usage

```
jelper               # show your timesheet
jelper --reconfigure # update saved credentials
```

On first launch, Jelper will ask for your Jira URL, email, and API token.
Your browser will open automatically to create an API token.

## Credits

Icon by [Freepik](https://www.flaticon.com/authors/freepik) from [Flaticon](https://www.flaticon.com/free-icon/activity_11710486).
