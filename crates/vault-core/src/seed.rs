//! BIP39 checks for typed seed phrases: catches typos and wrong words before they are saved.
//! Keyplate never derives keys from a seed and never touches funds.

use bip39::{Language, Mnemonic};
use serde::Serialize;

pub const VALID_COUNTS: [usize; 5] = [12, 15, 18, 21, 24];
const ENGLISH: Language = Language::English;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum SeedCheck {
    /// All words are known and the checksum matches.
    Valid,
    /// Some cells are empty, or the word count is not 12/15/18/21/24.
    #[serde(rename_all = "camelCase")]
    Incomplete { filled: usize },
    /// Words that are not in the BIP39 English list, with likely corrections.
    UnknownWords { words: Vec<UnknownWord> },
    /// Every word is real but the phrase is not a valid BIP39 mnemonic: usually a swapped or
    /// mistyped-but-valid word.
    BadChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnknownWord {
    pub index: usize,
    pub suggestions: Vec<&'static str>,
}

pub fn normalize(word: &str) -> String {
    word.trim().to_lowercase()
}

pub fn is_word(word: &str) -> bool {
    ENGLISH.find_word(&normalize(word)).is_some()
}

pub fn wordlist() -> &'static [&'static str; 2048] {
    ENGLISH.word_list()
}

fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1; b.len() + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != *cb);
            cur[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Words that start with what was typed, or failing that, the closest words by spelling.
pub fn suggest(typed: &str, max: usize) -> Vec<&'static str> {
    let typed = normalize(typed);
    if typed.is_empty() {
        return Vec::new();
    }
    let by_prefix = ENGLISH.words_by_prefix(&typed);
    if !by_prefix.is_empty() {
        return by_prefix.iter().take(max).copied().collect();
    }
    let mut close: Vec<(usize, &'static str)> = wordlist()
        .iter()
        .map(|w| (edit_distance(&typed, w), *w))
        .filter(|(d, _)| *d <= 2)
        .collect();
    close.sort();
    close.into_iter().take(max).map(|(_, w)| w).collect()
}

pub fn check(words: &[String]) -> SeedCheck {
    let norm: Vec<String> = words.iter().map(|w| normalize(w)).collect();
    let unknown: Vec<UnknownWord> = norm
        .iter()
        .enumerate()
        .filter(|(_, w)| !w.is_empty() && ENGLISH.find_word(w).is_none())
        .map(|(index, w)| UnknownWord { index, suggestions: suggest(w, 4) })
        .collect();
    if !unknown.is_empty() {
        return SeedCheck::UnknownWords { words: unknown };
    }
    let filled = norm.iter().filter(|w| !w.is_empty()).count();
    if filled != norm.len() || !VALID_COUNTS.contains(&filled) {
        return SeedCheck::Incomplete { filled };
    }
    match Mnemonic::parse_in_normalized(ENGLISH, &norm.join(" ")) {
        Ok(_) => SeedCheck::Valid,
        Err(bip39::Error::InvalidChecksum) => SeedCheck::BadChecksum,
        Err(_) => SeedCheck::Incomplete { filled },
    }
}
