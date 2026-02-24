PREFIX := /usr
BINDIR := $(PREFIX)/bin
PKG_CONFIG := pkg-config
UDEV_DIR := $(shell $(PKG_CONFIG) udev --variable=udev_dir)
UDEV_RULES_DIR := $(UDEV_DIR)/rules.d

all: build

.PHONY: build clean test

target/release/cecd: build

target/release/cectool: build

build:
	@cargo build -r --target-dir target

clean:
	@cargo clean

test:
	@cargo test

install: target/release/cecd target/release/cectool
	install -d -m 755 "$(DESTDIR)$(UDEV_RULES_DIR)"
	install -d -m 755 "$(DESTDIR)$(BINDIR)"
	install -m 644 linux-cec/data/udev-rules.d/80-cec-uaccess.rules "$(DESTDIR)$(UDEV_RULES_DIR)"
	install -m 755 target/release/cecd "$(DESTDIR)$(BINDIR)/cecd"
	install -m 755 target/release/cectool "$(DESTDIR)$(BINDIR)/cectool"
