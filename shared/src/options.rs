use super::{CURL, CURLcode, DynamicCurl};

macro_rules! define_options {
	( $( ($($options:tt)*) ),* $(,)?) => {
		define_options! { @parse tail {$(($($options)*),)*}; enum {}; setter(handle, lib) {}; }
	};

	(@parse tail {}; enum {$($enum_body:tt)*}; setter($handle:ident, $lib:ident) {$($setter_body:tt)*}; ) => {
		#[derive(Copy, Clone, Debug)]
		pub enum CurlOption<'a> {
			$($enum_body)*
		}

		impl CurlOption<'_> {
			pub fn set(self, $handle: *mut CURL, $lib: &DynamicCurl) -> CURLcode {
				match self {
					$($setter_body)*
				}
			}
		}
	};

	(@parse tail { (str, $name:ident, $curl_name:expr), $($tail:tt)* }; enum {$($enum_body:tt)*}; setter($handle:ident, $lib:ident) {$($setter_body:tt)*}; ) => {
		define_options! {
			@parse
			tail {
				$($tail)*
			};
			enum {
				$($enum_body)*
				$name(&'a std::ffi::CStr),
			};
			setter($handle, $lib) {
				$($setter_body)*
				CurlOption::$name(x) => $lib.set_option_str($handle, $curl_name, x),
			};
		}
	};

	(@parse tail { (int, $name:ident, $curl_name:expr ), $($tail:tt)* }; enum {$($enum_body:tt)*}; setter($handle:ident, $lib:ident) {$($setter_body:tt)*}; ) => {
		define_options! {
			@parse
			tail {
				$($tail)*
			};
			enum {
				$($enum_body)*
				$name(std::os::raw::c_long),
			};
			setter($handle, $lib) {
				$($setter_body)*
				CurlOption::$name(x) => $lib.set_option_int($handle, $curl_name, x),
			};
		}
	};
}

define_options! [
	(str, ClientCert, curl_sys::CURLOPT_SSLCERT),
	(str, ClientKey,  curl_sys::CURLOPT_SSLKEY),
];
