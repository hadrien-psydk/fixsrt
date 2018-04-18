use std;
use std::io;
use std::io::prelude::*;
use std::str::FromStr;
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::str;

use workfile;

#[derive(Default)]
pub struct Subtitle {
	num: u32,
	pub time_from: i32,
	pub time_to : i32,
	pub texts: [String; 5],
	pub text_count: u32
}

impl Subtitle {
	pub fn to_string(&self) -> String {
		let mut ret = self.num.to_string();
		ret.push_str("\r\n");
		ret.push_str(&format_srt_time_from_milli(self.time_from));
		ret.push_str(" --> ");
		ret.push_str(&format_srt_time_from_milli(self.time_to));
		ret.push_str("\r\n");
		for text_index in 0..self.text_count as usize {
			ret.push_str(&self.texts[text_index]);
			ret.push_str("\r\n");
		}
		ret.push_str("\r\n");
		ret
	}

	// Returns true if full
	fn push_text(&mut self, line: &str) -> bool {
		let next_index = self.text_count as usize;
		if next_index == self.texts.len() {
			true
		}
		else {
			self.texts[next_index] = line.to_string();
			self.text_count += 1;
			false
		}
	}
}

pub fn parse_srt(content: &str) -> Result<Vec<Subtitle>,String> {
	let mut subtitles: Vec<Subtitle> = Vec::new();

	#[derive(Debug)]
	enum State {
	    WantsNum,
	    WantsDuration,
	    WantsFirstText,
	    WantsFirstTextAgain,
	    WantsFollowingText
	}
	let mut state: State = State::WantsNum;

	let mut subtitle: Subtitle = Default::default();
	let mut line_num = 1;

	for line_ori in content.lines() {
		//println!("[{:?}] {}", state, line);

		// We may encounter a fake empty line, so trim the line before
		// looking at it
		let line = line_ori.trim_right();

		match state {
			State::WantsNum => {
				if line.is_empty() {
					// That's suspicious but accepted
					// Stay in the WantsNum state
				}
				else {
					subtitle.num = match u32::from_str(&line) {
						Ok(val) => val,
						Err(err) => {
							return Err(format!("Bad number at line {}: {}: '{}'",
								line_num, err, line));
						}
					};
					state = State::WantsDuration;
				}
			},
			State::WantsDuration => {
				let arrow_index = match line.find("-->") {
					Some(index) => index,
					None => {
						return Err(format!("Bad duration separator at line {}: {}",
							line_num, line));
					}
				};
				let left = &line[0..arrow_index].trim();
				let right = &line[arrow_index + 3..].trim();

				subtitle.time_from = match parse_srt_time_with_sign(left) {
					Some(milli) => milli,
					None => {
						return Err(format!("Bad time 'from' at line {}: {}",
							line_num, left));
					}
				};
				subtitle.time_to = match parse_srt_time_with_sign(right) {
					Some(milli) => milli,
					None => {
						return Err(format!("Bad time 'to' at line {}: {}",
							line_num, right));
					}
				};

				state = State::WantsFirstText;
			},
			State::WantsFirstText => {
				if line.is_empty() {
					// That's suspicious
					state = State::WantsFirstTextAgain;
				}
				else {
					subtitle.push_text(line);
					state = State::WantsFollowingText;
				}
			},
			State::WantsFirstTextAgain => {
				if line.is_empty() {
					// That's suspicious
				}
				else {
					// Maybe the subtitle is empty, check for a number
					let new_one = if let Ok(val) = u32::from_str(&line) {
						// Is that a new subtitle? Or the text that finaly came?
						if val == subtitle.num + 1 {
							// We assume it's a new subtitle
							subtitles.push(subtitle);
							subtitle = Default::default();
							subtitle.num = val;
							state = State::WantsDuration;
							true
						}
						else {
							false
						}
					}
					else {
						false
					};

					if !new_one {
						// The non-empty line is the line we were waiting for
						subtitle.push_text(line);
						state = State::WantsFollowingText;
					}
				}
			},
			State::WantsFollowingText => {
				if line.is_empty() {
					subtitles.push(subtitle);
					subtitle = Default::default();
					state = State::WantsNum;
				}
				else {
					if subtitle.push_text(line) {
						return Err(format!("Too much text at line {} sub {}",
							line_num, subtitle.num));
					}
				}
			}
		}
		line_num += 1;
	}

	// Push the last subtitle because we may not have an empty line
	// to know the last subtitle ended
	if subtitle.text_count > 0 {
		subtitles.push(subtitle);
	}
	Ok(subtitles)
}

#[test]
fn test_parse_srt() {
	///////////////////////////////////
	// Unexpected empty line in text
	{
		let srt = r#"42
00:00:16,087 --> 00:00:19,911

suspicious empty line above

43
00:00:20,000 --> 00:00:21,000
mango"#;
		let subs_res = parse_srt(srt);
		assert!(subs_res.is_ok(), format!("{}", subs_res.err().unwrap()) );
		let subs = subs_res.unwrap();
		assert_eq!(subs.len(), 2);
		assert_eq!(subs[0].num, 42);
		assert!(subs[0].text_count == 1);
		assert!(subs[0].texts[0] == "suspicious empty line above");
		assert!(subs[1].num == 43);
		assert!(subs[1].text_count == 1);
		assert!(subs[1].texts[0] == "mango");
	}

	///////////////////////////////////
	// Simple case
	{
		let srt = r#"42
00:00:16,087 --> 00:00:19,911
hello"#;
		let subs_res = parse_srt(srt);
		assert!(subs_res.is_ok(), format!("{}", subs_res.err().unwrap()) );
		let subs = subs_res.unwrap();
		assert!(subs.len() == 1);
	}

	///////////////////////////////////
	// Two lines of text
	{
		let srt = r#"42
00:00:16,087 --> 00:00:19,911
hello
mister

43
00:00:20,000 --> 00:00:21,000
hi"#;
		let subs_res = parse_srt(srt);
		assert!(subs_res.is_ok(), format!("{}", subs_res.err().unwrap()) );
		let subs = subs_res.unwrap();
		assert!(subs.len() == 2);
		assert!(subs[0].num == 42);
		assert!(subs[0].text_count == 2);
		assert!(subs[0].texts[0] == "hello");
		assert!(subs[0].texts[1] == "mister");
	}

	///////////////////////////////////
	// No text
	{
		let srt = r#"42
00:00:16,087 --> 00:00:19,911

43
00:00:20,000 --> 00:00:21,000
hi"#;
		let subs_res = parse_srt(srt);
		assert!(subs_res.is_ok(), format!("{}", subs_res.err().unwrap()) );
		let subs = subs_res.unwrap();
		assert!(subs.len() == 2);
		assert!(subs[0].num == 42);
		assert!(subs[0].text_count == 0);
		assert!(subs[1].num == 43);
		assert!(subs[1].text_count == 1);
		assert!(subs[1].texts[0] == "hi");
	}

	///////////////////////////////////
	// Double empty line
	{
		let srt = r#"51
00:00:16,087 --> 00:00:19,911
begin


52
00:00:20,000 --> 00:00:21,000
end"#;
		let subs_res = parse_srt(srt);
		assert!(subs_res.is_ok(), format!("{}", subs_res.err().unwrap()) );
		let subs = subs_res.unwrap();
		assert_eq!(subs.len(), 2);
		assert_eq!(subs[0].num, 51);
		assert_eq!(subs[0].text_count, 1);
		assert_eq!(subs[0].texts[0], "begin");
		assert_eq!(subs[1].num, 52);
		assert_eq!(subs[1].text_count, 1);
		assert_eq!(subs[1].texts[0], "end");
	}

	///////////////////////////////////
	// Fake empty line (contains spaces)
	{
		let srt = r#"61
00:00:16,087 --> 00:00:19,911
begin

62
00:00:20,000 --> 00:00:21,000
end"#;
		let subs_res = parse_srt(srt);
		assert!(subs_res.is_ok(), format!("{}", subs_res.err().unwrap()) );
		let subs = subs_res.unwrap();
		assert_eq!(subs.len(), 2);
		assert_eq!(subs[0].num, 61);
		assert_eq!(subs[0].text_count, 1);
		assert_eq!(subs[0].texts[0], "begin");
		assert_eq!(subs[1].num, 62);
		assert_eq!(subs[1].text_count, 1);
		assert_eq!(subs[1].texts[0], "end");
	}
}

///////////////////////////////////////////////////////////////////////////////
const W1252_80_9F: [char; 32] = [
	'\u{20ac}', '\u{0081}', '\u{201a}', '\u{0192}',
	'\u{201e}', '\u{2026}', '\u{2020}', '\u{2021}',
	'\u{02c6}', '\u{2030}', '\u{0160}', '\u{2039}',
	'\u{0152}', '\u{008d}', '\u{017d}', '\u{008f}',
	'\u{0090}', '\u{2018}', '\u{2019}', '\u{201c}',
	'\u{201d}', '\u{2022}', '\u{2013}', '\u{2014}',
	'\u{02dc}', '\u{2122}', '\u{0161}', '\u{203a}',
	'\u{0153}', '\u{009d}', '\u{017e}', '\u{0178}'
	];

fn decode_windows_1252(content: &[u8]) -> String {
	let mut ret = String::with_capacity(content.len() * 2);

	for cp8 in content {
		let c = if 0x80 <= *cp8 && *cp8 <= 0x9f {
			W1252_80_9F[(*cp8 as usize) - 0x80]
		}
		else {
			*cp8 as char
		};
		ret.push(c);
	}
	ret
}

#[test]
fn test_decode_windows_1252() {
    let raw = [
    	0x64u8, 0xe9, 0x6a, 0xe0,
    	0x20,
    	0x62,0x9c,0x75,0x66,
    	0x20,
    	0x33,0x80];
    let text = decode_windows_1252(&raw);
    assert!(text == "déjà bœuf 3€");
}

const BOM: [u8;3] = [0xEF, 0xBB, 0xBF];

///////////////////////////////////////////////////////////////////////////////
pub fn load_subtitles(file_path: &str) -> Result<Vec<Subtitle>,String> {
	let content = {
		let mut file = match File::open(file_path) {
			Ok(file) => file,
			Err(err) => {
				return Err(format!("Cannot open file: {}", err));
			}
		};
		let mut bytes = Vec::new();
		if let Err(err) = file.read_to_end(&mut bytes) {
			return Err(format!("File read error: {}", err));
		};
		bytes
	};

	// Detect encoding

	let has_bom = {
		if content.len() >= 3 && content[0..3] == BOM {
			true
		}
		else {
			false
		}
	};

	// Temporary String in case the content should be recreated
	// from windows-1252 to utf-8
	let tmp_str;

	let content_str = if has_bom {
		match str::from_utf8(&content[3..]) {
			Ok(res) => res,
			Err(err) => {
				return Err(format!("Invalid UTF-8: {}", err));
			}
		}
	}
	else {
		// Check if it is UTF-8 without BOM
		match str::from_utf8(&content) {
			Ok(res) => res,
			Err(_) => {
				// Assume it is windows-1252
				tmp_str = decode_windows_1252(&content);
				tmp_str.as_str()
			}
		}
	};

	return parse_srt(&content_str);
}

///////////////////////////////////////////////////////////////////////////////
pub fn save_subtitles(subtitles: &Vec<Subtitle>, file_path: &str) -> io::Result<()> {

	let mut work_file = match workfile::WorkFile::create(file_path) {
		Ok(file) => file,
		Err(err) => {
			println!("Cannot create file {}", err);
			return Err(err);
		}
	};
	match work_file.write(&BOM) {
		Ok(len) => if len != BOM.len() {
			println!("Cannot write BOM: not enough space");
			return Err(Error::new(ErrorKind::Other, "bad len BOM"));
		},
		Err(err) => {
			println!("Cannot write BOM: {}", err);
			return Err(err);
		}
	}
	for subtitle in subtitles.iter() {
		let data_str = subtitle.to_string();
		let data = data_str.as_bytes();
		match work_file.write(data) {
			Ok(len) => if len != data.len() {
				println!("Cannot write subtitle: not enough space");
				return Err(Error::new(ErrorKind::Other, "bad len"));
			},
			Err(err) => {
				println!("Cannot write subtitle: {}", err);
				return Err(err);
			}
		}
	}
	work_file.commit();
	Ok(())
}

///////////////////////////////////////////////////////////////////////////////
// Parses a SRT time expressed as hours:minutes:seconds,milliseconds.
// milliseconds can also be separated with '.'
// hours and minutes are optional
//
// Examples:
// 42.123
// 42.5
// 07:42,6
// 01:25:34,421
//
// Returns a number of milliseconds
pub fn parse_srt_time(time_str: &str) -> Option<i32> {
	
	if time_str.is_empty() {
		return None;
	}

	const MAX_DEC_DIGITS: usize = 3;
	const TEN_POW: &'static [i32] = &[1, 10, 100, 1000];

	let mut frag_index = 0;
	let mut frag_len = 0;
	let mut frags = [0i32;4];
	let mut milliseconds_str = "";

	// H:M:S
	for (i, c) in time_str.chars().enumerate() {
		if c.is_digit(10) {
			if frag_len < 2 {
				frags[frag_index] *= 10;
				frags[frag_index] += c.to_digit(10).unwrap() as i32;
				frag_len += 1;
			}
			else {
				return None; // Parse error: too many digits
			}
		}
		else {
			if frag_len == 0 {
				// No digit found, refuse the input
				return None;
			}
			if c == ':' {
				// Next frag
				frag_index += 1;
				frag_len = 0;
			}
			else if c == '.' || c == ',' {
				milliseconds_str = &time_str[i + 1..];
				break;
			}
			else
			{
				// Parse error
				return None;
			}
		}
	}

	// Adjust for missing frags
	if frag_index == 0 {
		// H:M missing
		frags[2] = frags[0];
		frags[1] = 0;
		frags[0] = 0;
	}
	else if frag_index == 1 {
		// H missing
		frags[2] = frags[1];
		frags[1] = frags[0];
		frags[0] = 0;
	}

	// Milliseconds
	if milliseconds_str.len() > 0 {
		frag_index = 3;
		frag_len = 0;

		for c in milliseconds_str.chars() {
			if c.is_digit(10) {
				if frag_len < MAX_DEC_DIGITS {
					frags[frag_index] *= 10;
					frags[frag_index] += c.to_digit(10).unwrap() as i32;
					frag_len += 1;
				}
				else {
					return None; // Parse error: too many digits
				}
			}
			else {
				// Parse error
				return None;
			}
		}
		// Adjust number of digits
		if frag_len < MAX_DEC_DIGITS {
			frags[3] *= TEN_POW[MAX_DEC_DIGITS - frag_len];
		}
	}

	// Combine all fragments
	let mut r: i64 = frags[0] as i64;
	r += (frags[0] * 60 * 60) as i64;
	r += (frags[1] * 60) as i64;
	r += frags[2] as i64;
	r *= TEN_POW[3] as i64;
	r += frags[3] as i64;

    if !(std::i32::MIN as i64 <= r && r <= std::i32::MAX as i64) {
        None
    } else {
        Some(r as i32)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Parses a time prefixed with an optional sign.
//
// Returns a number of milliseconds
pub fn parse_srt_time_with_sign(time_str: &str) -> Option<i32> {
	
	if time_str.is_empty() {
		return None;
	}

	let mut negate = false;

	let time_str_no_sign = {
		let maybe_c = time_str.chars().next();
		match maybe_c {
			Some(c) => match c {
				'+' => &time_str[1..],
				'-' => { negate = true; &time_str[1..] },
				_ => time_str
			},
			None => time_str
		}
	};
	let milli = parse_srt_time(time_str_no_sign)?;

	if negate {
		Some(-milli)
	}
	else {
		Some(milli)
	}
}

fn format_srt_time_from_milli(milli: i32) -> String {
	let milli_abs = if milli < 0 { -milli } else { milli };
	let sign_str = if milli < 0 { "-" } else { "" };

	let milli_rem = milli_abs % 1000;

	let mut tmp = milli_abs / 1000;
	let hours = tmp / (60 * 60);
	tmp = tmp % (60 * 60);
	let minutes = tmp / 60;
	let seconds = tmp % 60;

	let ret = format!("{}{:02}:{:02}:{:02},{:03}",
		sign_str, hours, minutes, seconds, milli_rem);
	ret
}

#[test]
fn test_format_srt_time_from_milli() {
	assert_eq!(format_srt_time_from_milli(42_000), "00:00:42,000");
	assert_eq!(format_srt_time_from_milli(62_789), "00:01:02,789");
	assert_eq!(format_srt_time_from_milli(3*60*60*1000+62_789), "03:01:02,789");
}

#[test]
fn test_parse_srt_time() {
	assert_eq!(parse_srt_time("42"), Some(42_000));
	assert_eq!(parse_srt_time("1.247"), Some(1_247));
	assert_eq!(parse_srt_time(""), None);
	assert_eq!(parse_srt_time("."), None);
	assert_eq!(parse_srt_time("0"), Some(0));
	assert_eq!(parse_srt_time("0."), Some(0));
	assert_eq!(parse_srt_time("0.2"), Some(0_200));
	assert_eq!(parse_srt_time("0.23"), Some(0_230));
	assert_eq!(parse_srt_time("0.234"), Some(0_234));
	assert_eq!(parse_srt_time("0.2345"), None);
	assert_eq!(parse_srt_time("14,28"), Some(14_280));
	assert_eq!(parse_srt_time("01:14,28"), Some(1*60_000 + 14_280));
}

#[test]
fn test_parse_srt_time_with_sign() {
	assert_eq!(parse_srt_time_with_sign(""), None);
	assert_eq!(parse_srt_time_with_sign("42"), Some(42_000));
	assert_eq!(parse_srt_time_with_sign("+42"), Some(42_000));
	assert_eq!(parse_srt_time_with_sign("-42"), Some(-42_000));
}

// Sometimes the last subtitle is not suitable, so we should ignore it
pub fn should_keep_last_sub(subtitles: &Vec<Subtitle>) -> bool {
	let sub_count = subtitles.len();
	if sub_count == 0 {
		return true;
	}

	let last_time = subtitles[sub_count - 1].time_from;
	if sub_count == 1 {
		return last_time > 0;
	}

	let prelast_time = subtitles[sub_count - 2].time_from;
	if last_time < prelast_time {
		// Last is going backward, do not keep it
		false
	}
	else if (last_time - prelast_time) > 5*60*60*1000 {
		// Last is very large, do not keep it
		false
	}
	else {
		true
	}
}

#[test]
fn test_should_keep_last_sub() {
		let srt = r#"1
00:00:00,000 --> 00:00:01,000
blah

2
00:00:10,000 --> 00:00:11,000
yop

3
-00:00:10,000 --> 00:00:11,000
yop
"#;
	let subs_res = parse_srt(srt);
	assert!(subs_res.is_ok(), format!("{}", subs_res.err().unwrap()) );
	let subs = subs_res.unwrap();
	assert!(!should_keep_last_sub(&subs));
}
