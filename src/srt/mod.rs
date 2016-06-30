use std::io;
use std::io::prelude::*;
use std::str::FromStr;
use std::io::BufReader;
use std::fs::File;
use std::io::{Error, ErrorKind};

mod workfile;
//use workfile;

#[derive(Default)]
pub struct Subtitle {
	num: u32,
	duration: String,
	pub texts: [String; 5],
	pub text_count: u32
}

impl Subtitle {
	pub fn to_string(&self) -> String {
		let mut ret = self.num.to_string();
		ret.push_str("\r\n");
		ret.push_str(&self.duration);
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
	for line in content.lines() {
		//println!("[{:?}] {}", state, line);
		match state {
			State::WantsNum => {
				subtitle.num = match u32::from_str(&line) {
					Ok(val) => val,
					Err(err) => {
						return Err(format!("Bad number at line {}: {}: '{}'",
							line_num, err, line));
					}
				};
				state = State::WantsDuration;
			},
			State::WantsDuration => {
				subtitle.duration = line.to_string();
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
						return Err(format!("Too much text at line {}", line_num));
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

const BOM: [u8;3] = [0xEF, 0xBB, 0xBF];

///////////////////////////////////////////////////////////////////////////////
pub fn load_subtitles(file_path: &str) -> Result<Vec<Subtitle>,String> {
	let file = match File::open(file_path) {
		Ok(file) => file,
		Err(err) => {
			return Err(format!("Cannot open file: {}", err));
		}
	};

	let mut buf_reader = BufReader::new(file);

	let remove_bom = {
		let maybe_bom = buf_reader.fill_buf().unwrap();
		if maybe_bom.len() >= 3
		&& maybe_bom[0..3] == BOM {
			true
		}
		else {
			false
		}
	};
	if remove_bom {
		buf_reader.consume(3);
	}
	let mut content = String::new();
	match buf_reader.read_to_string(&mut content) {
		Ok(_) => (),
		Err(err) => {
			return Err(format!("Cannot read content: {}", err));
		}
	}
	return parse_srt(&content);
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
