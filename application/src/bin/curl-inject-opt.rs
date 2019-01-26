use structopt::StructOpt;

use curl_inject_opt_shared::config::{PREFIX, LIBDIR};
use curl_inject_opt_shared::{CurlOption, serialize_options};

#[derive(Debug, Clone, StructOpt)]
#[structopt(about = "Set curl options for a subcommand.", author = "")]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
#[structopt(raw(setting = "structopt::clap::AppSettings::TrailingVarArg"))]
struct Args {
	#[structopt(long = "debug")]
	debug: bool,

	#[structopt(short = "-o", long = "--option", number_of_values = 1, multiple = true)]
	options: Vec<CurlOption>,

	#[structopt(name = "COMMAND", required = true)]
	command: Vec<String>,
}

fn main() {
	let args = Args::from_args();

	use std::os::unix::process::CommandExt;

	let mut command = std::process::Command::new(&args.command[0]);
	let mut command = command.args(&args.command[1..]);

	let preload_lib = std::path::Path::new(PREFIX).join(LIBDIR).join("libcurl_inject_opt_preload.so");

	if let Some(old_preload) = std::env::var_os("LD_PRELOAD") {
		let mut preloads = std::ffi::OsString::with_capacity(preload_lib.as_os_str().len() + old_preload.len() + 1);
		preloads.push(preload_lib.as_os_str());
		preloads.push(":");
		preloads.push(old_preload);
		command = command.env("LD_PRELOAD", preloads);
	} else {
		command = command.env("LD_PRELOAD", preload_lib.as_os_str());
	}

	let serialized = match serialize_options(args.options.iter()) {
		Ok(x) => x,
		Err(error) => {
			eprintln!("failed to serialize CURL options: {}", error);
			std::process::exit(1);
		}
	};

	command.env("CURL_INJECT_OPT", serialized);

	if args.debug {
		command = command.env("CURL_INJECT_OPT_DEBUG", "1");
	}

	let error = command.exec();
	eprintln!("Failed to execute command: {}", error);
	std::process::exit(-1);
}
