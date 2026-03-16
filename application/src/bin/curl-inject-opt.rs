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
