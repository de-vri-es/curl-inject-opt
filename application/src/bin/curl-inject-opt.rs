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

fn main() {
	let args   = curl_inject_opt::build_cli().get_matches();
	let debug  = args.is_present("debug");

	// Collect CURL options to set.
	let set_options = match curl_inject_opt::extract_curl_options(&args) {
		Ok(x)  => x,
		Err(e) => {
			eprintln!("{}", e);
			std::process::exit(1);
		}
	};

	// Serialize CURL options for passing through the environment.
	let serialized_options = serialize_options(set_options.iter());

	if args.is_present("print-env") {
		if debug {
			println!("CURL_INJECT_OPT_DEBUG=1");
		}
		println!("CURL_INJECT_OPT={}", String::from_utf8_lossy(&serialized_options));
		return;
	}


	use std::os::unix::process::CommandExt;

	let command : Vec<&str> = args.values_of("COMMAND").unwrap().collect();
	let mut child = std::process::Command::new(&command[0]);
	let mut child = child.args(&command[1..]);

	let preload_lib = match config::rely_on_search() {
		true  => PathBuf::from("libcurl_inject_opt_preload.so"),
		false => config::libdir().join("libcurl_inject_opt_preload.so"),
	};

	if let Some(old_preload) = std::env::var_os("LD_PRELOAD") {
		let mut preloads = std::ffi::OsString::with_capacity(preload_lib.as_os_str().len() + old_preload.len() + 1);
		preloads.push(preload_lib.as_os_str());
		preloads.push(":");
		preloads.push(old_preload);
		child = child.env("LD_PRELOAD", preloads);
	} else {
		child = child.env("LD_PRELOAD", preload_lib.as_os_str());
	}


	child.env("CURL_INJECT_OPT", std::ffi::OsStr::from_bytes(&serialized_options));

	if debug {
		child = child.env("CURL_INJECT_OPT_DEBUG", "1");
	}

	let error = child.exec();
	eprintln!("Failed to execute command: {}", error);
	std::process::exit(-1);
}
