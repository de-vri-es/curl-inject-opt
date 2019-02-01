DESTDIR ?= /

.PHONY: all clean install

PROJECT_ROOT = ${PWD}
BUILD_DIR    = ${PWD}
-include config.make

all:
	cd "${PROJECT_ROOT}" && CONFIG_CACHE="${BUILD_DIR}/config.cache" cargo build --target-dir="${BUILD_DIR}/target" --release

clean:
	rm -rf target

install: all
	@./target/release/install --destdir "${DESTDIR}" --bash --zsh --fish
