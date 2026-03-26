pyinstaller --onefile --name jelper-windows `
  --hidden-import keyring.backends.Windows `
  --hidden-import keyring.backends.fail `
  --hidden-import keyrings.alt `
  --icon jelper.ico `
  jelper.py
