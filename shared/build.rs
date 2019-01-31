// Copyright 2018-2019 Maarten de Vries <maarten@de-vri.es>
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this
//    list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

// The build script generates $OUT_DIR/config.rs with values taken from ../config.cache

use std::path::{Path, PathBuf};
use std::io::Write;

const RAW_CONFIG : &str = include_str!("../config.cache");

pub struct Config {
	pub raw_prefix: &'static str,
	pub raw_libdir: &'static str,
	pub raw_bindir: &'static str,
	pub raw_datadir: &'static str,
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

	pub fn datadir(&self) -> PathBuf {
		self.prefix().join(self.raw_datadir)
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
	let mut raw_prefix     = "/usr/local";
	let mut raw_libdir     = "lib";
	let mut raw_bindir     = "bin";
	let mut raw_datadir    = "share";
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
			"PREFIX"  => raw_prefix  = value,
			"LIBDIR"  => raw_libdir  = value,
			"BINDIR"  => raw_bindir  = value,
			"DATADIR" => raw_datadir = value,
			"RELY_ON_SEARCH" => rely_on_search = bool_value(value).map_err(|e| format!("invalid value for {}: {}", key, e))?,
			_ => return Err(format!("unknown parameter on line {}: {}", i, key))
		}
	}

	Ok(Config {
		raw_prefix,
		raw_libdir,
		raw_bindir,
		raw_datadir,
		rely_on_search
	})
}

fn main() {
	let config  = parse_config().expect("failed to parse config.cache");
	let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("failed to get OUT_DIR from environment"));
	let mut config_out = std::fs::File::create(out_dir.join("config.rs")).expect(&format!("failed to open {} for writing", out_dir.join("config.rs").display()));

	writeln!(config_out, "pub const PREFIX           : &str = {:#?};", config.raw_prefix).unwrap();
	writeln!(config_out, "pub const LIBDIR           : &str = {:#?};", config.raw_libdir).unwrap();
	writeln!(config_out, "pub const BINDIR           : &str = {:#?};", config.raw_bindir).unwrap();
	writeln!(config_out, "pub const DATADIR          : &str = {:#?};", config.raw_datadir).unwrap();
	writeln!(config_out, "pub const LIBDIR_RESOLVED  : &str = {:#?};", config.libdir()).unwrap();
	writeln!(config_out, "pub const BINDIR_RESOLVED  : &str = {:#?};", config.bindir()).unwrap();
	writeln!(config_out, "pub const DATADIR_RESOLVED : &str = {:#?};", config.datadir()).unwrap();
	writeln!(config_out, "pub const RELY_ON_SEARCH   : bool = {:#?};", config.rely_on_search()).unwrap();

	eprintln!("OUT_DIR: {}", out_dir.display());
}
