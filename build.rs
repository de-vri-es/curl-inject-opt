use std::path::{Path, PathBuf};
use std::io::Write;
use std::fs::File;

fn write_config(file: &Path) -> std::io::Result<()> {
	let mut file = File::create(file)?;
	writeln!(file, "#[allow(unused)] const PREFIX : &str = {:#?};", std::env::var("PREFIX").expect("failed to get environment variable PREFIX"))?;
	writeln!(file, "#[allow(unused)] const LIBDIR : &str = {:#?};", std::env::var("LIBDIR").expect("failed to get environment variable LIBDIR"))?;
	writeln!(file, "#[allow(unused)] const BINDIR : &str = {:#?};", std::env::var("BINDIR").expect("failed to get environment variable BINDIR"))?;
	println!("cargo:rerun-if-env-changed=PREFIX");
	println!("cargo:rerun-if-env-changed=LIBDIR");
	println!("cargo:rerun-if-env-changed=BINDIR");
	Ok(())
}

fn main() {
	let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("failed to get environment variable OUT_DIR"));
	write_config(&out_dir.join("config.rs")).unwrap();
}
