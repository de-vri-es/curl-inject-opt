// The build script generates $OUT_DIR/config.rs with values taken from ../config.cache

use std::path::{Path, PathBuf};
use std::io::Write;

const RAW_CONFIG : &str = include_str!("../config.cache");

pub struct Config {
	pub raw_prefix: &'static str,
	pub raw_libdir: &'static str,
	pub raw_bindir: &'static str,
	pub rely_on_search: bool,
}

impl Config {
	pub fn prefix(&self) -> &'static Path {
		Path::new(self.raw_prefix)
	}

	pub fn libdir(&self) -> PathBuf {
		self.prefix().join(self.raw_libdir)
	}

	pub fn bindir(&self) -> PathBuf {
		self.prefix().join(self.raw_bindir)
	}

	pub fn rely_on_search(&self) -> bool {
		self.rely_on_search
	}
}

fn bool_value(value: &str) -> Result<bool, String> {
	if value == "0" || value.eq_ignore_ascii_case("false") || value.eq_ignore_ascii_case("no")  || value.eq_ignore_ascii_case("off") {
		Ok(false)
	} else if value == "1" || value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("yes")  || value.eq_ignore_ascii_case("on") {
		Ok(true)
	} else {
		Err(String::from("invalid boolean value"))
	}
}

pub fn parse_config() -> Result<Config, String> {
	let mut raw_prefix = "/usr/local";
	let mut raw_libdir = "lib";
	let mut raw_bindir = "bin";
	let mut rely_on_search = false;

	for (i, line) in RAW_CONFIG.lines().enumerate() {
		let line = line.trim();
		if line.starts_with("#") {
			continue;
		}

		let split_at = match line.find("=") {
			Some(x) => x,
			None    => return Err(format!("invalid config value on line {}, expected PARAM=value", i)),
		};

		let key   = (&line[..split_at]).trim();
		let value = (&line[split_at + 1..]).trim();

		match key {
			"PREFIX" => raw_prefix = value,
			"LIBDIR" => raw_libdir = value,
			"BINDIR" => raw_bindir = value,
			"RELY_ON_SEARCH" => rely_on_search = bool_value(value).map_err(|e| format!("invalid value for {}: {}", key, e))?,
			_ => return Err(format!("unknown parameter on line {}: {}", i, key))
		}
	}

	Ok(Config {
		raw_prefix,
		raw_libdir,
		raw_bindir,
		rely_on_search
	})
}

fn main() {
	let config  = parse_config().expect("failed to parse config.cache");
	let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("failed to get OUT_DIR from environment"));
	let mut config_out = std::fs::File::create(out_dir.join("config.rs")).expect(&format!("failed to open {} for writing", out_dir.join("config.rs").display()));

	writeln!(config_out, "pub const PREFIX : &str = {:#?};", config.raw_prefix).unwrap();
	writeln!(config_out, "pub const LIBDIR : &str = {:#?};", config.raw_libdir).unwrap();
	writeln!(config_out, "pub const BINDIR : &str = {:#?};", config.raw_bindir).unwrap();
	writeln!(config_out, "pub const LIBDIR_RESOLVED : &str = {:#?};", config.libdir()).unwrap();
	writeln!(config_out, "pub const BINDIR_RESOLVED : &str = {:#?};", config.bindir()).unwrap();
	writeln!(config_out, "pub const RELY_ON_SEARCH : bool = {:#?};",  config.rely_on_search()).unwrap();

	eprintln!("OUT_DIR: {}", out_dir.display());
}
