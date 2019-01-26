use structopt::StructOpt;

use curl_inject_opt_shared::config::{PREFIX, LIBDIR};

#[derive(Debug, Clone, StructOpt)]
#[structopt(about = "Set curl options for a subcommand.", author = "")]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
#[structopt(raw(setting = "structopt::clap::AppSettings::TrailingVarArg"))]
struct Args {
	#[structopt(long = "debug")]
	debug: bool,

	#[structopt(long = "client-cert")]
	client_cert: Option<String>,

	#[structopt(long = "client-key")]
	client_key: Option<String>,

	#[structopt(name = "COMMAND", required = true)]
	command: String,

	#[structopt(name = "ARGS")]
	command_args: Vec<String>,
}

fn main() {
	let args = Args::from_args();

	use std::os::unix::process::CommandExt;

	let mut command = std::process::Command::new(args.command);
	let mut command = command.args(args.command_args);

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

	if args.debug {
		command = command.env("CURL_INJECT_OPT_DEBUG", "1");
	}

	if let Some(value) = args.client_cert {
		command = command.env("CURL_INJECT_OPT_SSLCERT", value);
	}

	if let Some(value) = args.client_key {
		command = command.env("CURL_INJECT_OPT_SSLKEY", value);
	}

	let error = command.exec();
	eprintln!("Failed to execute command: {}", error);
	std::process::exit(-1);
}
