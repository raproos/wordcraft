//! Spanish grammar: generic repeated-word and spacing checks, plus the opening question (¿) and
//! exclamation (¡) marks Spanish puts before a sentence — Word's "falta el signo de apertura".
//! A word written twice with a space is flagged, and the single word is the only suggestion:
//! unlike Indonesian, Spanish does not reduplicate by writing the word twice.

use crate::{Issue, IssueKind, ProofLang, words};

fn issue(start: usize, end: usize, message: String, suggestions: Vec<String>) -> Issue {
    Issue { start, end, kind: IssueKind::Grammar, message, suggestions, lang: ProofLang::Es }
}

pub fn check(text: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    repeated_words(text, &words(text), &mut issues);
    spaces(text, &mut issues);
    opening_marks(text, &mut issues);
    issues.sort_by_key(|i| (i.start, i.end));
    issues
}

/// Spanish opens a question with `¿` and an exclamation with `¡`. A sentence that closes with one
/// of those marks without having opened with it gets the mark suggested at its start.
fn opening_marks(text: &str, v: &mut Vec<Issue>) {
    let mut start = 0usize;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let (open, close, name) = match c {
            '?' => ('¿', '?', "«¿»"),
            '!' => ('¡', '!', "«¡»"),
            '.' | '…' => {
                start = i + c.len_utf8();
                continue;
            }
            _ => continue,
        };
        // Only a sentence-final mark counts: `?!` and `!?` are one ending, and a digit before the
        // dot of an ordinal is not a sentence break.
        if chars.peek().is_some_and(|(_, n)| *n == '?' || *n == '!') {
            continue;
        }
        let sentence = text.get(start..i).unwrap_or("");
        if sentence.contains(open) || sentence.contains(close) {
            start = i + c.len_utf8();
            continue;
        }
        // An abbreviation or an emoticon is not a sentence that needs an opening mark.
        let words_in_sentence = sentence.split_whitespace().count();
        if words_in_sentence > 0 {
            v.push(issue(i, i + c.len_utf8(), format!("Falta el signo de apertura {name}"), vec![open.to_string()]));
        }
        start = i + c.len_utf8();
    }
}

fn repeated_words(text: &str, ws: &[(usize, usize)], v: &mut Vec<Issue>) {
    for pair in ws.windows(2) {
        let [(a0, b0), (a1, b1)] = pair else { continue };
        let (Some(w0), Some(w1), Some(gap)) = (text.get(*a0..*b0), text.get(*a1..*b1), text.get(*b0..*a1)) else { continue };
        if !gap.is_empty() && gap.chars().all(char::is_whitespace) && w0.chars().all(char::is_alphabetic) && w0.to_lowercase() == w1.to_lowercase() {
            v.push(issue(*a0, *b1, format!("Palabra repetida: “{w1}”"), vec![w0.to_string()]));
        }
    }
}

fn spaces(text: &str, v: &mut Vec<Issue>) {
    for (i, c) in text.char_indices() {
        if c != ' ' {
            continue;
        }
        let rest = text.get(i + 1..).unwrap_or("");
        let mut it = rest.chars();
        let Some(p) = it.next() else { continue };
        let after = it.next();
        // A ratio/score keeps the spaces in digit-space-colon-space-digit.
        let ratio = p == ':'
            && text.get(..i).and_then(|before| before.chars().next_back()).is_some_and(|c| c.is_ascii_digit())
            && after == Some(' ')
            && it.next().is_some_and(|c| c.is_ascii_digit());
        let flag = !ratio
            && match p {
                ',' => true,
                // `;)` and `;-)` are emoticons.
                ';' => !matches!(after, Some(')' | '(' | '-' | 'P' | 'D')),
                // ` ...` is an ellipsis, ` .5` a number.
                '.' => !matches!(after, Some('.')) && !after.is_some_and(|a| a.is_ascii_digit()),
                // ` :)`, ` :-)`, ` :D` and similar are emoticons; ` ?!` is still a stray space.
                ':' | '!' | '?' => after.is_none_or(|a| a.is_whitespace() || matches!(a, '"' | '”' | '’' | '!' | '?')),
                // Spanish opens a question or exclamation with its own mark, and a space may follow it.
                '¿' | '¡' => false,
                _ => false,
            };
        if flag {
            v.push(issue(i, i + 1 + p.len_utf8(), "Espacio antes de la puntuación".into(), vec![p.to_string()]));
        }
    }
    for (i, _) in text.match_indices("  ") {
        if i > 0 && text.get(..i).is_some_and(|t| t.ends_with(' ')) {
            continue;
        }
        v.push(issue(i, i + 2, "Espacio doble".into(), vec![" ".into()]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_words_and_spaces_have_spanish_messages() {
        let text = "El el  ,  colegio.";
        let issues = check(text);
        assert!(issues.iter().any(|i| i.message == "Palabra repetida: “el”"), "{issues:?}");
        assert!(issues.iter().any(|i| i.message == "Espacio antes de la puntuación" && &text[i.start..i.end] == " ,"));
        assert!(issues.iter().any(|i| i.message == "Espacio doble"));
        assert!(issues.iter().all(|i| i.lang == ProofLang::Es && i.kind == IssueKind::Grammar));
        for text in ["El perro corre", "Hola. Adiós", "Dr. García", "¿Qué tal?", "¡Hola! Amigo", "3 : 1", "Número .5 y puntos ..."] {
            assert!(check(text).is_empty(), "{text}: {:?}", check(text));
        }
    }

    #[test]
    fn questions_and_exclamations_keep_their_opening_mark() {
        for (text, mark) in [("Qué tal?", '¿'), ("Hola!", '¡'), ("Dónde está?", '¿')] {
            let issues = check(text);
            let hit = issues.iter().find(|i| i.suggestions == [mark.to_string()]);
            assert!(hit.is_some_and(|i| i.message.contains("signo de apertura")), "{text}: {issues:?}");
        }
        for text in ["¿Qué tal?", "¡Hola!", "¿Qué? ¡Bien!", "¿Y ahora?", "Dice: «¿qué?»", "3 : 1"] {
            assert!(check(text).is_empty(), "{text}: {:?}", check(text));
        }
    }

    proptest::proptest! {
        #[test]
        fn never_panics_and_ranges_are_valid(s in "\\PC{0,80}") {
            for issue in check(&s) {
                proptest::prop_assert!(issue.start <= issue.end && issue.end <= s.len());
                proptest::prop_assert!(s.is_char_boundary(issue.start) && s.is_char_boundary(issue.end));
            }
        }
    }
}
