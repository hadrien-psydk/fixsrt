use std::io::prelude::*;
use std::str::FromStr;

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
