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

use curl_inject_opt_shared::SetOption;
use curl_inject_opt_shared::Value;
use curl_inject_opt_shared::parse_options;
use curl_inject_opt_shared::reexports::curl_sys;

use curl_sys::CURL;
use curl_sys::CURLM;
use curl_sys::CURLMcode;
use curl_sys::CURLcode;
use curl_sys::CURLoption;

type CurlEasySetOpt  = extern "C" fn(handle: *mut CURL, option: CURLoption, ...) -> CURLcode;
type CurlEasyPerform = extern "C" fn(handle: *mut CURL) -> CURLcode;
type CurlMultiAddHandle = extern "C" fn(multi_handle: *mut CURLM, handle: *mut CURL) -> CURLMcode;

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
	/// The original curl_easy_perform function.
	curl_easy_perform: CurlEasyPerform,

	/// The original curl_east_setopt function.
	curl_easy_setopt: CurlEasySetOpt,

	/// The original curl_multi_add_handle function.
	curl_multi_add_handle: CurlMultiAddHandle,

	/// The options to set on all handles.
	options: Vec<SetOption>,

	/// If true, run in debug mode, printing what we're doing.
	debug: bool,
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
		let curl_easy_perform     = load_next_fn!(curl_easy_perform     : CurlEasyPerform);
		let curl_easy_setopt      = load_next_fn!(curl_easy_setopt      : CurlEasySetOpt);
		let curl_multi_add_handle = load_next_fn!(curl_multi_add_handle : CurlMultiAddHandle);
		let debug                 = env_bool("CURL_INJECT_OPT_DEBUG");
		let no_inherit            = std::env::var_os("CURL_INJECT_OPT_NO_INHERIT");
		let options               = std::env::var_os("CURL_INJECT_OPT");
		let options               = options.map(|x| parse_options(x.as_bytes()).expect("failed to parse CURL_INJECT_OPT")).unwrap_or_default();

		if let Some(path) = no_inherit {
			if let Some(preload) = std::env::var_os("LD_PRELOAD") {
				std::env::set_var("LD_PRELOAD", std::env::join_paths(std::env::split_paths(&preload).filter(|x| *x != path)).unwrap());
			}
		}

		if debug {
			eprintln!("curl-inject-opt: debug is on");
			if let Some(err) = curl_easy_perform.as_ref().err() {
				eprintln!("curl-inject-opt: {}", err);
			}
			if let Some(err) = curl_easy_setopt.as_ref().err() {
				eprintln!("curl-inject-opt: {}", err);
			}
			if let Some(err) = curl_multi_add_handle.as_ref().err() {
				eprintln!("curl-inject-opt: {}", err);
			}
		}

		let result = Self {
			curl_easy_perform: curl_easy_perform?,
			curl_easy_setopt: curl_easy_setopt?,
			curl_multi_add_handle: curl_multi_add_handle?,
			options,
			debug,
		};

		Ok(result)
	}

	fn set_option(&self, handle: *mut CURL, option: &SetOption) -> CURLcode {
		if self.debug {
			eprintln!("curl-inject-opt: setting option {}: {}", option.name, option.value);
		}
		let code = match &option.value {
			Value::CString(x) => (self.curl_easy_setopt)(handle, option.option, x.as_ref() as *const CStr),
			Value::CLong(x)   => (self.curl_easy_setopt)(handle, option.option, *x),
		};
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

#[no_mangle]
pub extern "C" fn curl_multi_add_handle(multi_handle: *mut CURLM, handle: *mut CURL) -> CURLMcode {
	let init = match unsafe { _INIT.as_ref().unwrap() } {
		Err(string) => panic!("{}", string),
		Ok(init)    => init,
	};

	if init.debug {
		eprintln!("curl-inject-opt: curl_multi_add_handle() called");
	}

	// Set options, then delegate to the real handler.
	init.set_options(handle);
	(init.curl_multi_add_handle)(multi_handle, handle)
}
