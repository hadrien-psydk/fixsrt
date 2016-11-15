use std::env;

mod workfile;
mod srt;

fn is_separator(c: char) -> bool {
	return c == ' ' || c == '\u{A0}' || c == '.' || c == ',' || c == '"';
}

fn is_letter(c: char) -> bool {
	return !is_separator(c);
}

fn is_digit(c: char) -> bool {
	return c.is_digit(10);
}

struct Text {
	line: String
}

#[derive(Debug)]
enum Follow {
	Nothing,
	Any,
	Letter,
	Digit
}

impl Text {
	// Replaces a word or multiple words.
	// - If the word to search starts or ends with a *:
	//   a separator or letter can precede or follow
	// - If the word to search starts or ends with a +:
	//   a letter can precede or follow
	// - If the word to search starts or ends with a #:
	//   a digit can precede or follow
	// - If the word to search does not:
	//   only a separator can precede or follow
	fn replace(&mut self, what: &str, with: &str) {
		self.line = {
			let mut new_line = String::new(); // Result
			let mut line_str = self.line.as_str(); // Current piece of text

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

			loop {
				// Perform a simple string search first, then look at the result
				// more precisely
				match line_str.find(what_no_star) {

					None => {
						new_line.push_str(line_str);
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

						let (left, right) = line_str.split_at(index);
						new_line.push_str(left);
						line_str = &right[what_no_star.len()..];
						let end_reached = line_str.is_empty();

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
		};
	}
}

///////////////////////////////////////////////////////////////////////////////
fn replace_one(text: &str) -> String {
	let mut t = Text { line: text.to_string() };

	let rules = [
		// Trop d'espaces
		("  ", " "),

		// Espace insécable
		("+!",  "\u{A0}!"),
		("+ !", "\u{A0}!"),
		("+?",  "\u{A0}?"),
		("+ ?", "\u{A0}?"),

		// Ordinaux
		("1er",   "1ᵉʳ"),
		("1ers",  "1ᵉʳˢ"),
		("1ère",  "1ʳᵉ"),
		("1ères",  "1ʳᵉˢ"),

		("2ème",  "2ᵉ"),
		("3ème",  "3ᵉ"),
		("4ème",  "4ᵉ"),
		("5ème",  "5ᵉ"),
		("6ème",  "6ᵉ"),
		("7ème",  "7ᵉ"),
		("8ème",  "8ᵉ"),
		("9ème",  "9ᵉ"),

		("#0ème",  "0ᵉ"),
		("#1ème",  "1ᵉ"),
		("#2ème",  "2ᵉ"),
		("#3ème",  "3ᵉ"),
		("#4ème",  "4ᵉ"),
		("#5ème",  "5ᵉ"),
		("#6ème",  "6ᵉ"),
		("#7ème",  "7ᵉ"),
		("#8ème",  "8ᵉ"),
		("#9ème",  "9ᵉ"),

		("2e",  "2ᵉ"),
		("3e",  "3ᵉ"),
		("4e",  "4ᵉ"),
		("5e",  "5ᵉ"),
		("6e",  "6ᵉ"),
		("7e",  "7ᵉ"),
		("8e",  "8ᵉ"),
		("9e",  "9ᵉ"),

		("#0e",  "0ᵉ"),
		("#1e",  "1ᵉ"),
		("#2e",  "2ᵉ"),
		("#3e",  "3ᵉ"),
		("#4e",  "4ᵉ"),
		("#5e",  "5ᵉ"),
		("#6e",  "6ᵉ"),
		("#7e",  "7ᵉ"),
		("#8e",  "8ᵉ"),
		("#9e",  "9ᵉ"),

		("2è",  "2ᵉ"),
		("3è",  "3ᵉ"),
		("4è",  "4ᵉ"),
		("5è",  "5ᵉ"),
		("6è",  "6ᵉ"),
		("7è",  "7ᵉ"),
		("8è",  "8ᵉ"),
		("9è",  "9ᵉ"),

		("#0è",  "0ᵉ"),
		("#1è",  "1ᵉ"),
		("#2è",  "2ᵉ"),
		("#3è",  "3ᵉ"),
		("#4è",  "4ᵉ"),
		("#5è",  "5ᵉ"),
		("#6è",  "6ᵉ"),
		("#7è",  "7ᵉ"),
		("#8è",  "8ᵉ"),
		("#9è",  "9ᵉ"),

		// Cédille
		("ca", "ça"),
		("Ca", "Ça"),
		("lecon",   "leçon"),

		("D'ou", "D'où"),

		// Capitales accentuées
		("Ecart*",     "Écart"),
		("Echantillon", "Échantillon"),
		("Ecole",      "École"),
		("Economis*",  "Économis*"),
		("Ecout*",     "Écout"),
		("Ecras*",     "Écras"),
		("Ecris*",     "Écris"),
		("Ecume",      "Écume"),
		("Edition",    "Édition"),
		("Egoïste",    "Égoïste"),
		("Egypt*",     "Égypt"),
		("Elevé",      "Élevé"),
		("Eloign*",    "Éloign"),
		("Etat",       "État"),
		("Etais",      "Étais"),
		("Etant",      "Étant"),
		("Eteins ça",  "Éteins ça"),
		("Eteins-moi", "Éteins-moi"),
		("Eteins le",  "Éteins le"),
		("Eteins la",  "Éteins la"),
		("Etiez-vous", "Étiez-vous"),
		("Etonnant",   "Étonnant"),
		("Evidemment", "Évidemment"),
		("Evite*",     "Évite"),

		("Etes", "Êtes"),
		("Etre", "Être"),
		
		("A 1*",        "À 1"),
		("A 2*",        "À 2"),
		("A 3*",        "À 3"),
		("A 4*",        "À 4"),
		("A 5*",        "À 5"),
		("A 6*",        "À 6"),
		("A 7*",        "À 7"),
		("A 8*",        "À 8"),
		("A 9*",        "À 9"),
		("A bientôt",   "À bientôt"),
		("A cause",     "À cause"),
		("A ceci",      "À ceci"),
		("A ce",        "À ce"),
		("A chaque",    "À chaque"),
		("A combien",   "À combien"),
		("A commencer", "À commencer"),
		("A condition", "À condition"),
		("A croire",    "À croire"),
		("A courir",    "À courir"),
		("A demain",    "À demain"),
		("A déplacer",  "À déplacer"),
		("A des",       "À des"),
		("A deux",      "À deux"),
		("A emporter",  "À emporter"),
		("A faire",   "À faire"),
		("A fumer",   "À fumer"),
		("A l'*",     "À l'"),
		("A ma",      "À ma"),
		("A me",      "À me"),
		("A moins",   "À moins"),
		("A mon",     "À mon"),
		("A genoux",  "À genoux"),
		("A la",      "À la"),
		("A Los",     "À Los"),
		("A ne",      "À ne"),
		("A notre",   "À notre"),
		("A nous",    "À nous"),
		("A nouveau", "À nouveau"),
		("A part",    "À part"),
		("A peine,",  "À peine,"),
		("A peine",   "À peine"),
		("A peu près", "À peu près"),
		("A plus",    "À plus"),
		("A propos",  "À propos"),
		("A quelle",  "À quelle"),
		("A quoi",    "À quoi"),
		("A rien",    "À rien"),
		("A son",     "À son"),
		("A t'*",      "À t'"),
		("A table",   "À table"),
		("A terre",   "À terre"),
		("A te",      "À te"),
		("A tes",     "À tes"),
		("A toi",     "À toi"),
		("A ton",     "À ton"),
		("A tou",     "À tou"),
		("A un",      "À un"),
		("A votre",   "À votre"),
		("A vous",    "À vous"),
		("A vos",     "À vos"),
		("A vrai dire", "À vrai dire"),
		("A vendredi",  "À vendredi"),
		
		// Ligature œ
		("boeuf",     "bœuf"),
		("coeur",     "cœur"),
		("coeurs",    "cœurs"),
		("oeuf",      "œuf"),
		("oeufs",     "œufs"),
		("oei*",      "œi"),
		("Oeuf",      "Œuf"),
		("Oeufs",     "Œufs"),
		("Oei*",      "Œi"),
		("manoeuvrer","manœuvrer"),
		("noeud",     "nœud"),
		("noeuds",    "nœuds"),
		("recue",     "reçue"),
		("recus",     "reçus"),
		("soeur",     "sœur"),
		("soeurs",    "sœurs"),
		("oeuvre",    "œuvre"),
		("oeuvres",   "œuvres"),
		("Oeuvre",    "Œuvre"),
		("Oeuvres",   "Œuvres"),
		("voeux",     "vœux"),

		// Misc
		("des qu'*", "dès qu'"),
		("que tu ais", "que tu aies"),
		("Tous les 2", "Tous les deux"),
		("J'ai du vérifier", "J'ai dû vérifier"),
		("c'est règlé", "c'est réglé"),
		("Quelque soient", "Quels que soient"),

		// Sûr
		("Je suis sur qu'*", "Je suis sûr qu'"),
		("Bien sur,", "Bien sûr,"),
		("bien sur,", "bien sûr,"),
		
		// Subjonctif présent 2e personne
		("N'ais", "N'aies"),

		// Participe passé au lieu d'infinitif
		(" se déplacé", " se déplacer"),
		
		// Conjuguaison
		("Je doit", "Je dois")
	];
	for rule_ref in rules.iter() {
		let &(what, with) = rule_ref;
		t.replace(what, with);
	}

	t.line
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
		println!("fixsrt v13 - Hadrien Nilsson - 2016");
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
