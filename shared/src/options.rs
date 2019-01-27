use super::{CURL, CURLcode, CURLoption, CurlEasySetOpt};

use std::ffi::{CStr, CString};
use std::os::raw::c_long;

use serde_derive::{Deserialize, Serialize};

/// Global list of known CURL options.
pub const OPTIONS : &[Meta] = &[
	Meta::new("client-cert", curl_sys::CURLOPT_SSLCERT, Kind::CString, "Use a client certificate to authenticate with a remote server."),
	Meta::new("client-key",  curl_sys::CURLOPT_SSLKEY,  Kind::CString, "Use the given key with the client certificate. Useful if the key isn't embedded in the certificate."),
];

/// The possible kinds of CURL options.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Kind {
	CString,
	CLong,
}

impl std::fmt::Display for Kind {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Kind::CString => "string".fmt(f),
			Kind::CLong   => "integer".fmt(f),
		}
	}
}

/// The value for a CURL option.
///
/// It can be either a null-terminated string, or a long integer as defined by C.
#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Value {
	CString(CString),
	CLong(c_long),
}

impl Value {
	/// Get the kind of the the value.
	fn kind(&self) -> Kind {
		match self {
			Value::CString(_) => Kind::CString,
			Value::CLong(_)   => Kind::CLong,
		}
	}
}

impl std::fmt::Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Value::CString(x) => x.to_string_lossy().fmt(f),
			Value::CLong(x)   => x.fmt(f),
		}
	}
}

/// Metadata for a CURL option.
#[derive(Clone, Copy, Debug)]
pub struct Meta {
	/// A human friendly name for the option.
	pub name: &'static str,

	/// The CURLoption value for the option.
	pub option: CURLoption,

	/// The type of the options: CString or CLong.
	pub kind: Kind,

	/// A description of the option for humans.
	pub help: &'static str,
}

impl Meta {
	/// Create the metadata for a CURL option from the components.
	pub const fn new(name: &'static str, option: CURLoption, kind: Kind, help: &'static str) -> Self {
		Self{name, option, kind, help}
	}
}

/// A CURL option with an embedded value.
///
/// Can be used to set the option on a CURL handle.
pub struct SetOption {
	/// A human friendly name for the option.
	pub name: &'static str,

	/// The CURLoption value for the option.
	pub option: CURLoption,

	/// The value to set for the option.
	pub value: Value,
}

impl SetOption {
	/// Parse the value for an option with known metadata.
	pub fn parse_value(meta: Meta, value: &str) -> Result<Self, String> {
		let value = match meta.kind {
			Kind::CString => Value::CString(CString::new(value).map_err(|_| format!("value for option {} contains a null byte", meta.name))?),
			Kind::CLong   => Value::CLong(value.parse().map_err(|_| format!("invalid integer value for option {}", meta.name))?),
		};

		Ok(Self{name: meta.name, option: meta.option, value})
	}

	/// Parse an option from the name and value.
	///
	/// The name will be lookup up in the global OPTIONS list to retrieve the required metadata.
	pub fn parse_name_value(name: &str, value: &str) -> Result<Self, String> {
		for candidate in OPTIONS {
			if name.eq_ignore_ascii_case(candidate.name) {
				return Self::parse_value(*candidate, value)
			}
		}

		Err(format!("unknown option: {}", name))
	}

	/// Parse an option from the name and value.
	///
	/// The name will be lookup up in the global OPTIONS list to retrieve the required metadata.
	pub fn parse_name(name: &str, value: Value) -> Result<Self, String> {
		for candidate in OPTIONS {
			if name.eq_ignore_ascii_case(candidate.name) {
				if candidate.kind != value.kind() {
					return Err(format!("wrong value type for option {}: expected {} but got {}", candidate.name, candidate.kind, value.kind()))
				}
				return Ok(Self{name: candidate.name, option: candidate.option, value: value})
			}
		}

		Err(format!("unknown option: {}", name))
	}

	/// Set the value of this CURL option for a CURL easy handle.
	pub fn set(&self, curl_easy_setopt: CurlEasySetOpt, handle: *mut CURL) -> CURLcode {
		match &self.value {
			Value::CString(x) => curl_easy_setopt(handle, self.option, x.as_ref() as *const CStr),
			Value::CLong(x)   => curl_easy_setopt(handle, self.option, *x),
		}
	}
}
