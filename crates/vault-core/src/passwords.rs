//! Password strength rules and the passphrase generator.

use serde::Serialize;
use zeroize::Zeroizing;

use crate::error::{Error, Result};
use crate::kdf::passwords_equal;
use crate::seed::wordlist;

pub const MIN_EACH_BITS: f64 = 30.0;
pub const MIN_COMBINED_BITS: f64 = 80.0;
/// Each generated word is one of 2048, chosen uniformly.
pub const BITS_PER_WORD: f64 = 11.0;
pub const GENERATED_WORDS: usize = 5;
/// zxcvbn gets slow on very long input; characters past this only make a password stronger.
const MAX_ESTIMATE_CHARS: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Problem {
    FirstEmpty,
    SecondEmpty,
    Same,
    FirstWeak,
    SecondWeak,
    CombinedWeak,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Strength {
    pub first_bits: f64,
    pub second_bits: f64,
    pub combined_bits: f64,
    pub acceptable: bool,
    pub problems: Vec<Problem>,
}

fn truncated(s: &str) -> Zeroizing<String> {
    Zeroizing::new(s.chars().take(MAX_ESTIMATE_CHARS).collect())
}

/// Estimated bits of one password, penalised when it reuses the other one.
fn bits(password: &str, other: &str) -> f64 {
    if password.is_empty() {
        return 0.0;
    }
    let password = truncated(password);
    let other = truncated(other);
    let inputs: Vec<&str> = if other.is_empty() { vec![] } else { vec![other.as_str()] };
    zxcvbn::zxcvbn(&password, &inputs).guesses_log10() * std::f64::consts::LOG2_10
}

pub fn estimate(p1: &str, p2: &str) -> Strength {
    let first_bits = bits(p1, p2);
    let second_bits = bits(p2, p1);
    let combined_bits = first_bits + second_bits;

    let mut problems = Vec::new();
    if p1.is_empty() {
        problems.push(Problem::FirstEmpty);
    }
    if p2.is_empty() {
        problems.push(Problem::SecondEmpty);
    }
    if !p1.is_empty() && passwords_equal(p1, p2) {
        problems.push(Problem::Same);
    }
    if !p1.is_empty() && first_bits < MIN_EACH_BITS {
        problems.push(Problem::FirstWeak);
    }
    if !p2.is_empty() && second_bits < MIN_EACH_BITS {
        problems.push(Problem::SecondWeak);
    }
    if combined_bits < MIN_COMBINED_BITS {
        problems.push(Problem::CombinedWeak);
    }
    Strength { first_bits, second_bits, combined_bits, acceptable: problems.is_empty(), problems }
}

/// A passphrase of `GENERATED_WORDS` BIP39 words joined by hyphens: 55 bits of real entropy.
pub fn generate() -> Result<Zeroizing<String>> {
    let list = wordlist();
    let mut out = Zeroizing::new(String::new());
    let mut raw = Zeroizing::new([0u8; 2 * GENERATED_WORDS]);
    getrandom::getrandom(raw.as_mut()).map_err(|_| Error::Random)?;
    for (i, pair) in raw.as_chunks::<2>().0.iter().enumerate() {
        // 2048 is a power of two, so masking to 11 bits is perfectly uniform.
        let index = (u16::from_be_bytes([pair[0], pair[1]]) & 0x07FF) as usize;
        if i > 0 {
            out.push('-');
        }
        out.push_str(list[index]);
    }
    Ok(out)
}
