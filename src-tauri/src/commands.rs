//! Everything the UI can ask for. The UI never sees the vault key or the file contents; it gets
//! titles until you reveal an entry, and passwords go one way: in.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;
use vault_core::passwords::{self, Problem, Strength};
use vault_core::seed::{self, SeedCheck};
use vault_core::{kdf, Entry, EntryKind, Payload};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::config::{self, Config, Settings};
use crate::error::{AppError, AppResult};
use crate::session::{now_ms, Session, SyncOutcome};
use crate::storage::{self, Storage};

const KDF_TARGET: Duration = Duration::from_millis(1200);

pub struct AppState(Mutex<Inner>);

pub struct Inner {
    pub config: Config,
    pub session: Option<Session>,
    pub last_activity: Instant,
    pub clipboard: Option<(u32, Instant)>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self(Mutex::new(Inner { config, session: None, last_activity: Instant::now(), clipboard: None }))
    }

    pub fn lock(&self) -> MutexGuard<'_, Inner> {
        // A panic while holding the lock must not brick the app; the data is still consistent.
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_, Inner>> {
        self.0.try_lock().ok()
    }
}

impl Inner {
    fn storage(&self) -> AppResult<Storage> {
        let folder = self.config.vault_folder.clone().ok_or(AppError::NoVaultFolder)?;
        Ok(Storage::new(folder, config::cache_path()))
    }

    fn session(&mut self) -> AppResult<&mut Session> {
        self.last_activity = Instant::now();
        self.session.as_mut().ok_or(AppError::Locked)
    }

    /// Brings in other PCs' saves, applies `change`, then saves.
    fn mutate<T>(&mut self, change: impl FnOnce(&mut Payload) -> AppResult<T>) -> AppResult<T> {
        let outcome = self.session()?.sync_from_disk(true)?;
        if outcome == SyncOutcome::ForeignVault {
            self.session = None;
            return Err(AppError::Replaced);
        }
        let session = self.session()?;
        let out = change(&mut session.payload)?;
        session.save()?;
        Ok(out)
    }
}

// ---------- shapes sent to the UI ----------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    stage: &'static str,
    folder: Option<String>,
    offline: bool,
    practice_due: bool,
    settings: Settings,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntrySummary {
    id: String,
    kind: EntryKind,
    title: String,
    network: Option<String>,
    word_count: usize,
    has_passphrase: bool,
    has_body: bool,
    created: i64,
    updated: i64,
    checksum_valid: Option<bool>,
}

#[derive(Serialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct EntryDetail {
    id: String,
    #[zeroize(skip)]
    kind: EntryKind,
    title: String,
    words: Vec<String>,
    bip39_passphrase: Option<String>,
    network: Option<String>,
    body: String,
    created: i64,
    updated: i64,
}

#[derive(Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct EntryInput {
    id: Option<String>,
    #[zeroize(skip)]
    kind: EntryKind,
    title: String,
    #[serde(default)]
    words: Vec<String>,
    bip39_passphrase: Option<String>,
    network: Option<String>,
    #[serde(default)]
    body: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryInfo {
    folder: String,
    vault_file: String,
    installer_sha256: Option<String>,
    app_version: String,
    kdf: String,
}

fn summary(e: &Entry) -> EntrySummary {
    EntrySummary {
        id: e.id.clone(),
        kind: e.kind,
        title: e.title.clone(),
        network: e.network.clone(),
        word_count: e.words.len(),
        has_passphrase: e.bip39_passphrase.as_deref().is_some_and(|p| !p.is_empty()),
        has_body: !e.body.is_empty(),
        created: e.created,
        updated: e.updated,
        checksum_valid: (e.kind == EntryKind::Seed && !e.words.is_empty())
            .then(|| seed::check(&e.words) == SeedCheck::Valid),
    }
}

fn status_of(inner: &Inner) -> Status {
    let s = &inner.config.settings;
    let practice_due = s.practice_days > 0
        && inner.config.last_practice_ms.is_some_and(|t| now_ms() - t > i64::from(s.practice_days) * 86_400_000);
    Status {
        stage: if inner.session.is_some() {
            "unlocked"
        } else if inner.config.vault_folder.is_some() {
            "locked"
        } else {
            "welcome"
        },
        folder: inner.config.vault_folder.as_ref().map(|p| p.display().to_string()),
        offline: inner.session.as_ref().is_some_and(|s| s.offline),
        practice_due,
        settings: inner.config.settings.clone(),
    }
}

fn require_strong(first: &str, second: &str) -> AppResult<()> {
    let s = passwords::estimate(first, second);
    if s.problems.contains(&Problem::Same) {
        return Err(AppError::Vault(vault_core::Error::PasswordsMustDiffer));
    }
    if !s.acceptable {
        return Err(AppError::WeakPasswords);
    }
    Ok(())
}

fn kdf_label(params: kdf::KdfParams) -> String {
    format!("Argon2id · {} MiB · {} passes · {} lanes", params.m_kib / 1024, params.t, params.p)
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    tauri::async_runtime::spawn_blocking(f).await.expect("background task panicked")
}

/// Locks the vault and tells the UI why.
pub fn lock_now(app: &AppHandle, inner: &mut Inner, reason: &str) {
    if inner.session.take().is_some() {
        let _ = app.emit("vault-locked", reason);
    }
}

// ---------- commands ----------

#[tauri::command]
pub async fn status(state: State<'_, AppState>) -> AppResult<Status> {
    Ok(status_of(&state.lock()))
}

#[tauri::command]
pub async fn drive_suggestions() -> AppResult<Vec<String>> {
    Ok(storage::drive_suggestions().iter().map(|p| p.display().to_string()).collect())
}

#[tauri::command]
pub async fn pick_folder(app: AppHandle) -> AppResult<Option<String>> {
    let picked = blocking(move || app.dialog().file().set_title("Choose the vault folder").blocking_pick_folder()).await;
    Ok(picked.and_then(|p| p.into_path().ok()).map(|p| p.display().to_string()))
}

#[tauri::command]
pub async fn estimate_strength(first: String, second: String) -> AppResult<Strength> {
    let (first, second) = (Zeroizing::new(first), Zeroizing::new(second));
    Ok(passwords::estimate(&first, &second))
}

#[tauri::command]
pub async fn generate_passphrase() -> AppResult<String> {
    Ok(passwords::generate()?.to_string())
}

#[tauri::command]
pub async fn create_vault(state: State<'_, AppState>, folder: String, first: String, second: String) -> AppResult<Status> {
    let (first, second) = (Zeroizing::new(first), Zeroizing::new(second));
    require_strong(&first, &second)?;
    let storage = Storage::new(PathBuf::from(folder), config::cache_path());
    if storage.vault_exists() {
        return Err(AppError::VaultAlreadyHere);
    }
    let device_id = state.lock().config.device_id.clone();
    let session = blocking(move || -> AppResult<Session> {
        let params = kdf::calibrate(KDF_TARGET);
        let (envelope, key) = vault_core::create(&first, &second, params)?;
        let mut payload = Payload::new(device_id);
        payload.rev = 0;
        let bytes = vault_core::seal(&envelope, &key, &payload)?;
        storage.write(&bytes).map_err(|_| AppError::FolderUnavailable)?;
        Session::open(storage, &first, &second, &payload.device_id)
    })
    .await?;

    let mut inner = state.lock();
    inner.config.vault_folder = Some(session.storage.folder.clone());
    inner.config.last_practice_ms = Some(now_ms());
    config::save(&inner.config);
    inner.session = Some(session);
    inner.last_activity = Instant::now();
    Ok(status_of(&inner))
}

#[tauri::command]
pub async fn use_existing_vault(state: State<'_, AppState>, folder: String) -> AppResult<Status> {
    let storage = Storage::new(PathBuf::from(&folder), config::cache_path());
    let bytes = storage.read_vault().map_err(|_| AppError::NoVaultHere)?;
    vault_core::read_header(&bytes)?;
    let mut inner = state.lock();
    inner.session = None;
    inner.config.vault_folder = Some(storage.folder);
    inner.config.last_practice_ms = Some(now_ms());
    config::save(&inner.config);
    Ok(status_of(&inner))
}

#[tauri::command]
pub async fn forget_vault(app: AppHandle, state: State<'_, AppState>) -> AppResult<Status> {
    let mut inner = state.lock();
    lock_now(&app, &mut inner, "manual");
    inner.config.vault_folder = None;
    config::save(&inner.config);
    Ok(status_of(&inner))
}

#[tauri::command]
pub async fn unlock(state: State<'_, AppState>, first: String, second: String) -> AppResult<Status> {
    let (first, second) = (Zeroizing::new(first), Zeroizing::new(second));
    let (storage, device_id) = {
        let inner = state.lock();
        (inner.storage()?, inner.config.device_id.clone())
    };
    if !storage.vault_exists() && storage.read_cache().is_none() {
        return Err(if storage.folder.is_dir() { AppError::NoVaultHere } else { AppError::FolderUnavailable });
    }
    let session = blocking(move || Session::open(storage, &first, &second, &device_id)).await?;
    let mut inner = state.lock();
    inner.session = Some(session);
    inner.last_activity = Instant::now();
    Ok(status_of(&inner))
}

#[tauri::command]
pub async fn lock(app: AppHandle, state: State<'_, AppState>) -> AppResult<Status> {
    let mut inner = state.lock();
    lock_now(&app, &mut inner, "manual");
    Ok(status_of(&inner))
}

#[tauri::command]
pub async fn touch(state: State<'_, AppState>) -> AppResult<()> {
    state.lock().last_activity = Instant::now();
    Ok(())
}

#[tauri::command]
pub async fn list_entries(state: State<'_, AppState>) -> AppResult<Vec<EntrySummary>> {
    let mut inner = state.lock();
    let session = inner.session()?;
    let mut list: Vec<EntrySummary> = session.payload.live_entries().map(summary).collect();
    list.sort_by_key(|e| e.title.to_lowercase());
    Ok(list)
}

#[tauri::command]
pub async fn reveal_entry(state: State<'_, AppState>, id: String) -> AppResult<EntryDetail> {
    let mut inner = state.lock();
    let e = inner.session()?.payload.find(&id).ok_or(AppError::NotFound)?;
    Ok(EntryDetail {
        id: e.id.clone(),
        kind: e.kind,
        title: e.title.clone(),
        words: e.words.clone(),
        bip39_passphrase: e.bip39_passphrase.clone(),
        network: e.network.clone(),
        body: e.body.clone(),
        created: e.created,
        updated: e.updated,
    })
}

#[tauri::command]
pub async fn save_entry(state: State<'_, AppState>, input: EntryInput) -> AppResult<EntrySummary> {
    let now = now_ms();
    let title = match input.title.trim() {
        "" if input.kind == EntryKind::Seed => "Untitled seed phrase".to_string(),
        "" => "Untitled note".to_string(),
        t => t.to_string(),
    };
    let words: Vec<String> = if input.kind == EntryKind::Seed {
        input.words.iter().map(|w| seed::normalize(w)).filter(|w| !w.is_empty()).collect()
    } else {
        Vec::new()
    };
    let passphrase = input.bip39_passphrase.clone().filter(|p| !p.is_empty() && input.kind == EntryKind::Seed);
    let network = input.network.as_deref().map(str::trim).filter(|n| !n.is_empty()).map(String::from);
    let body = input.body.clone();
    let id = input.id.clone();
    let kind = input.kind;

    let mut inner = state.lock();
    inner.mutate(|payload| {
        let entry = match id {
            Some(id) => {
                let e = payload.find_mut(&id).ok_or(AppError::NotFound)?;
                e.title = title;
                e.words = words;
                e.bip39_passphrase = passphrase;
                e.network = network;
                e.body = body;
                e.updated = now.max(e.updated + 1);
                e.clone()
            }
            None => {
                let e = Entry {
                    id: vault_core::model::new_id()?,
                    kind,
                    title,
                    words,
                    bip39_passphrase: passphrase,
                    network,
                    body,
                    created: now,
                    updated: now,
                    deleted: false,
                };
                payload.entries.push(e.clone());
                e
            }
        };
        Ok(summary(&entry))
    })
}

#[tauri::command]
pub async fn delete_entry(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let mut inner = state.lock();
    inner.mutate(|payload| {
        payload.find_mut(&id).ok_or(AppError::NotFound)?.make_tombstone(now_ms());
        Ok(())
    })
}

#[tauri::command]
pub async fn check_seed(words: Vec<String>) -> AppResult<SeedCheck> {
    Ok(seed::check(&words))
}

#[tauri::command]
pub async fn suggest_words(typed: String) -> AppResult<Vec<&'static str>> {
    Ok(seed::suggest(&typed, 6))
}

#[tauri::command]
pub async fn copy_field(state: State<'_, AppState>, id: String, field: String) -> AppResult<u32> {
    let mut inner = state.lock();
    let seconds = inner.config.settings.clipboard_seconds;
    let e = inner.session()?.payload.find(&id).ok_or(AppError::NotFound)?;
    let text = Zeroizing::new(match field.as_str() {
        "phrase" => e.words.join(" "),
        "passphrase" => e.bip39_passphrase.clone().unwrap_or_default(),
        _ => e.body.clone(),
    });
    #[cfg(windows)]
    let sequence = crate::win::copy_sensitive(&text).ok_or(AppError::Clipboard)?;
    #[cfg(not(windows))]
    let sequence: u32 = { let _ = &text; return Err(AppError::Clipboard) };
    inner.clipboard = Some((sequence, Instant::now() + Duration::from_secs(u64::from(seconds))));
    Ok(seconds)
}

#[tauri::command]
pub async fn export_backup(app: AppHandle, state: State<'_, AppState>) -> AppResult<Option<String>> {
    let bytes = {
        let mut inner = state.lock();
        inner.session()?.current_file()?
    };
    let name = format!("keyplate-backup-{}.keyplate", &storage::timestamp()[..8]);
    let picked = blocking(move || {
        app.dialog()
            .file()
            .set_title("Save an encrypted backup")
            .set_file_name(&name)
            .add_filter("Keyplate vault", &["keyplate"])
            .blocking_save_file()
    })
    .await;
    let Some(path) = picked.and_then(|p| p.into_path().ok()) else { return Ok(None) };
    let storage = state.lock().storage()?;
    storage.export_to(&path, &bytes).map_err(|_| AppError::FolderUnavailable)?;
    Ok(Some(path.display().to_string()))
}

#[tauri::command]
pub async fn change_passwords(
    state: State<'_, AppState>,
    current_first: String,
    current_second: String,
    new_first: String,
    new_second: String,
) -> AppResult<()> {
    let (cf, cs) = (Zeroizing::new(current_first), Zeroizing::new(current_second));
    let (nf, ns) = (Zeroizing::new(new_first), Zeroizing::new(new_second));
    require_strong(&nf, &ns)?;
    let envelope = state.lock().session()?.envelope.clone();
    let (ok, params) = blocking(move || {
        (vault_core::unwrap(envelope.as_bytes(), &cf, &cs).is_ok(), kdf::calibrate(KDF_TARGET))
    })
    .await;
    if !ok {
        return Err(AppError::Vault(vault_core::Error::WrongPasswords));
    }
    let mut inner = state.lock();
    if inner.session()?.sync_from_disk(true)? == SyncOutcome::ForeignVault {
        inner.session = None;
        return Err(AppError::Replaced);
    }
    inner.session()?.change_passwords(&nf, &ns, params)?;
    inner.config.last_practice_ms = Some(now_ms());
    config::save(&inner.config);
    Ok(())
}

#[tauri::command]
pub async fn practice_unlock(state: State<'_, AppState>, first: String, second: String) -> AppResult<bool> {
    let (first, second) = (Zeroizing::new(first), Zeroizing::new(second));
    let envelope = state.lock().session()?.envelope.clone();
    let ok = blocking(move || vault_core::unwrap(envelope.as_bytes(), &first, &second).is_ok()).await;
    if ok {
        let mut inner = state.lock();
        inner.config.last_practice_ms = Some(now_ms());
        config::save(&inner.config);
    }
    Ok(ok)
}

#[tauri::command]
pub async fn save_settings(state: State<'_, AppState>, settings: Settings) -> AppResult<Status> {
    let mut inner = state.lock();
    inner.config.settings = settings.clamped();
    config::save(&inner.config);
    Ok(status_of(&inner))
}

#[tauri::command]
pub async fn recovery_info(state: State<'_, AppState>) -> AppResult<RecoveryInfo> {
    let mut inner = state.lock();
    let storage = inner.storage()?;
    let params = inner.session()?.envelope.header().params;
    let installer_sha256 = std::fs::read_to_string(storage.folder.join("installer").join("SHA256.txt"))
        .ok()
        .and_then(|s| s.split_whitespace().next().map(str::to_lowercase))
        .filter(|h| h.len() == 64 && h.chars().all(|c| c.is_ascii_hexdigit()));
    Ok(RecoveryInfo {
        folder: storage.folder.display().to_string(),
        vault_file: storage.vault_path().display().to_string(),
        installer_sha256,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        kdf: kdf_label(params),
    })
}

/// Every two seconds: clear the clipboard when due, auto-lock when idle, and pick up saves made
/// on other PCs.
pub fn spawn_background(app: AppHandle) {
    std::thread::spawn(move || {
        let mut tick: u64 = 0;
        loop {
            std::thread::sleep(Duration::from_secs(2));
            tick += 1;
            let state = app.state::<AppState>();
            let mut inner = state.lock();

            if let Some((_sequence, due)) = inner.clipboard {
                if Instant::now() >= due {
                    #[cfg(windows)]
                    crate::win::clear_clipboard_if(_sequence);
                    inner.clipboard = None;
                }
            }
            if inner.session.is_none() {
                continue;
            }
            let idle_limit = Duration::from_secs(u64::from(inner.config.settings.auto_lock_minutes) * 60);
            if inner.last_activity.elapsed() >= idle_limit {
                lock_now(&app, &mut inner, "idle");
                continue;
            }
            if tick.is_multiple_of(2) {
                // A full read every ~30 s backs up the cheap size-and-time check.
                let force = tick.is_multiple_of(16);
                let outcome = inner.session.as_mut().map(|s| s.sync_from_disk(force));
                match outcome {
                    Some(Ok(SyncOutcome::Updated)) => {
                        let _ = app.emit("vault-changed", ());
                    }
                    Some(Ok(SyncOutcome::ForeignVault)) => lock_now(&app, &mut inner, "replaced"),
                    _ => {}
                }
            }
        }
    });
}
