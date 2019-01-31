DESTDIR ?= /

.PHONY: all clean install

all:
	cargo build --release

clean:
	rm -rf target

install: all
	@./target/release/install --destdir "${DESTDIR}" --bash --zsh --fish
