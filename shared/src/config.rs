use std::path::Path;

mod raw {
	include!(concat!(env!("OUT_DIR"), "/config.rs"));
}

pub fn prefix() -> &'static Path {
	Path::new(raw::PREFIX)
}

pub fn libdir() -> &'static Path {
	Path::new(raw::LIBDIR_RESOLVED)
}

pub fn bindir() -> &'static Path {
	Path::new(raw::BINDIR_RESOLVED)
}

pub fn datadir() -> &'static Path {
	Path::new(raw::DATADIR_RESOLVED)
}

pub fn rely_on_search() -> bool {
	raw::RELY_ON_SEARCH
}

pub fn prefix_raw() -> &'static str {
	raw::PREFIX
}

pub fn libdir_raw() -> &'static str {
	raw::LIBDIR
}

pub fn bindir_raw() -> &'static str {
	raw::BINDIR
}

pub fn datadir_raw() -> &'static str {
	raw::DATADIR
}
