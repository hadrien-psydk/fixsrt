use std::env;

mod workfile;
mod srt;
mod rules;

fn is_separator(c: char) -> bool {
	return c == ' ' || c == '\u{A0}'
	 || c == '.' || c == ','
	 || c == '"' || c == '-';
}

fn is_letter(c: char) -> bool {
	return !is_separator(c);
}

fn is_digit(c: char) -> bool {
	return c.is_digit(10);
}

fn find_start_at(slice: &str, pat: &str, start_at: usize) -> Option<usize> {
    slice[start_at..].find(pat).map(|i| start_at + i)
}

#[derive(Debug)]
enum Follow {
	Nothing,
	Any,
	Letter,
	Digit
}

// Replaces a word or multiple words.
// - If the word to search starts or ends with a *:
//   a separator or letter can precede or follow
// - If the word to search starts or ends with a +:
//   a letter can precede or follow
// - If the word to search starts or ends with a #:
//   a digit can precede or follow
// - If the word to search does not:
//   only a separator can precede or follow
fn replace_by_rule(line_str: &str, what: &str, with: &str) -> String {
	let mut new_line = String::new(); // Result

	// First check if the text to search ends with a star.
	let (what_no_star_after, follow) = {
		if what.ends_with("*") {
			(&what[..what.len()-1], Follow::Any)
		}
		else if what.ends_with("+") {
			(&what[..what.len()-1], Follow::Letter)
		}
		else if what.ends_with("#") {
			(&what[..what.len()-1], Follow::Digit)
		}
		else {
			(what, Follow::Nothing)
		}
	};

	let (what_no_star, precede) = {
		if what_no_star_after.starts_with("*") {
			(&what_no_star_after[1..], Follow::Any)
		}
		else if what_no_star_after.starts_with("+") {
			(&what_no_star_after[1..], Follow::Letter)
		}
		else if what_no_star_after.starts_with("#") {
			(&what_no_star_after[1..], Follow::Digit)
		}
		else {
			(what_no_star_after, Follow::Nothing)
		}
	};
	/*
	println!("what: {} what_no_star: {} precede: {:?} follow: {:?}",
		what, what_no_star, precede, follow);*/

	let mut start_at = 0;
	loop {
		// Perform a simple string search first, then look at the result
		// more precisely
		match find_start_at(line_str, what_no_star, start_at) {
			None => {
				new_line.push_str(&line_str[start_at..]);
				break;
			},
			Some(index) => {
				// Found! Verify the characters around to know if it is really a word

				// Look also the previous char, we do not want to replace text in the
				// middle of a word. We accept ' as prev char too (not as next char).
				let prev_char = line_str[0..index].chars().last();
				let do_replace1 = match prev_char {
					Some(c) => match precede {
						Follow::Nothing => !is_letter(c) || (c == '\''),
						Follow::Any => true,
						Follow::Letter => is_letter(c),
						Follow::Digit => is_digit(c)
					},
					None => true // Beginning of the line
				};

				// Now, next char
				let next_char = line_str[index + what_no_star.len()..].chars().next();
				let do_replace2 = match next_char {
					Some(c) => match follow {
						Follow::Nothing => is_separator(c),
						Follow::Any => true,
						Follow::Letter => is_letter(c),
						Follow::Digit => is_digit(c)
					},
					None => true
				};

				// Poor man debugger
				/*
				println!("line: '{}' index: {} prev_char: {:?} next_char: {:?} r1: {} r2: {}",
					line_str, index, prev_char, next_char,
					do_replace1, do_replace2);*/

				new_line.push_str(&line_str[start_at..index]);
				start_at = index + what_no_star.len();
				let end_reached = start_at >= line_str.len();

				if do_replace1 && do_replace2 {
					// Replace the found text but verify if we are at the
					// end of a line and the "with" text ends with a space
					let with2 = if end_reached && with.ends_with(' ') {
						&with[0..with.len()-1]
					}
					else {
						with
					};
					new_line.push_str(with2);
				}
				else {
					new_line.push_str(what_no_star);
				}

				if end_reached {
					break;
				}
			}
		}
	}
	new_line
}

///////////////////////////////////////////////////////////////////////////////
// Replaces words in one line of text based on rules
fn replace_one(text: &str) -> String {
	let mut result = text.to_string();

	for rule_ref in rules::RULES_FR.iter() {
		let &(what, with) = rule_ref;
		result = replace_by_rule(&result, what, with);
	}
	result
}

#[test]
fn test_replace_one() {
	assert_eq!(replace_one("Ca va"), "Ça va");
	assert_eq!(replace_one("Ca."), "Ça.");
	assert_eq!(replace_one("Ca"), "Ça");
	assert!(replace_one("Caribou") != "Çaribou");
	assert_eq!(replace_one("oeizz"), "œizz");
	assert_eq!(replace_one("des soeurs."), "des sœurs.");

	// No space if newline
	assert_eq!(replace_one("D'ou sa"), "D'où sa");
	assert_eq!(replace_one("bien sur,"), "bien sûr,");

	assert_eq!(replace_one("bien sur,"), "bien sûr,");
	assert_eq!(replace_one("Ecart entre"), "Écart entre");
	assert_eq!(replace_one("un coca"), "un coca");
	assert_eq!(replace_one("l'Etat"), "l'État");
	assert_eq!(replace_one("ok?"), "ok\u{A0}?");
	assert_eq!(replace_one("ok ?"), "ok\u{A0}?");
	assert_eq!(replace_one("ok!"), "ok\u{A0}!");
	assert_eq!(replace_one("ok !"), "ok\u{A0}!");
	assert_eq!(replace_one("A quoi?"), "À quoi\u{A0}?");
	assert_eq!(replace_one("A t'écouter"), "À t'écouter");
	assert_eq!(replace_one("manoeuvrer"), "manœuvrer");
	assert_eq!(replace_one("Etiez-vous"), "Étiez-vous");
	assert_eq!(replace_one("des qu'il"), "dès qu'il");
	assert_eq!(replace_one("10ème"), "10ᵉ");
	assert_eq!(replace_one("10e"), "10ᵉ");
	assert_eq!(replace_one("10è"), "10ᵉ");
	assert_eq!(replace_one("\"Oeil pour oeil\""), "\"Œil pour œil\"");
	assert_eq!(replace_one("Etaient-ils"), "Étaient-ils");
	assert_eq!(replace_one("caca"), "caca");
}

///////////////////////////////////////////////////////////////////////////////
fn do_replacements(subtitles: &mut Vec<srt::Subtitle>) {
	for subtitle in subtitles.iter_mut() {
		for text_index in 0..subtitle.text_count as usize {
			subtitle.texts[text_index] = replace_one(&subtitle.texts[text_index]);
		}
		//print!("{}", subtitle.to_string());
	}
}

///////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
struct AppArgs {
	no_backup: bool,
	in_file_paths: Vec<String>,
	out_file_path: String
}

impl AppArgs {
	fn new() -> AppArgs {
		AppArgs { no_backup: false, in_file_paths: Vec::new(), out_file_path: String::new() }
	}
}

fn parse_app_args(args: Vec<String>) -> Result<AppArgs, String> {

	enum State {
		WantsNoBakOrInput,
		WantsInput,
		WantsOutOrAnotherInput,
		WantsOutput,
		WantsNothing
	};
	let mut state = State::WantsNoBakOrInput;

	let mut app_args = AppArgs::new();
	for arg in args {
		match state {
			State::WantsNoBakOrInput => if arg == "-nobak" {
				app_args.no_backup = true;
				state = State::WantsInput;
			}
			else {
				app_args.in_file_paths.push(arg);
				state = State::WantsOutOrAnotherInput;
			},
			State::WantsInput => {
				app_args.in_file_paths.push(arg);
				state = State::WantsOutOrAnotherInput;
			},
			State::WantsOutOrAnotherInput => if arg == "-out" {
				if app_args.in_file_paths.len() > 1 {
					return Err(String::from("-out works only with single input"));
				}
				state = State::WantsOutput
			}
			else {
				app_args.in_file_paths.push(arg);
			},
			State::WantsOutput => {
				app_args.out_file_path = arg;
				// -out disables backup
				app_args.no_backup = true;
				state = State::WantsNothing;
			},
			State::WantsNothing => {
				return Err(format!("Unexpected argument: {}", arg));
			}
		}
	}

	match state {
		State::WantsNoBakOrInput => {
			return Err(String::from("Missing arguments"));
		},
		State::WantsInput => {
			return Err(String::from("Missing input"));
		},
		State::WantsOutOrAnotherInput => (),
		State::WantsOutput => {
			return Err(String::from("Missing output"));
		}
		State::WantsNothing => ()
	}

	Ok(app_args)
}

#[test]
macro_rules! string_vec {
    ( $( $x:expr ),* ) => {
        {
            let mut _temp_vec = Vec::new();
            $(
                _temp_vec.push($x.to_string());
            )*
            _temp_vec
        }
    };
}

#[test]
fn test_parse_app_args() {
	assert!( parse_app_args(string_vec![]).is_err() );
	assert!( parse_app_args(string_vec!["-nobak"]).is_err() );
	assert!( parse_app_args(string_vec!["myfile", "-out"]).is_err() );
	assert!( parse_app_args(string_vec!["myfile1", "myfile2", "-out"]).is_err() );
	assert!( parse_app_args(string_vec!["myfile1", "myfile2", "-out", "myres"]).is_err() );

	let mut res = parse_app_args(string_vec!["myfile"]);
	assert!( res.is_ok() );
	let mut app_args = res.unwrap();
	assert_eq!( false, app_args.no_backup );
	assert_eq!( 1, app_args.in_file_paths.len() );
	assert_eq!( "myfile", app_args.in_file_paths[0] );
	assert!( app_args.out_file_path.is_empty() );

	res = parse_app_args(string_vec!["-nobak", "myfile"]);
	assert!( res.is_ok() );
	app_args = res.unwrap();
	assert_eq!( true, app_args.no_backup );
	assert_eq!( 1, app_args.in_file_paths.len() );
	assert_eq!( "myfile", app_args.in_file_paths[0] );
	assert!( app_args.out_file_path.is_empty() );

	res = parse_app_args(string_vec!["myfile", "-out", "myres"]);
	assert!( res.is_ok() );
	app_args = res.unwrap();
	assert_eq!( true, app_args.no_backup );
	assert_eq!( 1, app_args.in_file_paths.len() );
	assert_eq!( "myfile", app_args.in_file_paths[0] );
	assert_eq!( "myres", app_args.out_file_path );

	res = parse_app_args(string_vec!["myfile1", "myfile2"]);
	assert!( res.is_ok() );
	app_args = res.unwrap();
	assert_eq!( false, app_args.no_backup );
	assert_eq!( 2, app_args.in_file_paths.len() );
	assert_eq!( "myfile1", app_args.in_file_paths[0] );
	assert_eq!( "myfile2", app_args.in_file_paths[1] );
	assert!( app_args.out_file_path.is_empty() );
}

///////////////////////////////////////////////////////////////////////////////
fn main() {
	let mut args = env::args();
	if args.len() == 1 {
		println!("fixsrt v15 - Hadrien Nilsson - 2016/2017");
		println!("usage: fixsrt [-nobak] SRTFILE [SRTFILE2 [SRTFILE3 [...]]] [-out OUTFILE]");
		std::process::exit(0);
	}
	args.next().unwrap(); // Skip programe name

	let app_args = match parse_app_args(args.collect()) {
		Ok(app_args) => app_args,
		Err(err) => {
			println!("{}", err);
			std::process::exit(1);
		}
	};

	for in_file_path in app_args.in_file_paths {
		print!("{} ... ", in_file_path);

		let subtitles_res = srt::load_subtitles(&in_file_path);
		match subtitles_res {
			Ok(_) => (),
			Err(ref err) => {
				println!("{}", err);
				std::process::exit(1);
			}
		}

		let mut subtitles = subtitles_res.unwrap();
		do_replacements(&mut subtitles);

		// Do backup
		if !app_args.no_backup {
			let backup_file_path = format!("{}~", in_file_path);
			match std::fs::copy(&in_file_path, &backup_file_path) {
				Ok(_) => (),
				Err(err) => println!("Cannot create backup: {}", err)
			}
		}

		let out_file_path = if app_args.out_file_path.is_empty() {
			&in_file_path
		}
		else {
			&app_args.out_file_path
		};

		match srt::save_subtitles(&subtitles, &out_file_path) {
			Ok(_) => (),
			Err(_) => {
				println!("Save failed");
				std::process::exit(1);
			}
		}
		println!("done: {} subtitles", subtitles.len());
	}
}
