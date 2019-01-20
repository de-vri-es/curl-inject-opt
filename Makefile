PREFIX  ?= /usr/local
DESTDIR ?= /
LIBDIR  ?= lib
BINDIR  ?= bin

.PHONY: all install

all:
	PREFIX="${PREFIX}" LIBDIR="${LIBDIR}" BINDIR="${BINDIR}" cargo build --release

install: all
	@./target/release/curl-inject-opt-install --destdir "${DESTDIR}"
