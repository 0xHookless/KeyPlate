//! Errors as the UI shows them: a stable code for the screen to react to, and a plain message
//! that says what happened and what to do next.

use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    Vault(vault_core::Error),
    Locked,
    NoVaultFolder,
    NoVaultHere,
    VaultAlreadyHere,
    FolderUnavailable,
    WeakPasswords,
    NotFound,
    Clipboard,
    Replaced,
}

impl From<vault_core::Error> for AppError {
    fn from(e: vault_core::Error) -> Self {
        AppError::Vault(e)
    }
}

#[derive(Serialize)]
pub struct UiError {
    code: &'static str,
    message: String,
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use vault_core::Error as V;
        let (code, message): (&str, String) = match self {
            AppError::Vault(V::WrongPasswords) => (
                "wrongPasswords",
                "These passwords don't open this vault. Check each one, and that the first password is in the first box.".into(),
            ),
            AppError::Vault(V::PasswordsMustDiffer) => ("passwordsMustDiffer", "Use two different passwords.".into()),
            AppError::Vault(V::NotAVault) => (
                "notAVault",
                "This file isn't a Keyplate vault. Choose the folder that holds vault.keyplate.".into(),
            ),
            AppError::Vault(V::UnsupportedVersion(_)) => (
                "unsupportedVersion",
                "This vault was saved by a newer Keyplate. Install the latest version, then open it again.".into(),
            ),
            AppError::Vault(V::Corrupted | V::BadParams) => (
                "corrupted",
                "This vault file is damaged. Open an earlier copy from the versions folder, or Google Drive's version history.".into(),
            ),
            AppError::Vault(V::Random | V::Kdf) => (
                "internal",
                "Windows couldn't provide what Keyplate needs to encrypt safely. Restart Keyplate and try again.".into(),
            ),
            AppError::Locked => ("locked", "The vault locked itself. Unlock it to continue.".into()),
            AppError::NoVaultFolder => ("noVaultFolder", "Choose where your vault lives first.".into()),
            AppError::NoVaultHere => (
                "noVaultHere",
                "There's no vault.keyplate in this folder. If Google Drive is still syncing, wait a moment and try again.".into(),
            ),
            AppError::VaultAlreadyHere => (
                "vaultAlreadyHere",
                "This folder already holds a vault. Open it instead, or choose an empty folder.".into(),
            ),
            AppError::FolderUnavailable => (
                "folderUnavailable",
                "Keyplate can't reach the vault folder. Check that Google Drive is running, then try again.".into(),
            ),
            AppError::WeakPasswords => (
                "weakPasswords",
                "Together these passwords are too easy to guess. Lengthen one, or use Generate.".into(),
            ),
            AppError::NotFound => ("notFound", "That item no longer exists. It may have been deleted on another PC.".into()),
            AppError::Clipboard => ("clipboard", "Windows didn't allow copying just now. Try again.".into()),
            AppError::Replaced => (
                "replaced",
                "The vault file was replaced by a different vault, so Keyplate locked. Unlock to open the new file.".into(),
            ),
        };
        UiError { code, message }.serialize(s)
    }
}

pub type AppResult<T> = Result<T, AppError>;
