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

use std::ffi::CStr;
use std::os::unix::ffi::OsStrExt;

use curl_inject_opt_shared::{CURL, CURLcode, CurlEasySetOpt, CurlEasyPerform, SetOption, parse_options};

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
	options           : Vec<SetOption>,
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
		let options           = std::env::var_os("CURL_INJECT_OPT");
		let options           = options.map(|x| parse_options(x.as_bytes()).expect("failed to parse CURL_INJECT_OPT")).unwrap_or_default();
		let debug             = env_bool("CURL_INJECT_OPT_DEBUG");

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

	fn set_option(&self, handle: *mut CURL, option: &SetOption) -> CURLcode {
		if self.debug {
			eprintln!("curl-inject-opt: setting option {}: {}", option.name, option.value);
		}
		let code = option.set(self.curl_easy_setopt, handle);
		if code != curl_sys::CURLE_OK {
			eprintln!("curl-inject-opt: failed to set option {}: error {}", option.name, code);
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
