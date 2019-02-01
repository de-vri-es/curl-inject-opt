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
			.multiple(true)
			.number_of_values(1)
			.help(option.help)
		);
	}

	app
}

pub fn extract_curl_options(matches: &clap::ArgMatches) -> Result<Vec<SetOption>, String> {
	// Collect all occurences of curl options into a vector with the clap index, so we can sort on it.
	// Clap stores matches in a hash map, so we have saner way to do this.

	let mut options : Vec<_> = OPTIONS.iter().filter_map(|option| {
		let values  = matches.values_of_os(option.name)?;
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
