mod config {
	#[allow(dead_code)]
	mod raw {
		include!("../../config.rs");
	}

	pub use raw::PREFIX;
	pub use raw::LIBDIR;
	pub use raw::BINDIR;
}

pub use curl_sys::CURL;
pub use curl_sys::CURLoption;
pub use curl_sys::CURLcode;

pub type CurlEasySetOpt = extern "C" fn(handle: *mut CURL, option: CURLoption, ...) -> CURLcode;

pub struct DynamicCurl {
	curl_easy_setopt: CurlEasySetOpt,
}


impl DynamicCurl {
	pub fn set_option(&self, handle: *mut CURL, option: CurlOption) -> CURLcode {
		option.set(handle, self)
	}

	pub fn set_option_str(&self, handle: *mut CURL, option: CURLoption, value: &std::ffi::CStr) -> CURLcode {
		(self.curl_easy_setopt)(handle, option, value as *const std::ffi::CStr)
	}

	pub fn set_option_int(&self, handle: *mut CURL, option: CURLoption, value: std::os::raw::c_long) -> CURLcode {
		(self.curl_easy_setopt)(handle, option, value)
	}
}

mod options;
pub use options::CurlOption;

// #[derive(Clone, Copy, Debug)]
// pub enum CurlOption<'a> {
// 	ClientCert(&'a std::ffi::CStr),
// 	ClientKey(&'a std::ffi::CStr),
// }

// impl CurlOption<'_> {
// 	fn set(self, handle: *mut CURL, lib: &DynamicCurl) -> CURLcode {
// 		match self {
// 			Option::ClientCert(x) => lib.set_option_str(handle, curl_sys::CURLOPT_SSLCERT, x)
// 			Option::ClientKey(x)  => lib.set_option_str(handle, curl_sys::CURLOPT_SSLKEY, x)
// 		}
// 	}
// }
