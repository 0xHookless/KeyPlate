use vault_core::{create, open_with_key, read_header, seal, unlock, wrap, Entry, EntryKind, Error, KdfParams, Payload};

/// Small parameters so the suite runs fast. Production vaults use KdfParams::DEFAULT or stronger.
const FAST: KdfParams = KdfParams { m_kib: 8 * 1024, t: 1, p: 1 };
const P1: &str = "copper-lantern-orbit-mesh-violin";
const P2: &str = "quiet-harbor-salt-ember-canyon";

fn entry(id: &str, kind: EntryKind, title: &str, updated: i64) -> Entry {
    Entry {
        id: id.into(),
        kind,
        title: title.into(),
        words: if kind == EntryKind::Seed { vec!["abandon".into(); 11].into_iter().chain(["about".into()]).collect() } else { vec![] },
        bip39_passphrase: None,
        network: None,
        body: String::new(),
        created: 1,
        updated,
        deleted: false,
    }
}

fn payload(n: usize) -> Payload {
    let mut p = Payload::new("device-a");
    for i in 0..n {
        p.entries.push(entry(&format!("id-{i}"), EntryKind::Seed, &format!("Wallet {i}"), 10));
    }
    p
}

fn vault_file(p: &Payload) -> Vec<u8> {
    let (env, key) = create(P1, P2, FAST).unwrap();
    seal(&env, &key, p).unwrap()
}

#[test]
fn round_trip() {
    let p = payload(3);
    let file = vault_file(&p);
    let opened = unlock(&file, P1, P2).unwrap();
    assert_eq!(opened.payload, p);
}

#[test]
fn wrong_or_swapped_passwords_fail() {
    let file = vault_file(&payload(1));
    assert_eq!(unlock(&file, "wrong", P2).unwrap_err(), Error::WrongPasswords);
    assert_eq!(unlock(&file, P1, "wrong").unwrap_err(), Error::WrongPasswords);
    assert_eq!(unlock(&file, P2, P1).unwrap_err(), Error::WrongPasswords, "order must matter");
}

#[test]
fn split_point_matters() {
    let (env, key) = create("abcdef-ghij", "klmnop", FAST).unwrap();
    let file = seal(&env, &key, &payload(0)).unwrap();
    assert!(unlock(&file, "abcdef-ghij", "klmnop").is_ok());
    assert!(unlock(&file, "abcdef-ghijk", "lmnop").is_err());
    assert!(unlock(&file, "abcdef-ghijklmnop", "").is_err());
}

#[test]
fn unicode_normalisation_is_applied() {
    let composed = "caf\u{e9}-m\u{f6}we-lantern";
    let decomposed = "cafe\u{301}-mo\u{308}we-lantern";
    let (env, key) = create(composed, P2, FAST).unwrap();
    let file = seal(&env, &key, &payload(0)).unwrap();
    assert!(unlock(&file, decomposed, P2).is_ok());
}

#[test]
fn identical_passwords_are_refused() {
    assert_eq!(create(P1, P1, FAST).unwrap_err(), Error::PasswordsMustDiffer);
}

#[test]
fn every_envelope_byte_is_authenticated() {
    let file = vault_file(&payload(1));
    for i in 0..128 {
        let mut bad = file.clone();
        bad[i] ^= 0x01;
        assert!(unlock(&bad, P1, P2).is_err(), "flipping envelope byte {i} was not detected");
    }
}

#[test]
fn every_payload_byte_is_authenticated() {
    let file = vault_file(&payload(1));
    let key = unlock(&file, P1, P2).unwrap().key;
    for i in 128..file.len() {
        let mut bad = file.clone();
        bad[i] ^= 0x80;
        assert!(open_with_key(&bad, &key).is_err(), "flipping payload byte {i} was not detected");
    }
}

#[test]
fn truncated_or_extended_files_fail() {
    let file = vault_file(&payload(1));
    let key = unlock(&file, P1, P2).unwrap().key;
    for cut in [0, 7, 56, 127, 128, 151, 152, file.len() - 1] {
        assert!(open_with_key(&file[..cut], &key).is_err(), "truncated at {cut}");
    }
    let mut longer = file.clone();
    longer.push(0);
    assert!(open_with_key(&longer, &key).is_err());
}

#[test]
fn kdf_params_come_from_the_header() {
    let custom = KdfParams { m_kib: 16 * 1024, t: 2, p: 2 };
    let (env, key) = create(P1, P2, custom).unwrap();
    let file = seal(&env, &key, &payload(0)).unwrap();
    assert_eq!(read_header(&file).unwrap().params, custom);
    assert!(unlock(&file, P1, P2).is_ok());
}

#[test]
fn hostile_params_are_refused_before_any_work() {
    let mut file = vault_file(&payload(0));
    file[11..15].copy_from_slice(&u32::MAX.to_be_bytes());
    assert_eq!(unlock(&file, P1, P2).unwrap_err(), Error::BadParams);
}

#[test]
fn not_a_vault_and_future_versions() {
    assert_eq!(unlock(b"hello", P1, P2).unwrap_err(), Error::NotAVault);
    let mut file = vault_file(&payload(0));
    file[9] = 9;
    assert_eq!(unlock(&file, P1, P2).unwrap_err(), Error::UnsupportedVersion(9));
}

#[test]
fn padding_hides_the_entry_count() {
    assert_eq!(vault_file(&payload(0)).len(), vault_file(&payload(8)).len());
}

#[test]
fn every_save_uses_a_fresh_nonce() {
    let (env, key) = create(P1, P2, FAST).unwrap();
    let p = payload(1);
    assert_ne!(seal(&env, &key, &p).unwrap(), seal(&env, &key, &p).unwrap());
}

#[test]
fn changing_passwords_keeps_the_vault_key() {
    let p = payload(2);
    let file = vault_file(&p);
    let old = unlock(&file, P1, P2).unwrap();
    let new_env = wrap(&old.key, "new-first-pass", "new-second-pass", FAST).unwrap();
    let new_file = seal(&new_env, &old.key, &p).unwrap();
    assert_eq!(unlock(&new_file, "new-first-pass", "new-second-pass").unwrap().payload, p);
    assert!(unlock(&new_file, P1, P2).is_err());
    // The old file still opens with the in-memory key, which is what lets other PCs merge.
    assert!(open_with_key(&file, &old.key).is_ok());
}

#[test]
fn a_different_vault_does_not_open_with_this_key() {
    let mine = unlock(&vault_file(&payload(1)), P1, P2).unwrap();
    let other = vault_file(&payload(1));
    assert_eq!(open_with_key(&other, &mine.key).unwrap_err(), Error::Corrupted);
}

#[test]
#[ignore = "timing probe for the production parameters; run with --ignored --nocapture"]
fn default_params_timing() {
    let start = std::time::Instant::now();
    let (env, key) = create(P1, P2, KdfParams::DEFAULT).unwrap();
    let file = seal(&env, &key, &payload(0)).unwrap();
    println!("create: {:?}", start.elapsed());
    let start = std::time::Instant::now();
    unlock(&file, P1, P2).unwrap();
    println!("unlock: {:?}", start.elapsed());
    println!("calibrated: {:?}", vault_core::kdf::calibrate(std::time::Duration::from_millis(1200)));
}
