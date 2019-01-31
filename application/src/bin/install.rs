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

use structopt::StructOpt;
use std::path::{Path, PathBuf};
use std::ffi::{OsString};

use curl_inject_opt_shared::config;
use ansi_term::Colour::{Red, Green};

#[derive(Debug, Clone, StructOpt)]
#[structopt(about = "Install the curl-inject-opt binary and preload library.", author = "")]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
struct Args {
	#[structopt(long = "destdir", help = "Install files to a fake root (generally used for packaging).")]
	destdir: String,

	#[structopt(long = "bash", help = "Install bash completion.")]
	bash: bool,

	#[structopt(long = "zsh", help = "Install zsh completion.")]
	zsh: bool,

	#[structopt(long = "fish", help = "Install fish completion.")]
	fish: bool,
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

fn make_dir(path: impl AsRef<Path>) -> Result<(), String> {
	let path = path.as_ref();
	std::fs::create_dir_all(path).map_err(|e| format!("Failed to create directory: {}: {}", path.display(), e))
}

fn install() -> Result<(), String> {
	let args = Args::from_args();

	let cwd     = std::env::current_dir().map_err(|e| format!("Failed to get current working directory: {}", e))?;
	let destdir = cwd.join(args.destdir);
	let libdir  = concat_paths(&destdir, config::libdir());
	let bindir  = concat_paths(&destdir, config::bindir());
	let datadir = concat_paths(&destdir, config::datadir());
	let bashdir = datadir.join("bash-completion/completions");
	let zshdir  = datadir.join("zsh/site-functions");
	let fishdir = datadir.join("fish/vendor_completions.d");

	let mut cli = curl_inject_opt::build_cli();

	make_dir(&libdir)?;
	make_dir(&bindir)?;

	install_file(0o755, "target/release/curl-inject-opt",               &bindir)?;
	install_file(0o755, "target/release/libcurl_inject_opt_preload.so", &libdir)?;

	if args.bash {
		make_dir(&bashdir)?;
		cli.gen_completions("curl-inject-opt", clap::Shell::Bash, &bashdir);
		eprintln!("{} {}", Green.bold().paint("  Installing"), bashdir.join("curl-inject-opt.bash").display());
	}

	if args.zsh {
		make_dir(&zshdir)?;
		cli.gen_completions("curl-inject-opt", clap::Shell::Zsh, &zshdir);
		eprintln!("{} {}", Green.bold().paint("  Installing"), zshdir.join("_curl-inject-opt").display());
	}

	if args.fish {
		make_dir(&fishdir)?;
		cli.gen_completions("curl-inject-opt", clap::Shell::Fish, &fishdir);
		eprintln!("{} {}", Green.bold().paint("  Installing"), fishdir.join("curl-inject-opt.fish").display());
	}

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
