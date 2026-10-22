//! Spanish suggestions. The ranking is the dictionary-driven one Czech uses
//! ([`crate::cs::suggest::suggest_in`]: case, MAP, REP, missing spaces, KEY, TRY edits and a stem
//! scan); every table comes from the installed `es_ES` dictionary, so nothing Spanish is
//! hard-coded here.

use crate::affix::AffixDict;

/// Spelling suggestions for a Spanish word, best first, at most `max`, in the word's case.
pub fn suggest(word: &str, max: usize) -> Vec<String> {
    suggest_in(&super::dictionary(), word, max)
}

/// [`suggest`] against a given dictionary.
pub fn suggest_in(d: &AffixDict, word: &str, max: usize) -> Vec<String> {
    crate::cs::suggest::suggest_in(d, word, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spanish_suggestions_use_runtime_tables_and_case() {
        crate::es::test_dict::install();
        for (word, want) in [("csa", "casa"), ("nino", "niño"), ("escueal", "escuela"), ("libor", "libro")] {
            assert!(suggest(word, 8).contains(&want.into()), "{word}: {:?}", suggest(word, 8));
        }
    }

    #[test]
    fn suggestions_stay_bounded_and_skip_forbidden_words() {
        crate::es::test_dict::install();
        for w in ["", "dos palabras", &"a".repeat(41)] {
            assert!(suggest(w, 4).is_empty());
        }
        assert!(suggest("csa", 0).is_empty());
        assert!(suggest_in(&AffixDict::default(), "csa", 4).is_empty());
        assert!(!suggest("casa", 4).contains(&"casa".into()));
    }
}
