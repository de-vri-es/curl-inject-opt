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

pub mod config;
pub mod url_encode;
mod options;

pub use self::options::{Kind, Value, Meta, SetOption, OPTIONS};

pub use curl_sys::CURL;
pub use curl_sys::CURLoption;
pub use curl_sys::CURLcode;

pub type CurlEasySetOpt  = extern "C" fn(handle: *mut CURL, option: CURLoption, ...) -> CURLcode;
pub type CurlEasyPerform = extern "C" fn(handle: *mut CURL) -> CURLcode;

fn encode_option_append(buffer: &mut Vec<u8>, option: &SetOption) {
	buffer.extend(option.name.as_bytes());
	buffer.push(b'=');

	match &option.value {
		Value::CString(x) => url_encode::encode_append(buffer, x.as_bytes(), url_encode::escape_comma),
		Value::CLong(x)   => buffer.extend(format!("{}", x).as_bytes()),
	}
}

fn decode_option(data: &[u8]) -> Result<SetOption, String> {
	let split_at = data.iter().position(|b| *b == b'=').ok_or_else(|| String::from("invalid option syntax, expected name=value"))?;
	let name     = std::str::from_utf8(&data[..split_at]).map_err(|_| String::from("option name contains invalid UTF-8"))?;
	let value    = url_encode::decode(&data[split_at + 1..]).map_err(|e| format!("failed to decode value for option {}: {}", name, e))?;

	SetOption::parse_name_value(name, &value)
}

pub fn serialize_options<'a>(options: impl Iterator<Item = &'a SetOption>) -> Vec<u8> {
	let mut buffer = Vec::new();
	let mut first  = true;
	for option in options {
		if !std::mem::replace(&mut first, false) {
			buffer.push(b',');
		}
		encode_option_append(&mut buffer, option);
	}

	buffer
}

pub fn parse_options(data: &[u8]) -> Result<Vec<SetOption>, String> {
	data.split(|b| *b == b',').filter(|x| !x.is_empty()).map(decode_option).collect()
}
