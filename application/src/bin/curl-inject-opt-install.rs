use structopt::StructOpt;
use std::path::{Path, PathBuf};
use std::ffi::{OsString};

use curl_inject_opt_shared::{PREFIX, LIBDIR, BINDIR};
use ansi_term::Colour::{Red, Green};

#[derive(Debug, Clone, StructOpt)]
#[structopt(about = "Install the curl-inject-opt binary and preload library.", author = "")]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
struct Args {
	#[structopt(long = "destdir")]
	destdir: String,
}

fn concat_paths(a: impl AsRef<Path>, b: impl AsRef<Path>) -> PathBuf {
	let a = a.as_ref();
	let b = b.as_ref();

	if a == Path::new("/") {
		return PathBuf::from(b);
	}

	let mut result = OsString::from(a.as_os_str());
	if !b.is_absolute() {
		result.push("/");
	}
	result.push(b.as_os_str());

	PathBuf::from(result)
}

/// Similar to std::fs::copy, but first unlinks the destination and opens then opens it with O_EXCL to make sure it's a new inode.
fn install_file(mode: u32, source: impl AsRef<Path>, dest_dir: impl AsRef<Path>) -> Result<(), String> {
	let source   = source.as_ref();
	let dest_dir = dest_dir.as_ref();
	let dest     = dest_dir.join(source.file_name().unwrap());

	eprintln!("{} {}", Green.bold().paint("  Installing"), dest.display());

	std::fs::remove_file(&dest).or_else(|e| match e.kind() {
		std::io::ErrorKind::NotFound => Ok(()),
		_                            => Err(e),
	}).map_err(|e| format!("Failed to unlink destination file: {}: {}", dest.display(), e))?;


	use std::os::unix::fs::OpenOptionsExt;
	let mut source_file = std::fs::File::open(source).map_err(|e| format!("Failed to open file for reading: {}: {}", source.display(), e))?;
	let mut dest_file   = std::fs::OpenOptions::new()
		.read(false)
		.write(true)
		.create(true)
		.mode(mode)
		.custom_flags(libc::O_EXCL)
		.open(&dest)
		.map_err(|e| format!("Failed to open file for writing: {}: {}", dest.display(), e))?;

	std::io::copy(&mut source_file, &mut dest_file).map_err(|e| format!("Failed to copy data from {} to {}: {}", source.display(), dest.display(), e))?;

	Ok(())
}

fn install() -> Result<(), String> {
	let args = Args::from_args();

	let cwd     = std::env::current_dir().map_err(|e| format!("Failed to get current working directory: {}", e))?;
	let destdir = cwd.join(args.destdir);
	let bindir  = concat_paths(&destdir, BINDIR);
	let libdir  = concat_paths(&destdir, LIBDIR);

	std::fs::create_dir_all(&bindir).map_err(|e| format!("Failed to create directory: {}: {}", bindir.display(), e))?;
	std::fs::create_dir_all(&libdir).map_err(|e| format!("Failed to create directory: {}: {}", bindir.display(), e))?;

	install_file(0o755, "target/release/curl-inject-opt",               bindir)?;
	install_file(0o755, "target/release/libcurl_inject_opt_preload.so", libdir)?;

	Ok(())
}

fn main() {
	let error = match install() {
		Ok(())   => return,
		Err(err) => err,
	};

	eprintln!("{} {}", Red.bold().paint("Error:"), error);
	std::process::exit(1);
}
