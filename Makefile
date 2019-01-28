DESTDIR ?= /

.PHONY: all install

all:
	cargo build --release

install: all
	@./target/release/install --destdir "${DESTDIR}" --bash --zsh --fish
