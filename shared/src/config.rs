use std::path::{Path, PathBuf};

const CONFIG : &str = include_str!("../../config.cache");

pub struct Config {
	pub raw_prefix: &'static str,
	pub raw_libdir: &'static str,
	pub raw_bindir: &'static str,
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
}

pub fn parse_config() -> Result<Config, String> {
	let mut raw_prefix = "/usr/local";
	let mut raw_libdir = "lib";
	let mut raw_bindir = "bin";

	for (i, line) in CONFIG.lines().enumerate() {
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
			_ => return Err(format!("unknown parameter on line {}: {}", i, key))
		}
	}

	Ok(Config {
		raw_prefix,
		raw_libdir,
		raw_bindir,
	})
}
