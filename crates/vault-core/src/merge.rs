//! Combines two versions of a vault, e.g. edits made on two PCs before Drive synced.
//!
//! Entries match by id and the most recently updated copy wins. Deletes are tombstones, so a
//! delete on one PC beats an older edit on the other. Ties break deterministically, so every
//! device that merges the same two versions ends up with the same result.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::model::{Entry, Payload, SCHEMA};

fn compare(a: &Entry, b: &Entry) -> Ordering {
    a.updated
        .cmp(&b.updated)
        .then(a.deleted.cmp(&b.deleted))
        .then_with(|| {
            (&a.title, &a.words, &a.bip39_passphrase, &a.network, &a.body)
                .cmp(&(&b.title, &b.words, &b.bip39_passphrase, &b.network, &b.body))
        })
}

/// `device_id` comes from `local`; `rev` and `key_epoch` are the higher of the two.
pub fn merge(local: &Payload, remote: &Payload) -> Payload {
    let mut winners: BTreeMap<&str, &Entry> = BTreeMap::new();
    for entry in local.entries.iter().chain(&remote.entries) {
        winners
            .entry(entry.id.as_str())
            .and_modify(|current| {
                if compare(entry, current) == Ordering::Greater {
                    *current = entry;
                }
            })
            .or_insert(entry);
    }
    let mut entries: Vec<Entry> = winners.into_values().cloned().collect();
    entries.sort_by(|a, b| a.created.cmp(&b.created).then_with(|| a.id.cmp(&b.id)));

    Payload {
        schema: SCHEMA,
        rev: local.rev.max(remote.rev),
        device_id: local.device_id.clone(),
        key_epoch: local.key_epoch.max(remote.key_epoch),
        entries,
    }
}
