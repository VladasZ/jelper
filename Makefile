LATEST_TAG := $(shell git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
VERSION := $(patsubst v%,%,$(LATEST_TAG))
MAJOR := $(word 1,$(subst ., ,$(VERSION)))
MINOR := $(word 2,$(subst ., ,$(VERSION)))
PATCH := $(word 3,$(subst ., ,$(VERSION)))

.PHONY: patch minor

patch:
	$(eval PATCH_NUM := $(shell python -c "print($(PATCH)+1)"))
	$(eval NEW_TAG := v$(MAJOR).$(MINOR).$(PATCH_NUM))
	git tag $(NEW_TAG)
	git push origin $(NEW_TAG)
	@echo "Released $(NEW_TAG)"

minor:
	$(eval MINOR_NUM := $(shell python -c "print($(MINOR)+1)"))
	$(eval NEW_TAG := v$(MAJOR).$(MINOR_NUM).0)
	git tag $(NEW_TAG)
	git push origin $(NEW_TAG)
	@echo "Released $(NEW_TAG)"
