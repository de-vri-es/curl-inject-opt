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

use super::{CURL, CURLcode, CURLoption, CurlEasySetOpt};

use std::ffi::{CStr, CString};
use std::os::raw::c_long;

macro_rules! curl_option {
	( $name:literal, $curl_name:ident, $type:expr, $help:literal ) => {
		Meta {
			name      : $name,
			curl_name : stringify!($curl_name),
			option    : curl_sys::$curl_name,
			kind      : $type,
			help      : concat!(stringify!($curl_name), ": ", $help),
		}
	};
}

/// Global list of known CURL options.
pub const OPTIONS : &[Meta] = &[
	curl_option!("verbose",          CURLOPT_VERBOSE,           Kind::CLong,   "Enable verbose output from CURL."),

	curl_option!("timeout",          CURLOPT_TIMEOUT_MS,        Kind::CLong,   "Timeout in milliseconds for the whole request."),
	curl_option!("connect-timeout",  CURLOPT_CONNECTTIMEOUT_MS, Kind::CLong,   "Timeout in milliseconds for the connection phase of the request."),

	curl_option!("proxy",            CURLOPT_PROXY,             Kind::CString, "Set the proxy to use."),
	curl_option!("proxy-port",       CURLOPT_PROXYPORT,         Kind::CLong,   "Set the proxy port."),
	curl_option!("proxy-type",       CURLOPT_PROXYTYPE,         Kind::CString, "Set the proxy type."),
	curl_option!("proxy-tunnel",     CURLOPT_HTTPPROXYTUNNEL,   Kind::CLong,   "Use CONNECT to tunnel through a configured HTTP proxy."),
	curl_option!("no-proxy",         CURLOPT_NOPROXY,           Kind::CString, "Contact these hosts directly, bypassing the proxy."),

	curl_option!("client-cert",      CURLOPT_SSLCERT,           Kind::CString, "Use a client certificate for requests."),
	curl_option!("client-cert-type", CURLOPT_SSLCERTTYPE,       Kind::CString, "Specify the type of the client certificate."),
	curl_option!("client-key",       CURLOPT_SSLKEY,            Kind::CString, "Use a separate file as key with the client certificate."),
	curl_option!("client-key-type",  CURLOPT_SSLKEYTYPE,        Kind::CString, "Specify the type of the client key."),

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

	/// The CURL name for the option.
	pub curl_name: &'static str,

	/// The CURLoption value for the option.
	pub option: CURLoption,

	/// The type of the options: CString or CLong.
	pub kind: Kind,

	/// A description of the option for humans.
	pub help: &'static str,
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
