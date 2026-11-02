use vault_core::{merge, Entry, EntryKind, Payload};

fn note(id: &str, body: &str, created: i64, updated: i64) -> Entry {
    Entry {
        id: id.into(),
        kind: EntryKind::Note,
        title: format!("Note {id}"),
        words: vec![],
        bip39_passphrase: None,
        network: None,
        body: body.into(),
        created,
        updated,
        deleted: false,
    }
}

fn vault(device: &str, rev: u64, entries: Vec<Entry>) -> Payload {
    let mut p = Payload::new(device);
    p.rev = rev;
    p.entries = entries;
    p
}

fn bodies(p: &Payload) -> Vec<(String, String, bool)> {
    p.entries.iter().map(|e| (e.id.clone(), e.body.clone(), e.deleted)).collect()
}

#[test]
fn edits_to_different_entries_are_both_kept() {
    let a = vault("a", 3, vec![note("1", "base", 1, 1), note("2", "from a", 2, 5)]);
    let b = vault("b", 4, vec![note("1", "base", 1, 1), note("3", "from b", 3, 6)]);
    let m = merge(&a, &b);
    assert_eq!(m.entries.len(), 3);
    assert_eq!(m.rev, 4);
    assert_eq!(m.device_id, "a");
}

#[test]
fn newer_edit_wins() {
    let a = vault("a", 1, vec![note("1", "older", 1, 10)]);
    let b = vault("b", 1, vec![note("1", "newer", 1, 20)]);
    assert_eq!(merge(&a, &b).entries[0].body, "newer");
    assert_eq!(merge(&b, &a).entries[0].body, "newer");
}

#[test]
fn delete_beats_an_older_edit_and_loses_to_a_newer_one() {
    let mut deleted = note("1", "secret", 1, 10);
    deleted.make_tombstone(30);
    assert!(deleted.body.is_empty() && deleted.title.is_empty(), "tombstones carry no secrets");

    let older_edit = vault("a", 1, vec![note("1", "edit", 1, 20)]);
    let with_delete = vault("b", 1, vec![deleted.clone()]);
    assert!(merge(&older_edit, &with_delete).entries[0].deleted);

    let newer_edit = vault("a", 1, vec![note("1", "edit", 1, 40)]);
    assert!(!merge(&newer_edit, &with_delete).entries[0].deleted);
}

#[test]
fn ties_resolve_the_same_way_on_every_device() {
    let a = vault("a", 1, vec![note("1", "alpha", 1, 10), note("2", "x", 2, 10)]);
    let b = vault("b", 1, vec![note("1", "bravo", 1, 10), note("2", "x", 2, 10)]);
    assert_eq!(bodies(&merge(&a, &b)), bodies(&merge(&b, &a)));
}

#[test]
fn the_newest_key_epoch_is_kept() {
    let mut a = vault("a", 1, vec![]);
    a.key_epoch = 2;
    let mut b = vault("b", 1, vec![]);
    b.key_epoch = 5;
    assert_eq!(merge(&a, &b).key_epoch, 5);
}
