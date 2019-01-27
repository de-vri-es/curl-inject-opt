pub fn escape_comma(byte: u8) -> bool {
	byte == b',' || byte == b'%'
}

pub fn url_encode_append(buffer: &mut Vec<u8>, data: &[u8], should_escape: impl Fn(u8) -> bool) {
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

pub fn url_encode(data: &[u8], should_escape: impl Fn(u8) -> bool) -> Vec<u8> {
	let mut buffer = Vec::new();
	url_encode_append(&mut buffer, data, should_escape);
	buffer
}

pub fn url_decode_append(buffer: &mut Vec<u8>, data: &[u8]) -> Result<(), String> {
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

pub fn url_decode(data: &[u8]) -> Result<Vec<u8>, String> {
	let mut buffer = Vec::new();
	url_decode_append(&mut buffer, data)?;
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
