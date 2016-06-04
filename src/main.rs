#![allow(dead_code)]
#![allow(unused_imports)]
use std::io;
use std::io::Lines;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::process;
use std::str::FromStr;
use std::env;
use std::io::{Error, ErrorKind};
use std::borrow;

#[derive(Default)]
struct Subtitle {
	num: u32,
	duration: String,
	text: [String; 5],
	text_count: u32
}

impl Subtitle {
	fn to_string(&self) -> String {
		let mut ret = self.num.to_string();
		ret.push_str("\r\n");
		ret.push_str(&self.duration);
		ret.push_str("\r\n");
		for text_index in 0..self.text_count as usize {
			ret.push_str(&self.text[text_index]);
			ret.push_str("\r\n");
		}
		ret.push_str("\r\n");
		ret
	}
}

fn parse_lines<L: IntoIterator<Item=io::Result<String>>>(lines: L)
	-> Result<Vec<Subtitle>,String> {
	let mut subtitles: Vec<Subtitle> = Vec::new();

	enum State {
	    WantsNum,
	    WantsDuration,
	    WantsText
	}
	let mut state: State = State::WantsNum;

	let mut subtitle: Subtitle = Default::default();
	let mut counter = 0;
	for line_result in lines {
		let line = line_result.unwrap();
		match state {
			State::WantsNum => {
				subtitle.num = u32::from_str(&line).unwrap();
				state = State::WantsDuration;
			},
			State::WantsDuration => {
				subtitle.duration = line;
				state = State::WantsText;
			},
			State::WantsText => {
				if line.is_empty() {
					subtitles.push(subtitle);
					subtitle = Default::default();
					state = State::WantsNum;
				}
				else {
					let next_index = subtitle.text_count as usize;
					if next_index == subtitle.text.len() {
						return Err(format!("too much text for {}", subtitle.num))
					}
					else {
						subtitle.text[next_index] = line;
						subtitle.text_count += 1;
					}
				}
			}
		}
		counter += 1;
		if counter == 10 { break; }
	}

	if subtitle.text_count > 0 {
		subtitles.push(subtitle);
	}
	Ok(subtitles)
}

struct Text {
	line: String
}

impl Text {
	fn replace(&mut self, what: &str, with: &str) {
		self.line = {
			let mut new_line = String::new();
			let mut line_str = self.line.as_str();

			loop {
				match line_str.find(what) {
					None => {
						new_line.push_str(line_str);
						break;
					},
					Some(index) => {
						let (left, right) = line_str.split_at(index);
						new_line.push_str(left);
						new_line.push_str(with);
						line_str = &right[what.len()..];
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
	
	// Trop d'espaces
	let mut t = Text { line: text.to_string() };

	let rules = [
		("  ", " "),
		("Salut", "Yo"),

		// Ordinaux
		("2ème ", "2e "),
		("3ème ", "3e "),
		("4ème ", "4e "),
		("5ème ", "5e "),
		("6ème ", "6e "),
		("25ème ", "25e "),

		("Ca ", "Ça "),
		("Ca\u{00A0}", "Ça\u{00A0}"),
		("Ca,", "Ça,"),
		("Ca.", "Ça."),

		("Ecartez", "Écartez"),
		("Echantillon", "Échantillon"),
		("Ecole ", "École "),
		("Economis", "Économis"),
		("Ecout", "Écout"),
		("Ecras", "Écras"),
		("Edition", "Édition"),
		("Egoïste", "Égoïste"),
		("Egypt", "Égypt"),
		("Elevé", "Élevé"),
		("Eloign", "Éloign"),
		("Etat", "État"),
		("Etais", "Étais"),
		("Etant ", "Étant "),
		("Eteins ça", "Éteins ça"),
		("Eteins-moi ", "Éteins-moi "),
		("Eteins le ", "Éteins le "),
		("Eteins la ", "Éteins la "),
		("Etonnant", "Étonnant"),
		("Evidemment", "Évidemment"),
		("Evite ", "Évite "),

		("Etes", "Êtes"),
		("Etre ", "Être "),
		
		("A bientôt", "À bientôt"),
		("A cause", "À cause"),
		("A ceci", "À ceci"),
		("A ce ", "À ce "),
		("A chaque", "À chaque"),
		("A combien", "À combien"),
		("A commencer", "À commencer"),
		("A condition", "À condition"),
		("A croire ", "À croire "),
		("A demain", "À demain"),
		("A déplacer", "À déplacer"),
		("A des ", "À des "),
		("A deux", "À deux"),
		("A emporter", "À emporter"),
		("A faire", "À faire"),
		("A fumer", "À fumer"),
		("A l'", "À l'"),
		("A me ", "À me "),
		("A moins", "À moins"),
		("A mon ", "À mon "),
		("A genoux", "À genoux"),
		("A la ", "À la "),
		("A Los ", "À Los "),
		("A ne ", "À ne "),
		("A notre", "À notre"),
		("A nous ", "À nous "),
		("A nouveau", "À nouveau"),
		("A part", "À part"),
		("A peine,", "À peine,"),
		("A peine ", "À peine "),
		("A peu près", "À peu près"),
		("A plus !", "À plus !"),
		("A plus\u{00A0}!", "À plus\u{00A0}!"),
		("A plus,", "À plus,"),
		("A plus tard", "À plus tard"),
		("A propos", "À propos"),
		("A qu", "À qu"),
		("A rien", "À rien"),
		("A son ", "À son "),
		("A table", "À table"),
		("A terre ", "À terre"),
		("A te ", "À te "),
		("A tes ", "À tes "), // À tes yeux
		("A toi", "À toi"),
		("A ton ", "À ton "),
		("A tou", "À tou"),
		("A un", "À un"),
		("A votre ", "À votre "),
		("A vous ", "À vous "),
		("A vos ", "À vos "),
		("A vrai dire", "À vrai dire"),
		("A vendredi", "À vendredi"),
		
		// U+0153 = ligature oe	
		("coeur", "c\u{0153}ur"),
		("oeuf", "\u{0153}uf"),
		("oei", "\u{0153}i"),
		("noeud", "n\u{0153}ud"),
		("soeur", "s\u{0153}ur"),
		("oeuvre", "\u{0153}uvre"),
		("voeu", "v\u{0153}u"),

		// Misc
		("des qu'", "dès qu'"),
		("que tu ais ", "que tu aies "),
		("Tous les 2 ", "Tous les deux "),
		("J'ai du vérifier", "J'ai dû vérifier"),
		("c'est règlé", "c'est réglé"),
		
		// Subjonctif présent 2e personne
		("N'ais ", "N'aies "),

		// Participe passé au lieu d'infinitif
		(" se déplacé ", " se déplacer "),
		
		// Conuguaison
		("Je doit ", "Je dois ")
	];
	for rule_ref in rules.iter() {
		let &(what, with) = rule_ref;
		t.replace(what, with);
	}

	t.line
}

///////////////////////////////////////////////////////////////////////////////
fn do_replacements(subtitles: &mut Vec<Subtitle>) {
	for subtitle in subtitles.iter_mut() {
		subtitle.text[0] = replace_one(&subtitle.text[0]);
		print!("{}", subtitle.to_string());
	}
}

const BOM: [u8;3] = [0xEF, 0xBB, 0xBF];

struct WorkFile {
	file_path: String,
	work_file_path: String,
	file: Option<File>
}

impl WorkFile {
	fn create(file_path: &str) -> io::Result<WorkFile> {
		let work_file_path: String = format!("{}.work", file_path);
		let file = match File::create(&work_file_path) {
			Ok(file) => file,
			Err(err) => { return Err(err); }
		};
		Ok(WorkFile {
			file_path: file_path.to_string(),
			work_file_path: work_file_path,
			file: Some(file)
		})
	}

	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		let ret = match self.file {
			Some(ref mut some_file) => some_file.write(buf),
			None => Err( Error::new(ErrorKind::Other, "oops") )
		};
		ret
	}

	fn commit(&mut self) {
		let file = self.file.take();
		drop(file);
		match std::fs::rename(&self.work_file_path, &self.file_path) {
			Ok(_) => (),
			Err(err) => panic!("commit failed: {}", err)
		}
	}
}

impl Drop for WorkFile {
	fn drop(&mut self) {
		if self.file.is_some() {
			drop(self.file.take());
			match std::fs::remove_file(&self.work_file_path) {
				Ok(_) => (),
				Err(err) => panic!("rollback failed: {}", err)
			}
		}
	}
}

///////////////////////////////////////////////////////////////////////////////
fn save_subtitles(subtitles: &Vec<Subtitle>, file_path: &str) {

	let mut work_file = match WorkFile::create(file_path) {
		Ok(file) => file,
		Err(err) => {
			println!("Cannot create file {}", err);
			return;
		}
	};
	match work_file.write(&BOM) {
		Ok(len) => if len != BOM.len() {
			println!("Cannot write BOM: not enough space");
			return;
		},
		Err(err) => {
			println!("Cannot write BOM: {}", err);
			return;
		}
	}
	for subtitle in subtitles.iter() {
		let data_str = subtitle.to_string();
		let data = data_str.as_bytes();
		match work_file.write(data) {
			Ok(len) => if len != data.len() {
				println!("Cannot write subtitle: not enough space");
				return;
			},
			Err(err) => {
				println!("Cannot write subtitle: {}", err);
				return;
			}
		}
	}
	work_file.commit();
}

///////////////////////////////////////////////////////////////////////////////
fn load_subtitles(file_path: &str) -> Result<Vec<Subtitle>,String> {
	let file = match File::open(file_path) {
		Ok(file) => file,
		Err(err) => {
			println!("Cannot open file: {}", err);
			std::process::exit(1);
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
	return parse_lines(buf_reader.lines());
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
		println!("usage: fixsrt [-nobak] SRTFILE [OUTFILE]");
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

	let out_file_path = match args.next() {
		Some(arg) => arg,
		None => file_path.clone() // Same as input file path
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

	let subtitles_res = load_subtitles(&app_args.file_path);
	match subtitles_res {
		Ok(ref subtitles) => println!("{} subtitles", subtitles.len()),
		Err(ref err) => println!("{}", err)
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
	save_subtitles(&subtitles, &app_args.out_file_path);
}
