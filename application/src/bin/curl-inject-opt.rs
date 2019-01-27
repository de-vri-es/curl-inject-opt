use curl_inject_opt_shared::{OPTIONS, SetOption, parse_config, serialize_options};

fn build_clap<'a, 'b>() -> clap::App<'a, 'b> {
	let mut app = clap::App::new("curl-inject-opt")
		.setting(clap::AppSettings::TrailingVarArg)
		.about("Inject options into CURL requests for a subcommand.")
		.arg(clap::Arg::with_name("debug")
			.long("--debug")
			.short("d")
			.help("Enable some debug printing in the preloaded library.")
		).arg(clap::Arg::with_name("COMMAND")
			.required(true)
			.multiple(true)
			.help("The command to run.")
		);

	for option in OPTIONS {
		app = app.arg(clap::Arg::with_name(option.name)
			.long(option.name)
			.takes_value(true)
			.help(option.help)
		);
	}

	app
}

fn main() {
	let args = build_clap().get_matches();

	let debug = args.is_present("debug");

	let command : Vec<&str> = args.values_of("COMMAND").unwrap().collect();

	let config = parse_config().expect("baked-in config does not parse");

	use std::os::unix::process::CommandExt;

	let mut child = std::process::Command::new(&command[0]);
	let mut child = child.args(&command[1..]);

	let preload_lib = config.libdir().join("libcurl_inject_opt_preload.so");

	if let Some(old_preload) = std::env::var_os("LD_PRELOAD") {
		let mut preloads = std::ffi::OsString::with_capacity(preload_lib.as_os_str().len() + old_preload.len() + 1);
		preloads.push(preload_lib.as_os_str());
		preloads.push(":");
		preloads.push(old_preload);
		child = child.env("LD_PRELOAD", preloads);
	} else {
		child = child.env("LD_PRELOAD", preload_lib.as_os_str());
	}

	let mut set_options = Vec::with_capacity(OPTIONS.len());

	for option in OPTIONS {
		if let Some(value) = args.value_of(option.name) {
			match SetOption::parse_value(*option, value) {
				Ok(x) => set_options.push(x),
				Err(e) => {
					eprintln!("{}", e);
					std::process::exit(1);
				}
			}
		}
	}

	let serialized = match serialize_options(set_options.iter()) {
		Ok(x) => x,
		Err(error) => {
			eprintln!("failed to serialize CURL options: {}", error);
			std::process::exit(1);
		}
	};

	child.env("CURL_INJECT_OPT", serialized);

	if debug {
		child = child.env("CURL_INJECT_OPT_DEBUG", "1");
	}

	let error = child.exec();
	eprintln!("Failed to execute command: {}", error);
	std::process::exit(-1);
}
