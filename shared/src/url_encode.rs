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

pub fn escape_comma(byte: u8) -> bool {
	byte == b',' || byte == b'%'
}

pub fn encode_append(buffer: &mut Vec<u8>, data: &[u8], should_escape: impl Fn(u8) -> bool) {
	let escape_count = data.iter().filter(|byte| should_escape(**byte)).count();
	buffer.reserve(data.len() + escape_count * 2);

	for &byte in data {
		if should_escape(byte) {
			buffer.push(b'%');
			buffer.push(u8_to_ascii_hex_digit(byte >> 4 & 0x0f).unwrap());
			buffer.push(u8_to_ascii_hex_digit(byte >> 0 & 0x0f).unwrap());
		} else {
			buffer.push(byte);
		}
	}
}

pub fn encode(data: &[u8], should_escape: impl Fn(u8) -> bool) -> Vec<u8> {
	let mut buffer = Vec::new();
	encode_append(&mut buffer, data, should_escape);
	buffer
}

pub fn decode_append(buffer: &mut Vec<u8>, data: &[u8]) -> Result<(), String> {
	buffer.reserve(data.len());

	let mut i = 0;
	while i < data.len() {
		let byte = data[i];
		if byte == b'%' {
			let high = u8_from_ascii_hex_digit(data[i + 1]).ok_or_else(|| format!("invalid hexadecimal digit: {}", data[i + 1]))?;
			let low  = u8_from_ascii_hex_digit(data[i + 2]).ok_or_else(|| format!("invalid hexadecimal digit: {}", data[i + 2]))?;
			buffer.push(high << 4 | low);
			i += 3;
		} else {
			buffer.push(byte);
			i += 1;
		}
	}

	Ok(())
}

pub fn decode(data: &[u8]) -> Result<Vec<u8>, String> {
	let mut buffer = Vec::new();
	decode_append(&mut buffer, data)?;
	Ok(buffer)
}

fn u8_from_ascii_hex_digit(byte: u8) -> Option<u8> {
	match byte {
		b'0' ..= b'9' => Some(byte - b'0'),
		b'a' ..= b'f' => Some(byte - b'a' + 10),
		b'A' ..= b'F' => Some(byte - b'A' + 10),
		_ => None,
	}
}

fn u8_to_ascii_hex_digit(byte: u8) -> Option<u8> {
	match byte {
		0  ..= 9  => Some(b'0' + byte),
		10 ..= 15 => Some(b'A' + byte - 10),
		_ => None,
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_urlencode() {
		assert_eq!(&encode(b",foo,bar,",       escape_comma), b"%2Cfoo%2Cbar%2C");
		assert_eq!(&encode(b"%2Cfoo%2Cbar%2C", escape_comma), b"%252Cfoo%252Cbar%252C");
		assert_eq!(&encode(b"%,foo%,%bar,%", escape_comma),   b"%25%2Cfoo%25%2C%25bar%2C%25");
	}

	#[test]
	fn test_urldecode() {
		assert_eq!(&decode(b"%2Cfoo%2Cbar%2C").unwrap(),             b",foo,bar,");
		assert_eq!(&decode(b"%252Cfoo%252Cbar%252C").unwrap(),       b"%2Cfoo%2Cbar%2C");
		assert_eq!(&decode(b"%25%2Cfoo%25%2C%25bar%2C%25").unwrap(), b"%,foo%,%bar,%");
	}
}
