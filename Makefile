LATEST_TAG := $(shell git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
VERSION := $(patsubst v%,%,$(LATEST_TAG))
MAJOR := $(word 1,$(subst ., ,$(VERSION)))
MINOR := $(word 2,$(subst ., ,$(VERSION)))
PATCH := $(word 3,$(subst ., ,$(VERSION)))

.PHONY: patch minor

patch:
	@bash -c 'NEW_TAG=v$(MAJOR).$(MINOR).$$(($(PATCH)+1)); git tag $$NEW_TAG && git push origin $$NEW_TAG && echo "Released $$NEW_TAG"'

minor:
	@bash -c 'NEW_TAG=v$(MAJOR).$$(($(MINOR)+1)).0; git tag $$NEW_TAG && git push origin $$NEW_TAG && echo "Released $$NEW_TAG"'
