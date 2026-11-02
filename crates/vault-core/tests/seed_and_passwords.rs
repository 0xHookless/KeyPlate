use vault_core::passwords::{self, Problem};
use vault_core::seed::{self, SeedCheck};

fn words(s: &str) -> Vec<String> {
    s.split_whitespace().map(String::from).collect()
}

// Valid mnemonics from the Trezor BIP39 reference vectors.
const VECTORS: &[&str] = &[
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    "legal winner thank year wave sausage worth useful legal winner thank yellow",
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
    "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong",
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon \
     abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art",
    "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo vote",
];

#[test]
fn reference_vectors_are_valid() {
    for v in VECTORS {
        assert_eq!(seed::check(&words(v)), SeedCheck::Valid, "{v}");
    }
}

#[test]
fn case_and_spacing_are_forgiven() {
    let mut w = words(VECTORS[0]);
    w[0] = "  Abandon ".into();
    assert_eq!(seed::check(&w), SeedCheck::Valid);
}

#[test]
fn bad_checksum_is_caught() {
    let w = vec!["abandon".to_string(); 12];
    assert_eq!(seed::check(&w), SeedCheck::BadChecksum);
}

#[test]
fn unknown_words_get_suggestions() {
    let mut w = words(VECTORS[0]);
    w[11] = "abuot".into();
    match seed::check(&w) {
        SeedCheck::UnknownWords { words } => {
            assert_eq!(words[0].index, 11);
            assert!(words[0].suggestions.contains(&"about"), "{:?}", words[0].suggestions);
        }
        other => panic!("expected unknown words, got {other:?}"),
    }
    assert_eq!(seed::suggest("aban", 3), vec!["abandon"]);
}

#[test]
fn empty_cells_and_odd_counts_are_incomplete() {
    let mut w = words(VECTORS[0]);
    w[4].clear();
    assert_eq!(seed::check(&w), SeedCheck::Incomplete { filled: 11 });
    assert_eq!(seed::check(&words(VECTORS[0])[..11]), SeedCheck::Incomplete { filled: 11 });
}

#[test]
fn generated_passphrases_are_five_list_words() {
    let a = passwords::generate().unwrap();
    let b = passwords::generate().unwrap();
    assert_ne!(*a, *b);
    let parts: Vec<&str> = a.split('-').collect();
    assert_eq!(parts.len(), passwords::GENERATED_WORDS);
    assert!(parts.iter().all(|w| seed::is_word(w)));
}

#[test]
fn two_generated_passphrases_are_acceptable() {
    for _ in 0..20 {
        let a = passwords::generate().unwrap();
        let b = passwords::generate().unwrap();
        let s = passwords::estimate(&a, &b);
        assert!(s.acceptable, "{} / {}: {s:?}", a.as_str(), b.as_str());
    }
}

#[test]
fn weak_or_repeated_passwords_are_rejected() {
    let s = passwords::estimate("password", "123456");
    assert!(!s.acceptable);
    assert!(s.problems.contains(&Problem::FirstWeak) && s.problems.contains(&Problem::CombinedWeak));

    let same = passwords::estimate("copper-lantern-orbit", "copper-lantern-orbit");
    assert!(same.problems.contains(&Problem::Same));

    let empty = passwords::estimate("", "");
    assert!(empty.problems.contains(&Problem::FirstEmpty) && empty.problems.contains(&Problem::SecondEmpty));
}
