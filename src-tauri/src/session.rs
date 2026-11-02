//! An unlocked vault: the key, the decrypted contents and the file they came from.
//! Dropping a Session wipes the key and contents from memory.

use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};
use vault_core::{merge, open_with_key, seal, unlock, wrap, Envelope, KdfParams, Payload, VaultKey};

use crate::error::AppError;
use crate::storage::{FileStamp, Storage};

pub struct Session {
    pub key: VaultKey,
    pub envelope: Envelope,
    pub payload: Payload,
    pub storage: Storage,
    /// Hash and stamp of the vault file as last read or written, to notice saves from other PCs.
    disk_hash: Option<[u8; 32]>,
    disk_stamp: Option<FileStamp>,
    /// True when Drive was unreachable at unlock and the offline copy was used.
    pub offline: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SyncOutcome {
    Unchanged,
    Updated,
    /// The vault file now belongs to a different vault; the caller must lock.
    ForeignVault,
}

pub fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

impl Session {
    /// Unlocks the vault in `storage`, falling back to the offline copy when Drive is unreachable,
    /// then folds in anything the offline copy has that Drive does not yet.
    pub fn open(storage: Storage, p1: &str, p2: &str, device_id: &str) -> Result<Self, AppError> {
        let (bytes, offline) = match storage.read_vault() {
            Ok(b) => (b, false),
            Err(_) => (storage.read_cache().ok_or(AppError::FolderUnavailable)?, true),
        };
        let opened = unlock(&bytes, p1, p2)?;
        let mut session = Session {
            key: opened.key,
            envelope: opened.envelope,
            payload: opened.payload,
            disk_hash: (!offline).then(|| sha256(&bytes)),
            disk_stamp: if offline { None } else { storage.stamp() },
            storage,
            offline,
        };
        session.payload.device_id = device_id.to_string();

        if let Some(cache) = session.storage.read_cache() {
            if sha256(&cache) != sha256(&bytes) {
                if let Ok((env, cached)) = open_with_key(&cache, &session.key) {
                    if session.absorb(env, cached) && !offline {
                        session.save()?;
                    }
                }
            }
        }
        Ok(session)
    }

    /// Merges another version in. Returns true if our copy gained anything the other lacked.
    fn absorb(&mut self, env: Envelope, other: Payload) -> bool {
        let merged = merge(&self.payload, &other);
        let other_is_complete = merged.entries == other.entries && other.key_epoch >= self.payload.key_epoch;
        if other.key_epoch > self.payload.key_epoch {
            // The other PC changed the passwords: adopt its envelope so we don't undo that.
            self.envelope = env;
        }
        self.payload = merged;
        !other_is_complete
    }

    pub fn save(&mut self) -> Result<(), AppError> {
        self.payload.rev += 1;
        let bytes = seal(&self.envelope, &self.key, &self.payload)?;
        self.storage.write(&bytes).map_err(|_| AppError::FolderUnavailable)?;
        self.disk_hash = Some(sha256(&bytes));
        self.disk_stamp = self.storage.stamp();
        self.offline = false;
        Ok(())
    }

    /// Picks up saves made on other PCs (and Drive conflict copies) and merges them in.
    ///
    /// Without `force`, the file is only read when its size or modification time changed. That
    /// check is cheap but not airtight: Windows file times tick every ~15 ms and padding keeps
    /// sizes equal, so callers force a full read before edits and every so often.
    pub fn sync_from_disk(&mut self, force: bool) -> Result<SyncOutcome, AppError> {
        let mut updated = false;
        let mut must_write = false;

        let stamp = self.storage.stamp();
        if stamp.is_none() && !self.storage.folder.is_dir() {
            // Drive is offline or the folder was unmounted; try again later.
            return Ok(SyncOutcome::Unchanged);
        }
        if stamp.is_none() {
            // The vault file was deleted or moved away: put it back from memory.
            must_write = true;
        } else if force || stamp != self.disk_stamp {
            if let Ok(bytes) = self.storage.read_vault() {
                let hash = sha256(&bytes);
                if Some(hash) != self.disk_hash {
                    match open_with_key(&bytes, &self.key) {
                        Ok((env, remote)) => {
                            let before = self.payload.entries.clone();
                            must_write |= self.absorb(env, remote);
                            updated |= self.payload.entries != before;
                            self.disk_hash = Some(hash);
                        }
                        Err(_) => return Ok(SyncOutcome::ForeignVault),
                    }
                }
                self.disk_stamp = stamp;
            }
        }

        for dup in self.storage.duplicates() {
            let Ok(bytes) = std::fs::read(&dup) else { continue };
            if let Ok((env, other)) = open_with_key(&bytes, &self.key) {
                let before = self.payload.entries.clone();
                self.absorb(env, other);
                updated |= self.payload.entries != before;
                must_write = true;
                let _ = self.storage.archive_duplicate(&dup);
            }
        }

        if must_write {
            self.save()?;
        }
        Ok(if updated || must_write { SyncOutcome::Updated } else { SyncOutcome::Unchanged })
    }

    /// Re-wraps the same vault key under new passwords. Other PCs keep working: they adopt the
    /// new envelope the next time they see this file.
    pub fn change_passwords(&mut self, p1: &str, p2: &str, params: KdfParams) -> Result<(), AppError> {
        self.envelope = wrap(&self.key, p1, p2, params)?;
        self.payload.key_epoch += 1;
        self.save()
    }

    pub fn current_file(&self) -> Result<Vec<u8>, AppError> {
        Ok(seal(&self.envelope, &self.key, &self.payload)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vault_core::{Entry, EntryKind};

    const FAST: KdfParams = KdfParams { m_kib: 8 * 1024, t: 1, p: 1 };

    fn note(id: &str) -> Entry {
        Entry {
            id: id.into(),
            kind: EntryKind::Note,
            title: id.into(),
            words: vec![],
            bip39_passphrase: None,
            network: None,
            body: format!("body {id}"),
            created: now_ms(),
            updated: now_ms(),
            deleted: false,
        }
    }

    fn ids(s: &Session) -> Vec<String> {
        let mut v: Vec<String> = s.payload.live_entries().map(|e| e.id.clone()).collect();
        v.sort();
        v
    }

    #[test]
    fn two_pcs_stay_in_step() {
        let root = std::env::temp_dir().join(format!("keyplate-sync-{}", now_ms()));
        let folder = root.join("My Drive").join("Keyplate");
        let pc1 = Storage::new(folder.clone(), root.join("pc1").join("cache.keyplate"));
        let pc2 = Storage::new(folder.clone(), root.join("pc2").join("cache.keyplate"));

        let (env, key) = vault_core::create("first-pass-one", "second-pass-two", FAST).unwrap();
        pc1.write(&seal(&env, &key, &Payload::new("pc1")).unwrap()).unwrap();
        let mut s1 = Session::open(pc1, "first-pass-one", "second-pass-two", "pc1").unwrap();
        let mut s2 = Session::open(pc2, "first-pass-one", "second-pass-two", "pc2").unwrap();

        // PC1 adds an entry; PC2 sees it.
        s1.payload.entries.push(note("a"));
        s1.save().unwrap();
        assert_eq!(s2.sync_from_disk(true).unwrap(), SyncOutcome::Updated);
        assert_eq!(ids(&s2), ["a"]);
        assert_eq!(s2.sync_from_disk(true).unwrap(), SyncOutcome::Unchanged);

        // PC2 changes the passwords; PC1 adopts them instead of undoing the change.
        s2.change_passwords("new-first-pass", "new-second-pass", FAST).unwrap();
        s1.sync_from_disk(true).unwrap();
        s1.payload.entries.push(note("b"));
        s1.save().unwrap();
        let bytes = s1.storage.read_vault().unwrap();
        assert!(unlock(&bytes, "first-pass-one", "second-pass-two").is_err());
        let fresh = unlock(&bytes, "new-first-pass", "new-second-pass").unwrap();
        assert_eq!(fresh.payload.live_entries().count(), 2);

        // Drive left a conflict copy holding an entry the main file lacks: it is merged and archived.
        s2.sync_from_disk(true).unwrap();
        let mut other = s2.payload.clone();
        other.entries.push(note("c"));
        std::fs::write(folder.join("vault (1).keyplate"), seal(&s2.envelope, &s2.key, &other).unwrap()).unwrap();
        assert_eq!(s1.sync_from_disk(true).unwrap(), SyncOutcome::Updated);
        assert_eq!(ids(&s1), ["a", "b", "c"]);
        assert!(s1.storage.duplicates().is_empty());

        // A deleted vault file is restored from memory.
        std::fs::remove_file(s1.storage.vault_path()).unwrap();
        s1.sync_from_disk(true).unwrap();
        assert!(s1.storage.vault_exists());

        // A different vault dropped in its place must not be overwritten.
        let (env, key) = vault_core::create("someone-else-1", "someone-else-2", FAST).unwrap();
        std::fs::write(s1.storage.vault_path(), seal(&env, &key, &Payload::new("x")).unwrap()).unwrap();
        assert_eq!(s1.sync_from_disk(true).unwrap(), SyncOutcome::ForeignVault);

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn offline_copy_opens_when_drive_is_unreachable() {
        let root = std::env::temp_dir().join(format!("keyplate-offline-{}", now_ms()));
        let storage = Storage::new(root.join("drive"), root.join("cache.keyplate"));
        let (env, key) = vault_core::create("first-pass-one", "second-pass-two", FAST).unwrap();
        let mut p = Payload::new("pc1");
        p.entries.push(note("a"));
        storage.write(&seal(&env, &key, &p).unwrap()).unwrap();

        std::fs::remove_dir_all(root.join("drive")).unwrap();
        let s = Session::open(storage, "first-pass-one", "second-pass-two", "pc1").unwrap();
        assert!(s.offline);
        assert_eq!(ids(&s), ["a"]);
        std::fs::remove_dir_all(&root).unwrap();
    }
}
