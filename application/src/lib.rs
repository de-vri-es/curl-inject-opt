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

use curl_inject_opt_shared::{OPTIONS, SetOption};
use std::os::unix::ffi::OsStrExt;

pub fn build_cli() -> clap::Command {
	let mut app = clap::Command::new("curl-inject-opt")
		.about("Inject options into CURL requests for a subcommand.")
		.arg(clap::Arg::new("debug")
			.long("debug")
			.short('d')
			.action(clap::ArgAction::SetTrue)
			.help("Enable some debug printing in the preloaded library.")
		)
		.arg(clap::Arg::new("no-inherit")
			.long("no-inherit")
			.action(clap::ArgAction::SetTrue)
			.help("Do not inject options for child processes of the subcommand.")
		)
		.arg(clap::Arg::new("print-env")
			.long("print-env")
			.action(clap::ArgAction::SetTrue)
			.help("Print the environment variables and exit without running a command.")
		)
		.arg(clap::Arg::new("COMMAND")
			.required_unless_present("print-env")
			.action(clap::ArgAction::Append)
			.trailing_var_arg(true)
			.help("The command to run.")
		);

	for option in OPTIONS {
		app = app.arg(clap::Arg::new(option.name)
			.long(option.name)
			.value_name("VAL")
			.action(clap::ArgAction::Set)
			.number_of_values(1)
			.help(option.help)
		);
	}

	app
}

pub fn extract_curl_options(matches: &clap::ArgMatches) -> Result<Vec<SetOption>, String> {
	// Collect all occurences of curl options into a vector with the clap index, so we can sort on it.
	// Clap stores matches in a hash map, so we have no saner way to do this.

	let mut options : Vec<_> = OPTIONS.iter().filter_map(|option| {
		let values  = matches.get_raw(option.name)?;
		let indices = matches.indices_of(option.name).expect("clap match has values, but no indices");
		Some((option, values, indices))
	}).flat_map(|(option, values, indices)| {
		std::iter::repeat(option).zip(values).zip(indices)
	}).collect();

	// Sort by index on the command line.
	options.sort_unstable_by_key(|(_, index)| *index);

	// Parse the options.
	options.into_iter().map(|((option, value), _)| SetOption::parse_value(*option, value.as_bytes())).collect()
}

pub fn should_color(fd: i32) -> bool {
	if std::env::var_os("CLI_COLOR").map(|x| x == "0") == Some(true) {
		false
	} else if std::env::var_os("CLI_COLOR_FORCE").map(|x| x != "0") == Some(true) {
		true
	} else {
		unsafe { libc::isatty(fd) != 0 }
	}
}
