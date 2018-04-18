use rules;

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
pub fn replace_one(text: &str, language: &str) -> String {
	let mut result = text.to_string();

	let mut rule_set = rules::RULES_FR;
	if language == "EN" {
		rule_set = rules::RULES_EN;
	}

	for rule_ref in rule_set.iter() {
		let &(what, with) = rule_ref;
		result = replace_by_rule(&result, what, with);
	}
	result
}

// For unit testing
#[allow(dead_code)]
pub fn replace_one_fr(text: &str) -> String {
	replace_one(text, "FR")
}

#[test]
fn test_replace_one_fr() {
	assert_eq!(replace_one_fr("Ca va"), "Ça va");
	assert_eq!(replace_one_fr("Ca."), "Ça.");
	assert_eq!(replace_one_fr("Ca"), "Ça");
	assert!(replace_one_fr("Caribou") != "Çaribou");
	assert_eq!(replace_one_fr("oeizz"), "œizz");
	assert_eq!(replace_one_fr("des soeurs."), "des sœurs.");

	// No space if newline
	assert_eq!(replace_one_fr("D'ou sa"), "D'où sa");
	assert_eq!(replace_one_fr("bien sur,"), "bien sûr,");

	assert_eq!(replace_one_fr("bien sur,"), "bien sûr,");
	assert_eq!(replace_one_fr("Ecart entre"), "Écart entre");
	assert_eq!(replace_one_fr("un coca"), "un coca");
	assert_eq!(replace_one_fr("l'Etat"), "l'État");
	assert_eq!(replace_one_fr("ok?"), "ok\u{A0}?");
	assert_eq!(replace_one_fr("ok ?"), "ok\u{A0}?");
	assert_eq!(replace_one_fr("ok!"), "ok\u{A0}!");
	assert_eq!(replace_one_fr("ok !"), "ok\u{A0}!");
	assert_eq!(replace_one_fr("A quoi?"), "À quoi\u{A0}?");
	assert_eq!(replace_one_fr("A t'écouter"), "À t'écouter");
	assert_eq!(replace_one_fr("manoeuvrer"), "manœuvrer");
	assert_eq!(replace_one_fr("Etiez-vous"), "Étiez-vous");
	assert_eq!(replace_one_fr("des qu'il"), "dès qu'il");
	assert_eq!(replace_one_fr("10ème"), "10ᵉ");
	assert_eq!(replace_one_fr("10e"), "10ᵉ");
	assert_eq!(replace_one_fr("10è"), "10ᵉ");
	assert_eq!(replace_one_fr("\"Oeil pour oeil\""), "\"Œil pour œil\"");
	assert_eq!(replace_one_fr("Etaient-ils"), "Étaient-ils");
	assert_eq!(replace_one_fr("caca"), "caca");
}
