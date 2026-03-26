import os

version = os.environ["VERSION"]
content = open("jelper.py", encoding="utf-8").read().replace("__JELPER_VERSION__", version)
open("jelper.py", "w", encoding="utf-8").write(content)
