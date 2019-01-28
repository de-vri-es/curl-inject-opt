use curl_inject_opt_shared::OPTIONS;

pub fn build_cli<'a, 'b>() -> clap::App<'a, 'b> {
	let mut app = clap::App::new("curl-inject-opt")
		.setting(clap::AppSettings::TrailingVarArg)
		.setting(clap::AppSettings::DeriveDisplayOrder)
		.setting(clap::AppSettings::ColoredHelp)
		.about("Inject options into CURL requests for a subcommand.")
		.arg(clap::Arg::with_name("debug")
			.long("--debug")
			.short("d")
			.help("Enable some debug printing in the preloaded library.")
		)
		.arg(clap::Arg::with_name("print-env")
			.long("--print-env")
			.help("Print the environment variables and exit without running a command.")
		)
		.arg(clap::Arg::with_name("COMMAND")
			.required_unless("print-env")
			.multiple(true)
			.help("The command to run.")
		);

	for option in OPTIONS {
		app = app.arg(clap::Arg::with_name(option.name)
			.long(option.name)
			.takes_value(true)
			.value_name("VAL")
			.help(option.help)
		);
	}

	app
}
