use std::ffi::CStr;

use curl_inject_opt_shared::{CURL, CURLcode, CurlEasySetOpt, CurlEasyPerform, CurlOption, parse_options};

macro_rules! load_next_fn {
	( $name:ident : $type:ty ) => {{
		let name = unsafe { CStr::from_bytes_with_nul_unchecked(concat!(stringify!($name), "\0").as_bytes()) };
		unsafe {
			// Clear dlerror() before calling dlsym().
			libc::dlerror();

			// Look-up the wanted symbol.
			let symbol = libc::dlsym(libc::RTLD_NEXT, name.as_ptr());

			// Check dlerror().
			let error  = libc::dlerror();
			if !error.is_null() {
				// Convert to Err(String).
				Err(format!("failed to look up symbol {}: {}", stringify!($name), CStr::from_ptr(error).to_string_lossy()))
			} else {
				// Convert to Ok(fn).
				let symbol : $type = std::mem::transmute(symbol);
				Ok(symbol)
			}
		}
	}};
}

struct CurlInjectOpt {
	curl_easy_perform : CurlEasyPerform,
	curl_easy_setopt  : CurlEasySetOpt,
	options           : Vec<CurlOption>,
	debug             : bool,
}

fn env_bool(name: &str) -> bool {
	if let Some(val) = std::env::var_os(name) {
		val != "0" && val != "false" && val != "off"
	} else {
		false
	}
}

impl CurlInjectOpt {
	fn init() -> Result<Self, String> {
		let curl_easy_perform = load_next_fn!(curl_easy_perform : CurlEasyPerform);
		let curl_easy_setopt  = load_next_fn!(curl_easy_setopt  : CurlEasySetOpt);
		let options           = std::env::var("CURL_INJECT_OPT").expect("invalid UTF-8 in CURL_INJECT_OPT environment variable");
		let debug             = env_bool("CURL_INJECT_OPT_DEBUG");

		let options = if options.is_empty() {
			Vec::new()
		} else {
			parse_options(&options).expect("failed to parse CURL_INJECT_OPT")
		};

		if debug {
			eprintln!("curl-inject-opt: debug is on");
			if let Some(err) = curl_easy_perform.as_ref().err() {
				eprintln!("curl-inject-opt: {}", err);
			}
			if let Some(err) = curl_easy_setopt.as_ref().err() {
				eprintln!("curl-inject-opt: {}", err);
			}
		}

		let result = Self {
			curl_easy_perform: curl_easy_perform?,
			curl_easy_setopt:  curl_easy_setopt?,
			options,
			debug,
		};

		Ok(result)
	}

	fn set_option(&self, handle: *mut CURL, option: &CurlOption) -> CURLcode {
		if self.debug {
			eprintln!("curl-inject-opt: setting option {}: {}", option.key(), option.value());
		}
		let code = option.set(self.curl_easy_setopt, handle);
		if code != curl_sys::CURLE_OK {
			eprintln!("curl-inject-opt: failed to set option {}: error {}", option.key(), code);
		}
		code
	}

	fn set_options(&self, handle: *mut CURL) {
		for option in &self.options {
			self.set_option(handle, option);
		}
	}
}

static mut _INIT : Option<Result<CurlInjectOpt, String>> = None;

extern "C" fn initialize() {
	unsafe {
		_INIT = Some(CurlInjectOpt::init());
	}
}

#[used]
#[allow(non_upper_case_globals)]
#[cfg_attr(target_os = "linux", link_section = ".ctors")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
pub static init_curl_inject_opt: extern "C" fn() = {
	initialize
};

#[no_mangle]
pub extern "C" fn curl_easy_perform(handle: *mut CURL) -> CURLcode {
	let init = match unsafe { _INIT.as_ref().unwrap() } {
		Err(string) => panic!("{}", string),
		Ok(init)    => init,
	};

	if init.debug {
		eprintln!("curl-inject-opt: curl_easy_perform() called");
	}

	// Set options, then delegate to the real handler.
	init.set_options(handle);
	(init.curl_easy_perform)(handle)
}
