use super::{CURL, CURLcode, CURLoption, CurlEasySetOpt};

use std::ffi::{CStr, CString};
use std::os::raw::c_long;

macro_rules! curl_option {
	( $name:literal, $curl_name:ident, $type:expr, $help:literal ) => {
		Meta::new($name, curl_sys::$curl_name, $type, concat!(stringify!($curl_name), " - ", $help))
	};
}

/// Global list of known CURL options.
pub const OPTIONS : &[Meta] = &[
	curl_option!("verbose",          CURLOPT_VERBOSE,         Kind::CLong,   "Set to 1 to enable verbose output from CURL."),

	curl_option!("proxy",            CURLOPT_PROXY,           Kind::CString, "Set the proxy to use."),
	curl_option!("proxy-port",       CURLOPT_PROXYPORT,       Kind::CLong,   "Set the proxy port."),
	curl_option!("proxy-type",       CURLOPT_PROXYTYPE,       Kind::CString, "Set the proxy type."),
	curl_option!("proxy-tunnel",     CURLOPT_HTTPPROXYTUNNEL, Kind::CLong,   "Set to 1 to use CONNECT to tunnel through a configured HTTP proxy."),
	curl_option!("no-proxy",         CURLOPT_NOPROXY,         Kind::CString, "Set hosts to contact directly, bypassing the proxy settings."),

	curl_option!("client-cert",      CURLOPT_SSLCERT,         Kind::CString, "Use a client certificate to authenticate with a remote server."),
	curl_option!("client-cert-type", CURLOPT_SSLCERTTYPE,     Kind::CString, "Specify the type of the client certificate (normally defaults to PEM)."),
	curl_option!("client-key",       CURLOPT_SSLKEY,          Kind::CString, "Use a separate file as key with the client certificate."),

	//curl_option!("proxy-client-cert",      CURLOPT_PROXY_SSLCERT,      Kind::CString, "Use a client certificate to authenticate with the proxy."),
	//curl_option!("proxy-client-cert-type", CURLOPT_PROXY_SSLCERTTYPE,  Kind::CString, "Specify the type of the proxy client certificate."),
	//curl_option!("proxy-client-key",       CURLOPT_PROXY_SSLKEY,       Kind::CString, "Use the given key with the proxy client certificate."),
];

/// The possible kinds of CURL options.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
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
#[derive(Clone, Debug, PartialEq, PartialOrd)]
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

fn parse_long(bytes: &[u8]) -> Result<c_long, std::num::ParseIntError> {
	// NOTE: from_str_radix only works on ASCII anyway.
	// It immediately converts the string to bytes anyway.
	let as_str = unsafe { std::str::from_utf8_unchecked(bytes) };
	c_long::from_str_radix(as_str, 10)
}

impl SetOption {
	/// Parse the value for an option with known metadata.
	pub fn parse_value(meta: Meta, value: &[u8]) -> Result<Self, String> {
		let value = match meta.kind {
			Kind::CString => Value::CString(CString::new(value).map_err(|_| format!("value for option {} contains a null byte", meta.name))?),
			Kind::CLong   => Value::CLong(parse_long(value).map_err(|_| format!("invalid integer value for option {}", meta.name))?),
		};

		Ok(Self{name: meta.name, option: meta.option, value})
	}

	/// Parse an option from the name and value.
	///
	/// The name will be lookup up in the global OPTIONS list to retrieve the required metadata.
	pub fn parse_name_value(name: &str, value: &[u8]) -> Result<Self, String> {
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
