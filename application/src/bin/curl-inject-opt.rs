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

use curl_inject_opt_shared::{config, serialize_options};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use yansi::Paint;

fn main() {
	if !curl_inject_opt::should_color(2) {
		Paint::disable();
	}

	let args       = curl_inject_opt::build_cli().get_matches();
	let print_env  = args.get_flag("print-env");
	let debug      = args.get_flag("debug");
	let no_inherit = args.get_flag("no-inherit");

	let preload_lib = match config::rely_on_search() {
		true  => PathBuf::from("libcurl_inject_opt_preload.so"),
		false => config::libdir().join("libcurl_inject_opt_preload.so"),
	};

	// Collect CURL options to set.
	let set_options = match curl_inject_opt::extract_curl_options(&args) {
		Ok(x)  => x,
		Err(e) => {
			eprintln!("{} {}", Paint::red("Error:").bold(), e);
			std::process::exit(1);
		}
	};

	// Serialize CURL options for passing through the environment.
	let serialized_options = serialize_options(set_options.iter());

	if print_env {
		if debug {
			println!("CURL_INJECT_OPT_DEBUG=1");
		}
		if no_inherit {
			println!("CURL_INJECT_OPT_NO_INHERIT={}", preload_lib.display());
		}
		println!("CURL_INJECT_OPT={}", String::from_utf8_lossy(&serialized_options));
		return;
	}

	use std::os::unix::process::CommandExt;

	let command : Vec<&String> = args.get_many("COMMAND").unwrap().collect();
	let mut child = std::process::Command::new(command[0]);
	let mut child = child.args(&command[1..]);

	if let Some(old_preload) = std::env::var_os("LD_PRELOAD") {
		let new_preload = match std::env::join_paths(std::iter::once(preload_lib.clone()).chain(std::env::split_paths(&old_preload))) {
			Ok(x) => x,
			Err(_) => {
				eprintln!("{}: preload library path contains separator: {}", Paint::red("Error:").bold(), preload_lib.display());
				std::process::exit(1);
			}
		};
		child = child.env("LD_PRELOAD", new_preload);
	} else {
		child = child.env("LD_PRELOAD", preload_lib.as_os_str());
	}

	if debug {
		child = child.env("CURL_INJECT_OPT_DEBUG", "1");
	}

	if no_inherit {
		child = child.env("CURL_INJECT_OPT_NO_INHERIT", preload_lib);
	}

	child.env("CURL_INJECT_OPT", std::ffi::OsStr::from_bytes(&serialized_options));

	let error = child.exec();
	eprintln!("{} failed to execute command: {}", Paint::red("Error:").bold(), error);
	std::process::exit(2);
}
