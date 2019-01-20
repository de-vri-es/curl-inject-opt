use std::ffi::CStr;
use std::os::unix::ffi::OsStrExt;

use curl_sys::{CURL, CURLcode, CURLoption};

macro_rules! load_next_fn {
	( $name:ident ( $($arg:ident : $type:ty),*$(,)? ) -> $ret:ty ) => {{
		let name   = stringify!($name).as_bytes().as_ptr() as *const i8;
		unsafe {
			// Clear dlerror() before calling dlsym().
			libc::dlerror();

			// Look-up the wanted symbol.
			let symbol = libc::dlsym(libc::RTLD_NEXT, name);

			// Check dlerror().
			let error  = libc::dlerror();
			if !error.is_null() {
				// Convert to Err(String).
				Err(format!("failed to look up symbol {}: {}", stringify!($name), CStr::from_ptr(error).to_string_lossy()))
			} else {
				// Convert to Ok(fn).
				let symbol : extern "C" fn($($arg : $type),*) -> $ret = std::mem::transmute(symbol);
				Ok(symbol)
			}
		}
	}}
}

struct Init {
	next_curl_easy_init       : extern "C" fn() -> *mut CURL,
	next_curl_easy_setopt_str : extern "C" fn(handle: *mut CURL, option: CURLoption, value: *const u8) -> CURLcode,
	cert_path                 : Option<std::ffi::OsString>,
	key_path                  : Option<std::ffi::OsString>,
}


impl Init {
	fn init() -> Result<Self, String> {
		Ok(Self {
			next_curl_easy_init       : load_next_fn!(curl_easy_init() -> *mut CURL)?,
			next_curl_easy_setopt_str : load_next_fn!(curl_easy_setopt(handle: *mut CURL, option: CURLoption, value: *const u8) -> CURLcode)?,
			cert_path                 : std::env::var_os("CURL_INJECT_OPT_SSLCERT"),
			key_path                  : std::env::var_os("CURL_INJECT_OPT_SSLKEY"),
		})
	}
}

static mut _INIT : Option<Result<Init, String>> = None;

extern "C" fn initialize() {
	unsafe {
		_INIT = Some(Init::init());
	}
}

#[used]
#[allow(non_upper_case_globals)]
#[cfg_attr(target_os = "linux", link_section = ".ctors")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
pub static init_curl_inject: extern "C" fn() = {
	initialize
};

pub extern "C" fn curl_easy_init() -> *mut CURL {
	let init = match unsafe { _INIT.as_ref().unwrap() } {
		Err(string) => panic!("{}", string),
		Ok(init)    => init,
	};


	// Delegate to the real handler.
	let handle = (init.next_curl_easy_init)();

	// Set client cert path if requested.
	if let Some(value) = &init.cert_path {
		(init.next_curl_easy_setopt_str)(handle, curl_sys::CURLOPT_SSLCERT, value.as_bytes().as_ptr());
	}

	// Set client key path if requested.
	if let Some(value) = &init.key_path {
		(init.next_curl_easy_setopt_str)(handle, curl_sys::CURLOPT_SSLKEY, value.as_bytes().as_ptr());
	}

	handle
}
