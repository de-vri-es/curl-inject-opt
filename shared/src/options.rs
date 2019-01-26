use super::{CURL, CURLcode, CurlEasySetOpt};

use std::ffi::{CStr, CString};
use std::os::raw::c_long;

define_options! [
	(str, ClientCert, "client-cert", curl_sys::CURLOPT_SSLCERT),
	(str, ClientKey,  "client-key",  curl_sys::CURLOPT_SSLKEY),
];

/// The value for a CURL option.
///
/// It can be either a null-terminated string, or a long integer as defined by C.
pub enum OptionValue<'a> {
	#[used]
	CStr(&'a CStr),

	#[used]
	CLong(c_long),
}

impl<'a> std::fmt::Display for OptionValue<'a> {
	/// Printing an option value simply prints the string value or the integer value.
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match *self {
			OptionValue::CStr(x)  => x.to_string_lossy().fmt(f),
			OptionValue::CLong(x) => x.fmt(f),
		}
	}
}

/// Macro to define all the required boilerplate for a CURL option.
///
/// The macro takes care of defining an enum with the right variants,
/// setting the option on a CURL easy handle,
/// as well as string serialization and parsing.
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

		/// A CURL option with an embedded value.
		///
		/// Can be used to set the option, if given a CURL handle.
		#[derive(Clone, Debug)]
		pub enum CurlOption {
			$($enum_body)*
		}

		impl CurlOption {
			/// Set the value of this CURL option for a CURL easy handle.
			pub fn set(&self, $set_easyopt: CurlEasySetOpt, $handle: *mut CURL) -> CURLcode {
				match self {
					$($setter_body)*
				}
			}

			/// Parse a key=value string as CurlOption.
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

			/// Parse a CurlOption from a key and a value.
			fn from_key_value($key: &str, $value: &str) -> Result<Self, String> {
				match $key {
					$($from_str_body)*
					_ => Err(format!("unrecognized option: {}", $key))
				}
			}

			/// Parse an integer from a value string.
			#[used]
			fn parse_int(key: &str, value: &str) -> Result<std::os::raw::c_long, String> {
				value.parse().map_err(|_| format!("invalid integer value for option `{}`: {}", key, value))
			}

			/// Create a null-terminated from a value string.
			#[used]
			fn parse_str(key: &str, value: &str) -> Result<CString, String> {
				CString::new(value).map_err(|_| format!("option `{}` value contains zero byte", key))
			}

			/// Get the key of this CurlOption.
			pub fn key(&self) -> &'static str {
				match self {
					$($key_body)*
				}
			}

			/// Get the value of this CurlOption.
			pub fn value(&self) -> OptionValue {
				match self {
					$($value_body)*
				}
			}

			/// Convert this CurlOption to a string that can be parsed by `parse`.
			pub fn to_string(&self) -> String {
				format!("{}={}", self.key(), self.value())
			}

			/// Confirm that a str contains only ASCII, or get the index of the first non-ASCII byte.
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
