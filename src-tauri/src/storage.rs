//! Where vault files live and how they are written.
//!
//! The vault folder is normally inside Google Drive (`My Drive\Keyplate`) and synced by Drive for
//! desktop. Keyplate itself never talks to the network: it only reads and writes files here.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const VAULT_FILE: &str = "vault.keyplate";
const TMP_FILE: &str = "vault.keyplate.tmp";
const VERSIONS_DIR: &str = "versions";
const EXTENSION: &str = "keyplate";
const KEEP_VERSIONS: usize = 30;

#[derive(Debug, Clone)]
pub struct Storage {
    pub folder: PathBuf,
    /// Encrypted copy of the last save, so the vault still opens when Drive is offline.
    pub cache: PathBuf,
}

/// Size and modification time, used to notice changes without reading the whole file.
pub type FileStamp = (u64, SystemTime);

impl Storage {
    pub fn new(folder: PathBuf, cache: PathBuf) -> Self {
        Self { folder, cache }
    }

    pub fn vault_path(&self) -> PathBuf {
        self.folder.join(VAULT_FILE)
    }

    pub fn versions_dir(&self) -> PathBuf {
        self.folder.join(VERSIONS_DIR)
    }

    pub fn vault_exists(&self) -> bool {
        self.vault_path().is_file()
    }

    pub fn read_vault(&self) -> io::Result<Vec<u8>> {
        fs::read(self.vault_path())
    }

    pub fn stamp(&self) -> Option<FileStamp> {
        let meta = fs::metadata(self.vault_path()).ok()?;
        Some((meta.len(), meta.modified().ok()?))
    }

    pub fn read_cache(&self) -> Option<Vec<u8>> {
        fs::read(&self.cache).ok()
    }

    /// Saves a new vault: keeps the previous file in `versions\`, then swaps the new one in
    /// atomically so a crash or power cut can never leave a half-written vault.
    pub fn write(&self, bytes: &[u8]) -> io::Result<()> {
        fs::create_dir_all(&self.folder)?;
        let target = self.vault_path();
        if target.is_file() {
            let versions = self.versions_dir();
            fs::create_dir_all(&versions)?;
            fs::copy(&target, unique_path(&versions, &format!("vault-{}", timestamp())))?;
            prune_versions(&versions);
        }
        write_atomic(&self.folder.join(TMP_FILE), &target, bytes)?;
        if let Some(dir) = self.cache.parent() {
            // The offline copy is a convenience; a failure here must not fail the save.
            let _ = fs::create_dir_all(dir);
            let _ = write_atomic(&self.cache.with_extension("tmp"), &self.cache, bytes);
        }
        Ok(())
    }

    /// Other `.keyplate` files in the folder, such as `vault (1).keyplate` left by a Drive
    /// conflict. They are merged in and then moved to `versions\`.
    pub fn duplicates(&self) -> Vec<PathBuf> {
        let Ok(dir) = fs::read_dir(&self.folder) else { return Vec::new() };
        dir.filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_file())
            .filter(|p| p.extension().is_some_and(|x| x == EXTENSION))
            .filter(|p| p.file_name().is_some_and(|n| n != VAULT_FILE))
            .collect()
    }

    pub fn archive_duplicate(&self, path: &Path) -> io::Result<()> {
        let versions = self.versions_dir();
        fs::create_dir_all(&versions)?;
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("vault");
        fs::rename(path, unique_path(&versions, &format!("merged-{}-{}", timestamp(), sanitize(stem))))
    }

    pub fn export_to(&self, dest: &Path, bytes: &[u8]) -> io::Result<()> {
        let tmp = dest.with_extension("keyplate.tmp");
        write_atomic(&tmp, dest, bytes)
    }
}

fn write_atomic(tmp: &Path, target: &Path, bytes: &[u8]) -> io::Result<()> {
    {
        let mut f = fs::File::create(tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    // On Windows this replaces the target in one step.
    fs::rename(tmp, target)
}

fn sanitize(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect()
}

fn unique_path(dir: &Path, stem: &str) -> PathBuf {
    let mut path = dir.join(format!("{stem}.{EXTENSION}"));
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("{stem}-{n}.{EXTENSION}"));
        n += 1;
    }
    path
}

fn prune_versions(dir: &Path) {
    let Ok(read) = fs::read_dir(dir) else { return };
    let mut files: Vec<PathBuf> = read
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("vault-")))
        .collect();
    // Names start with a sortable UTC timestamp, so name order is age order.
    files.sort();
    let excess = files.len().saturating_sub(KEEP_VERSIONS);
    for old in &files[..excess] {
        let _ = fs::remove_file(old);
    }
}

/// UTC time as `YYYYMMDD-HHMMSS`.
pub fn timestamp() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}{m:02}{d:02}-{:02}{:02}{:02}", rem / 3600, rem % 3600 / 60, rem % 60)
}

/// Howard Hinnant's days-to-civil algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

/// Likely Google Drive folders on this PC, each with `Keyplate` appended.
pub fn drive_suggestions() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    // Drive for desktop in "Stream files" mode mounts a virtual drive, G: by default.
    for letter in b'D'..=b'Z' {
        let drive = format!("{}:\\", letter as char);
        for name in ["My Drive", "Mi unidad", "Meine Ablage", "Mon Drive"] {
            let p = Path::new(&drive).join(name);
            if p.is_dir() {
                roots.push(p);
            }
        }
    }
    // "Mirror files" mode keeps a normal folder in the user profile.
    if let Some(home) = std::env::var_os("USERPROFILE") {
        for name in ["My Drive", "Google Drive\\My Drive", "Google Drive"] {
            let p = Path::new(&home).join(name);
            if p.is_dir() && !roots.contains(&p) {
                roots.push(p);
            }
        }
    }
    roots.into_iter().map(|r| r.join("Keyplate")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_759), (2026, 11, 2));
        assert_eq!(civil_from_days(11_016), (2000, 2, 29));
    }

    #[test]
    fn writes_keep_versions_and_cache() {
        let root = std::env::temp_dir().join(format!("keyplate-test-{}", timestamp()));
        let _ = fs::remove_dir_all(&root);
        let s = Storage::new(root.join("vault"), root.join("cache").join("cache.keyplate"));
        s.write(b"one").unwrap();
        s.write(b"two").unwrap();
        s.write(b"three").unwrap();
        assert_eq!(s.read_vault().unwrap(), b"three");
        assert_eq!(s.read_cache().unwrap(), b"three");
        assert_eq!(fs::read_dir(s.versions_dir()).unwrap().count(), 2);
        assert!(!s.folder.join(TMP_FILE).exists());

        fs::write(s.folder.join("vault (1).keyplate"), b"dup").unwrap();
        let dups = s.duplicates();
        assert_eq!(dups.len(), 1);
        s.archive_duplicate(&dups[0]).unwrap();
        assert!(s.duplicates().is_empty());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn keeps_only_the_newest_versions() {
        let root = std::env::temp_dir().join(format!("keyplate-prune-{}", timestamp()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        for i in 0..(KEEP_VERSIONS + 5) {
            fs::write(root.join(format!("vault-20260101-0000{i:02}.keyplate")), b"x").unwrap();
        }
        prune_versions(&root);
        let mut left: Vec<String> =
            fs::read_dir(&root).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
        left.sort();
        assert_eq!(left.len(), KEEP_VERSIONS);
        assert_eq!(left[0], "vault-20260101-000005.keyplate");
        fs::remove_dir_all(&root).unwrap();
    }
}
