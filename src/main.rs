use std::io::prelude::*;
use std::env;

mod srt;

fn is_separator(c: char) -> bool {
	return c == ' ' || c == '\u{A0}' || c == '.' || c == ',';
}

struct Text {
	line: String
}

impl Text {
	// Replaces a word or multiple words.
	// If the word to search ends with a *: a separator or letter can follow
	// If the word to search does not: only a separator can follow
	fn replace(&mut self, what: &str, with: &str) {
		self.line = {
			let mut new_line = String::new();
			let mut line_str = self.line.as_str();
			let (what_no_star, anything_can_follow) = {
				if what.ends_with("*") {
					(&what[..what.len()-1], true)
				}
				else {
					(what, false)
				}
			};

			loop {
				match line_str.find(what_no_star) {
					None => {
						new_line.push_str(line_str);
						break;
					},
					Some(index) => {
						// Found! Verify the next char
						let next_char = line_str[index + what_no_star.len()..].chars().next();
						let do_replace = match next_char {
							Some(c) => anything_can_follow || is_separator(c),
							None => true
						};
						let (left, right) = line_str.split_at(index);
						new_line.push_str(left);
						if do_replace {
							new_line.push_str(with);
						}
						else {
							new_line.push_str(what_no_star);
						}
						line_str = &right[what_no_star.len()..];
						if line_str.is_empty() {
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
		("  ", " "),

		// Ordinaux
		("1ère",  "1ʳᵉ"),
		("2ème",  "2ᵉ"),
		("3ème",  "3ᵉ"),
		("4ème",  "4ᵉ"),
		("5ème",  "5ᵉ"),
		("6ème",  "6ᵉ"),
		("25ème", "25ᵉ"),

		("Ca", "Ça"),

		("Ecartez",    "Écartez"),
		("Echantillon", "Échantillon"),
		("Ecole",      "École"),
		("Economis*",  "Économis*"),
		("Ecout*",     "Écout"),
		("Ecras*",     "Écras"),
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
		("A rien",    "À rien"),
		("A son",     "À son"),
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
		("coeur",   "cœur"),
		("coeurs",  "cœurs"),
		("oeuf",    "œuf"),
		("oeufs",   "œufs"),
		("oei*",    "œi"),
		("noeud",   "nœud"),
		("noeuds",  "nœuds"),
		("soeur",   "sœur"),
		("soeurs",  "sœurs"),
		("oeuvre",  "œuvre"),
		("oeuvres", "œuvres"),
		("voeux",   "vœux"),

		// Misc
		("des qu'", "dès qu'"),
		("que tu ais", "que tu aies"),
		("Tous les 2", "Tous les deux"),
		("J'ai du vérifier", "J'ai dû vérifier"),
		("c'est règlé", "c'est réglé"),
		
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
	assert_eq!("Ça va", replace_one("Ca va"));
	assert_eq!("Ça.", replace_one("Ca."));
	assert_eq!("Ça", replace_one("Ca"));
	assert!("Çaribou" != replace_one("Caribou"));
	assert_eq!("œizz", replace_one("oeizz"));
	assert_eq!("des frères, des sœurs.", replace_one("des frères, des soeurs."));
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
		assert!(subs.len() == 2);
		assert!(subs[0].num == 42);
		assert!(subs[0].text_count == 1);
		assert!(subs[0].text[0] == "suspicious empty line above");
		assert!(subs[1].num == 43);
		assert!(subs[1].text_count == 1);
		assert!(subs[1].text[0] == "mango");
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
		assert!(subs[0].text[0] == "hello");
		assert!(subs[0].text[1] == "mister");
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
		assert!(subs[1].text[0] == "hi");
	}
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
	file_path: String,
	out_file_path: String
}

fn parse_app_args() -> AppArgs {
	let mut args = env::args();
	if args.len() == 1 {
		println!("fixsrt - Hadrien Nilsson - 2016");
		println!("usage: fixsrt [-nobak] SRTFILE [-out OUTFILE]");
		std::process::exit(0);
	}
	args.next().unwrap(); // Skip programe name

	let (no_backup, file_path) = {
		match args.next() {
			Some(arg) => {
				if arg == "-nobak" {
					match args.next() {
						Some(arg2) => (true, arg2),
						None => {
							println!("Missing file");
							std::process::exit(1);
						}
					}
				}
				else {
					(false, arg)
				}
			},
			None => {
				println!("No arg");
				std::process::exit(1);
			}
		}
	};

	let has_out = match args.next() {
		Some(arg) => if arg == "-out" {
			true
		}
		else {
			println!("Unexpected argument: {}", arg);
			std::process::exit(1);
		},
		None => {
			false
		}
	};

	let out_file_path = if has_out {
		match args.next() {
			Some(arg) => arg,
			None => {
				println!("Missing output file");
				std::process::exit(1);
			}
		}
	}
	else {
		file_path.clone() // Same as input file path
	};
	
	AppArgs {
		no_backup: no_backup,
		file_path: file_path,
		out_file_path: out_file_path
	}
}

///////////////////////////////////////////////////////////////////////////////
fn main() {
	let app_args = parse_app_args();

	let subtitles_res = srt::load_subtitles(&app_args.file_path);
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
		let backup_file_path = format!("{}~", &app_args.file_path);
		match std::fs::copy(&app_args.file_path, &backup_file_path) {
			Ok(_) => (),
			Err(err) => println!("Cannot create backup: {}", err)
		}
	}
	match srt::save_subtitles(&subtitles, &app_args.out_file_path) {
		Ok(_) => (),
		Err(_) => {
			println!("Save failed");
			std::process::exit(1);
		}
	}
	println!("Done - {} subtitles", subtitles.len())
}
