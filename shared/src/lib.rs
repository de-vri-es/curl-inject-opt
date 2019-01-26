pub mod config {
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

pub type CurlEasySetOpt  = extern "C" fn(handle: *mut CURL, option: CURLoption, ...) -> CURLcode;
pub type CurlEasyPerform = extern "C" fn(handle: *mut CURL) -> CURLcode;

pub fn serialize_options<'a>(options: impl Iterator<Item = &'a CurlOption>) -> Result<String, String> {
	let vec : Vec<String> = options.map(|option| option.to_string()).collect();
	serde_json::to_string(&vec).map_err(|x| format!("failed to encode curl options: {}", x))
}

pub fn parse_options(data: &str) -> Result<Vec<CurlOption>, String> {
	let vec : Vec<String> = serde_json::from_str(data).map_err(|x| format!("failed to parse curl options: {}", x))?;
	vec.iter().map(|x| x.parse()).collect()
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
