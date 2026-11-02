use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("this file is not a Keyplate vault")]
    NotAVault,
    #[error("this vault was made by a newer version of Keyplate (format {0})")]
    UnsupportedVersion(u16),
    #[error("the vault header holds settings Keyplate refuses to use")]
    BadParams,
    /// The key-wrap tag failed. Either password, their order, or the header is wrong.
    #[error("these passwords don't open this vault")]
    WrongPasswords,
    /// The wrap opened but the contents did not authenticate.
    #[error("the vault contents are damaged or were changed")]
    Corrupted,
    #[error("the two passwords must be different")]
    PasswordsMustDiffer,
    #[error("the operating system's random number generator failed")]
    Random,
    #[error("key derivation failed")]
    Kdf,
}

pub type Result<T> = std::result::Result<T, Error>;
