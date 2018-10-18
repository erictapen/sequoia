# Configuration.
CARGO_TARGET_DIR	?= $(shell pwd)/../target
# We currently only support absolute paths.
CARGO_TARGET_DIR	:= $(abspath $(CARGO_TARGET_DIR))
SQV	?= $(CARGO_TARGET_DIR)/debug/sqv

# Tools.
CARGO	?= cargo
INSTALL		?= install

all: src/sqv-usage.rs

# Installation.
.PHONY: build-release
build-release:
	CARGO_TARGET_DIR=$(CARGO_TARGET_DIR) \
	    $(CARGO) build $(CARGO_FLAGS) --release --package sequoia-sqv

.PHONY: install
install: build-release
	$(INSTALL) -d $(DESTDIR)$(PREFIX)/bin
	$(INSTALL) -t $(DESTDIR)$(PREFIX)/bin $(CARGO_TARGET_DIR)/release/sqv

# Maintenance.
.PHONY: update-usage
update-usage: src/sqv-usage.rs

src/sqv-usage.rs: make-usage.sh $(SQV)
	sh make-usage.sh $(SQV) >$@