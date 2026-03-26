#!/usr/bin/env bash
set -euo pipefail

pyinstaller --onefile --name jelper-macos \
  --hidden-import keyring.backends.macOS \
  --hidden-import keyring.backends.fail \
  --hidden-import keyrings.alt \
  --icon jelper.ico \
  jelper.py
