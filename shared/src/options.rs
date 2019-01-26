use super::{CURL, CURLcode, CurlEasySetOpt};

use std::ffi::{CStr, CString};
use std::os::raw::c_long;

pub enum OptionValue<'a> {
	#[used]
	CStr(&'a CStr),

	#[used]
	CLong(c_long),
}

impl<'a> std::fmt::Display for OptionValue<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match *self {
			OptionValue::CStr(x)  => x.to_string_lossy().fmt(f),
			OptionValue::CLong(x) => x.fmt(f),
		}
	}
}

macro_rules! define_options {
	( $( ($($options:tt)*) ),* $(,)?) => {
		define_options! { @parse tail {$(($($options)*),)*}; enum {}; setter(curl_easy_setopt, handle) {}; from_str(key, value) {}; key() {}; value() {}; }
	};

	(@parse
		tail {};
		enum {$($enum_body:tt)*};
		setter($set_easyopt:ident, $handle:ident) {$($setter_body:tt)*};
		from_str($key:ident, $value:ident) {$($from_str_body:tt)*};
		key() {$($key_body:tt)*};
		value() {$($value_body:tt)*};
	) => {
		#[derive(Clone, Debug)]
		pub enum CurlOption {
			$($enum_body)*
		}

		impl CurlOption {
			pub fn set(&self, $set_easyopt: CurlEasySetOpt, $handle: *mut CURL) -> CURLcode {
				match self {
					$($setter_body)*
				}
			}

			fn parse(raw: &str) -> Result<Self, String> {
				let split_at = raw.find("=").ok_or_else(|| String::from("invalid format for option, expected name=value"))?;
				let key      = &raw[..split_at];
				let value    = &raw[split_at + 1..];

				let key = Self::as_ascii(key).map_err(|i| format!("option contains non-ascii value at index {}: `{}`...", i, &key[..i.min(60)]))?;

				if key.len() > 60 {
					// Already checked that the whole key is ASCII, so this slicing is safe.
					return Err(format!("option name exceeds maximum length of 60 characters: {}...", &key[..60]))
				}

				Self::from_key_value(&key.to_ascii_lowercase(), value)
			}

			fn from_key_value($key: &str, $value: &str) -> Result<Self, String> {
				match $key {
					$($from_str_body)*
					_ => Err(format!("unrecognized option: {}", $key))
				}
			}

			#[used]
			fn parse_int(key: &str, value: &str) -> Result<std::os::raw::c_long, String> {
				value.parse().map_err(|_| format!("invalid integer value for option `{}`: {}", key, value))
			}

			#[used]
			fn parse_str(key: &str, value: &str) -> Result<CString, String> {
				CString::new(value).map_err(|_| format!("option `{}` value contains zero byte", key))
			}

			pub fn key(&self) -> &'static str {
				match self {
					$($key_body)*
				}
			}

			pub fn value(&self) -> OptionValue {
				match self {
					$($value_body)*
				}
			}

			pub fn to_string(&self) -> String {
				format!("{}={}", self.key(), self.value())
			}

			fn as_ascii(value: &str) -> Result<&str, usize> {
				for (i, byte) in value.bytes().enumerate() {
					if !byte.is_ascii() {
						return Err(i);
					}
				}
				return Ok(value);
			}
		}
	};

	(@parse
		tail { (str, $rust_name:ident, $name:literal, $curl_name:expr), $($tail:tt)* };
		enum {$($enum_body:tt)*};
		setter($set_easyopt:ident, $handle:ident) {$($setter_body:tt)*};
		from_str($key:ident, $value:ident) {$($from_str_body:tt)*};
		key() {$($key_body:tt)*};
		value() {$($value_body:tt)*};
	) => {
		define_options! {
			@parse

			tail {
				$($tail)*
			};

			enum {
				$($enum_body)*
				$rust_name(CString),
			};

			setter($set_easyopt, $handle) {
				$($setter_body)*
				CurlOption::$rust_name(x) => ($set_easyopt)($handle, $curl_name, x.as_ref() as *const CStr),
			};

			from_str($key, $value) {
				$($from_str_body)*
				$name => Ok(CurlOption::$rust_name(Self::parse_str($key, $value)?)),
			};

			key() {
				$($key_body)*
				CurlOption::$rust_name(_) => $name,
			};

			value() {
				$($value_body)*
				CurlOption::$rust_name(x) => OptionValue::CStr(x),
			};
		}
	};

	(@parse
		tail { (int, $rust_name:ident, $name:literal,$curl_name:expr ), $($tail:tt)* };
		enum {$($enum_body:tt)*};
		setter($set_easyopt:ident, $handle:ident) {$($setter_body:tt)*};
		from_str($key:ident, $value:ident) {$($from_str_body:tt)*};
		key() {$($key_body:tt)*};
		value() {$($value_body:tt)*};
	) => {
		define_options! {
			@parse

			tail {
				$($tail)*
			};

			enum {
				$($enum_body)*
				$rust_name(std::os::raw::c_long),
			};

			setter($set_easyopt, $handle) {
				$($setter_body)*
				CurlOption::$rust_name(x) => ($set_easyopt)($handle, $curl_name, x),
			};

			from_str($key, $value) {
				$($from_str_body)*
				$name => Ok(CurlOption::$rust_name(Self::parse_int($key, $value)?)),
			};

			key() {
				$($key_body)*
				CurlOption::$rust_name(_) => $name,
			};

			value() {
				$($value_body)*
				CurlOption::$rust_name(x) => OptionValue::CLong(x),
			};
		}
	};
}

impl std::str::FromStr for CurlOption {
	type Err = String;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		Self::parse(value)
	}
}

define_options! [
	(str, ClientCert, "client-cert", curl_sys::CURLOPT_SSLCERT),
	(str, ClientKey,  "client-key",  curl_sys::CURLOPT_SSLKEY),
];
