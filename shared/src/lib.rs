mod config;
mod options;

pub use self::config::{Config, parse_config};
pub use self::options::{Kind, Value, Meta, SetOption, OPTIONS};

pub use curl_sys::CURL;
pub use curl_sys::CURLoption;
pub use curl_sys::CURLcode;

pub type CurlEasySetOpt  = extern "C" fn(handle: *mut CURL, option: CURLoption, ...) -> CURLcode;
pub type CurlEasyPerform = extern "C" fn(handle: *mut CURL) -> CURLcode;

pub fn serialize_options<'a>(options: impl Iterator<Item = &'a SetOption>) -> Result<String, String> {
	let vec : Vec<(&str, String)> = options.map(|option| (option.name, format!("{}", option.value))).collect();
	serde_json::to_string(&vec).map_err(|x| format!("failed to encode curl options: {}", x))
}

pub fn parse_options(data: &str) -> Result<Vec<SetOption>, String> {
	let vec : Vec<(&str, String)> = serde_json::from_str(data).map_err(|x| format!("failed to parse curl options: {}", x))?;
	vec.into_iter().map(|(name, value)| {
		SetOption::parse_name_value(name, &value)
	}).collect()
}
