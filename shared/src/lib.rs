mod config;
mod options;
pub mod url_encode;

pub use self::config::{Config, parse_config};
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
