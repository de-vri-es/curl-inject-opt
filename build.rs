use std::path::{Path, PathBuf};
use std::io::Write;
use std::fs::File;

fn write_config(file: &Path) -> std::io::Result<()> {
	let cwd    = std::env::current_dir().expect("failed to get current working directory");
	let prefix = cwd.join(std::env::var("PREFIX").expect("failed to get environment variable PREFIX"));
	let libdir = prefix.join(std::env::var("LIBDIR").expect("failed to get environment variable LIBDIR"));
	let bindir = prefix.join(std::env::var("BINDIR").expect("failed to get environment variable BINDIR"));

	let mut file = File::create(file)?;
	writeln!(file, "#[allow(unused)] const PREFIX : &str = {:#?};", prefix.display())?;
	writeln!(file, "#[allow(unused)] const LIBDIR : &str = {:#?};", libdir.display())?;
	writeln!(file, "#[allow(unused)] const BINDIR : &str = {:#?};", bindir.display())?;
	println!("cargo:rerun-if-env-changed=PREFIX");
	println!("cargo:rerun-if-env-changed=LIBDIR");
	println!("cargo:rerun-if-env-changed=BINDIR");
	Ok(())
}

fn main() {
	let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("failed to get environment variable OUT_DIR"));
	write_config(&out_dir.join("config.rs")).unwrap();
}
