.PHONY: patch minor

patch:
	python3 scripts/release.py patch

minor:
	python3 scripts/release.py minor
