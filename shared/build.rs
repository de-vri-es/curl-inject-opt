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

trait SliceExt<T>: Sized {
	fn trim_left(self, fun: impl FnMut(&T) -> bool) -> Self;

	fn trim_right(self, fun: impl FnMut(&T) -> bool) -> Self;

	fn trim(self, fun: impl Copy + FnMut(&T) -> bool) -> Self {
		self.trim_left(fun).trim_right(fun)
	}
}

impl<'a, T> SliceExt<T> for &'a [T] {
	fn trim_left(self, mut fun: impl FnMut(&T) -> bool) -> Self {
		if let Some(position) = self.iter().position(|b| !fun(b)) {
			&self[position..]
		} else {
			&self[0..0]
		}
	}

	fn trim_right(self, mut fun: impl FnMut(&T) -> bool) -> Self {
		if let Some(position) = self.iter().rposition(|b| !fun(b)) {
			&self[..=position]
		} else {
			&self[0..0]
		}
	}
}

pub struct Config {
	pub raw_prefix: String,
	pub raw_libdir: String,
	pub raw_bindir: String,
	pub raw_datadir: String,
	pub rely_on_search: bool,
}

impl Config {
	pub fn prefix(&self) -> &Path {
		&Path::new(&self.raw_prefix)
	}

	pub fn libdir(&self) -> PathBuf {
		self.prefix().join(&self.raw_libdir)
	}

	pub fn bindir(&self) -> PathBuf {
		self.prefix().join(&self.raw_bindir)
	}

	pub fn datadir(&self) -> PathBuf {
		self.prefix().join(&self.raw_datadir)
	}

	pub fn rely_on_search(&self) -> bool {
		self.rely_on_search
	}
}

fn bool_value(value: &[u8]) -> Result<bool, String> {
	if value == b"0" || value.eq_ignore_ascii_case(b"false") || value.eq_ignore_ascii_case(b"no")  || value.eq_ignore_ascii_case(b"off") {
		Ok(false)
	} else if value == b"1" || value.eq_ignore_ascii_case(b"true") || value.eq_ignore_ascii_case(b"yes")  || value.eq_ignore_ascii_case(b"on") {
		Ok(true)
	} else {
		Err(String::from("invalid boolean value"))
	}
}

fn read_file(path: impl AsRef<Path>) -> Result<Vec<u8>, String> {
	let path = path.as_ref();
	println!("cargo:rerun-if-changed={}", path.display());
	std::fs::read(path).map_err(|e| format!("failed to read file: {}: {}", path.display(), e))
}

fn get_env(name: &str) -> Result<String, String> {
	println!("cargo:rerun-if-env-changed={}", name);
	std::env::var(name).map_err(|e| format!("failed to get environment variable: {}: {}", name, e))
}

pub fn parse_config() -> Result<Config, String> {
	let mut raw_prefix  : &[u8] = b"/usr/local";
	let mut raw_libdir  : &[u8] = b"lib";
	let mut raw_bindir  : &[u8] = b"bin";
	let mut raw_datadir : &[u8] = b"share";
	let mut rely_on_search = false;

	let raw_config = read_file(get_env("CONFIG_CACHE")?)?;

	for (i, line) in raw_config.split(|b| *b == b'\n').enumerate() {
		let line = line.trim(|b| *b == b' ');
		if line.is_empty() || line.starts_with(b"#") {
			continue;
		}

		println!("{}: {}", i, String::from_utf8_lossy(line));


		let split_at = match line.iter().position(|b| *b == b'=') {
			Some(x) => x,
			None    => return Err(format!("invalid config value on line {}, expected PARAM=value, got {}", i, String::from_utf8_lossy(line))),
		};

		let key   = (&line[..split_at]).trim(|b| *b == b' ');
		let value = (&line[split_at + 1..]).trim(|b| *b == b' ');

		match key {
			b"PREFIX"  => raw_prefix  = value,
			b"LIBDIR"  => raw_libdir  = value,
			b"BINDIR"  => raw_bindir  = value,
			b"DATADIR" => raw_datadir = value,
			b"RELY_ON_SEARCH" => rely_on_search = bool_value(value).map_err(|_| format!("invalid boolean value for {}", std::str::from_utf8(key).unwrap()))?,
			_ => return Err(format!("unknown parameter on line {}: {}", i, String::from_utf8_lossy(key)))
		}
	}

	Ok(Config {
		raw_prefix:  String::from_utf8(raw_prefix.to_vec()).map_err(|_| format!("PREFIX contains invalid UTF-8"))?,
		raw_libdir:  String::from_utf8(raw_libdir.to_vec()).map_err(|_| format!("LIBDIR contains invalid UTF-8"))?,
		raw_bindir:  String::from_utf8(raw_bindir.to_vec()).map_err(|_| format!("BINDIR contains invalid UTF-8"))?,
		raw_datadir: String::from_utf8(raw_datadir.to_vec()).map_err(|_| format!("DATADIR contains invalid UTF-8"))?,
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
