//! The decrypted contents of a vault.

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

pub const SCHEMA: u32 = 1;

/// A random 128-bit id, hex-encoded.
pub fn new_id() -> crate::Result<String> {
    let mut raw = [0u8; 16];
    getrandom::getrandom(&mut raw).map_err(|_| crate::Error::Random)?;
    Ok(raw.iter().map(|b| format!("{b:02x}")).collect())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    Seed,
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    #[zeroize(skip)]
    pub kind: EntryKind,
    pub title: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub words: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bip39_passphrase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(default)]
    pub body: String,
    /// Unix milliseconds.
    pub created: i64,
    /// Unix milliseconds. Decides which side wins a merge.
    pub updated: i64,
    #[serde(default)]
    pub deleted: bool,
}

impl Entry {
    /// Replaces this entry with a tombstone that carries no secrets but still syncs the delete.
    pub fn make_tombstone(&mut self, now_ms: i64) {
        self.title.zeroize();
        self.words.zeroize();
        self.bip39_passphrase.zeroize();
        self.network.zeroize();
        self.body.zeroize();
        self.title = String::new();
        self.words = Vec::new();
        self.bip39_passphrase = None;
        self.network = None;
        self.body = String::new();
        self.updated = now_ms.max(self.updated + 1);
        self.deleted = true;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub schema: u32,
    /// Bumped on every save.
    pub rev: u64,
    /// The device that wrote this revision.
    pub device_id: String,
    /// Bumped on every password change, so the newest key envelope wins a merge.
    pub key_epoch: u64,
    pub entries: Vec<Entry>,
}

impl Payload {
    pub fn new(device_id: impl Into<String>) -> Self {
        Self { schema: SCHEMA, rev: 0, device_id: device_id.into(), key_epoch: 0, entries: Vec::new() }
    }

    pub fn live_entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(|e| !e.deleted)
    }

    pub fn find(&self, id: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.id == id && !e.deleted)
    }

    pub fn find_mut(&mut self, id: &str) -> Option<&mut Entry> {
        self.entries.iter_mut().find(|e| e.id == id && !e.deleted)
    }
}
