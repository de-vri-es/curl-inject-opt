use clap_complete::Shell;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use curl_inject_opt_shared::config;
use yansi::Paint;

#[derive(Debug, Clone, clap::Parser)]
#[clap(about = "Install the curl-inject-opt binary and preload library.")]
struct Args {
	/// Install files to a fake root (generally used for packaging).
	#[structopt(long)]
	destdir: String,

	/// Install bash completion.
	#[structopt(long)]
	bash: bool,

	/// Install zsh completion.
	#[structopt(long)]
	zsh: bool,

	/// Install fish completion.
	#[structopt(long)]
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

	eprintln!("{} {}",
		Paint::green("  Installing").bold(),
		dest.display()
	);

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
	let args: Args = clap::Parser::parse();

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
		eprintln!("{} {}",
			Paint::green("  Installing").bold(),
			bashdir.join("curl-inject-opt.bash").display()
		);
		clap_complete::generate_to(Shell::Bash, &mut cli, "curl-inject-opt", &bashdir)
			.map_err(|e| format!("Failed to write shell completion: {}.", e))?;
	}

	if args.zsh {
		make_dir(&zshdir)?;
		clap_complete::generate_to(Shell::Zsh, &mut cli, "curl-inject-opt", &zshdir)
			.map_err(|e| format!("Failed to write shell completion: {}.", e))?;
		eprintln!("{} {}",
			Paint::green("  Installing").bold(),
			zshdir.join("_curl-inject-opt").display()
		);
	}

	if args.fish {
		make_dir(&fishdir)?;
		clap_complete::generate_to(Shell::Fish, &mut cli, "curl-inject-opt", &fishdir)
			.map_err(|e| format!("Failed to write shell completion: {}.", e))?;

		eprintln!("{} {}",
			Paint::green("  Installing").bold(),
			fishdir.join("curl-inject-opt.fish").display()
		);
	}

	Ok(())
}

fn main() {
	if !curl_inject_opt::should_color(2) {
		Paint::disable();
	}

	if let Err(error) = install() {
		eprintln!("{} {}",
			Paint::red("Error:").bold(),
			error
		);
		std::process::exit(1);
	}
}
