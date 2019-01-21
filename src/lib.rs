use std::ffi::CStr;
use std::os::unix::ffi::OsStrExt;

use curl_sys::{CURL, CURLcode, CURLoption};

macro_rules! load_next_fn {
	( $name:ident ( $($arg:ident : $type:ty),*$(,)? ) $( -> $ret:ty )? ) => {{
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
				let symbol : extern "C" fn($($arg : $type),*) $( -> $ret )? = std::mem::transmute(symbol);
				Ok(symbol)
			}
		}
	}};
}

struct CurlInjectOpt {
	next_curl_easy_perform    : extern "C" fn(handle: *mut CURL) -> CURLcode,
	next_curl_easy_setopt_str : extern "C" fn(handle: *mut CURL, option: CURLoption, value: *const std::ffi::CStr) -> CURLcode,
	debug                     : bool,
	cert_path                 : Option<std::ffi::CString>,
	key_path                  : Option<std::ffi::CString>,
}

fn env_bool(name: &str) -> bool {
	if let Some(val) = std::env::var_os(name) {
		val != "0" && val != "false" && val != "off"
	} else {
		false
	}
}

fn get_env_nul(name: impl AsRef<std::ffi::OsStr>) -> Option<std::ffi::CString> {
	Some(std::ffi::CString::new(std::env::var_os(name)?.as_bytes()).unwrap())
}

impl CurlInjectOpt {
	fn init() -> Result<Self, String> {
		let next_curl_easy_perform    = load_next_fn!(curl_easy_perform(handle: *mut CURL) -> CURLcode);
		let next_curl_easy_setopt_str = load_next_fn!(curl_easy_setopt(handle: *mut CURL, option: CURLoption, value: *const std::ffi::CStr) -> CURLcode);
		let debug                     = env_bool("CURL_INJECT_OPT_DEBUG");
		let cert_path                 = get_env_nul("CURL_INJECT_OPT_SSLCERT");
		let key_path                  = get_env_nul("CURL_INJECT_OPT_SSLKEY");

		if debug {
			eprintln!("curl-inject-opt: debug is on");
			if let Some(err) = next_curl_easy_perform.as_ref().err() {
				eprintln!("curl-inject-opt: {}", err);
			}
			if let Some(err) = next_curl_easy_setopt_str.as_ref().err() {
				eprintln!("curl-inject-opt: {}", err);
			}
		}

		let result = Self {
			next_curl_easy_perform:    next_curl_easy_perform?,
			next_curl_easy_setopt_str: next_curl_easy_setopt_str?,
			cert_path,
			key_path,
			debug,
		};

		Ok(result)
	}

	fn set_easy_str_opt(&self, handle: *mut CURL, opt: curl_sys::CURLoption, value: &std::ffi::CString) -> CURLcode {
		if self.debug {
			eprintln!("curl-inject-opt: setting option {}: {:?}", opt, value.as_bytes_with_nul());
		}
		let code = (self.next_curl_easy_setopt_str)(handle, opt, value.as_c_str());
		if code != curl_sys::CURLE_OK {
			eprintln!("curl-inject-opt: failed to set option {}: error {}", opt, code);
		}
		code
	}

	fn set_easy_options(&self, handle: *mut CURL) {
		// Set client cert path if requested.
		if let Some(value) = &self.cert_path {
			self.set_easy_str_opt(handle, curl_sys::CURLOPT_SSLCERT, value);
		}

		// Set client key path if requested.
		if let Some(value) = &self.key_path {
			self.set_easy_str_opt(handle, curl_sys::CURLOPT_SSLKEY, value);
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

	init.set_easy_options(handle);

	// Delegate to the real handler.
	(init.next_curl_easy_perform)(handle)
}
