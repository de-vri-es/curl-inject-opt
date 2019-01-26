DESTDIR ?= /

.PHONY: all install

all:
	cargo build --release

install: all
	@./target/release/curl-inject-opt-install --destdir "${DESTDIR}"
