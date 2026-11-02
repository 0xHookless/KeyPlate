//! Keyplate's vault core: file format, key derivation, encryption, data model, merge and seed
//! checks. Pure logic with no file or network I/O, so all of it is unit-testable.
//!
//! Only symmetric primitives with 256-bit keys are used (Argon2id, HKDF-SHA-512,
//! XChaCha20-Poly1305), which keeps the design resistant to quantum attacks: Shor's algorithm has
//! no public key to break, and Grover's at best halves the 256-bit key strength.

pub mod error;
pub mod format;
pub mod kdf;
pub mod merge;
pub mod model;
pub mod passwords;
pub mod seed;

pub use error::{Error, Result};
pub use format::{create, open_with_key, read_header, seal, unlock, unwrap, wrap, Envelope, Header, Unlocked, VaultKey};
pub use kdf::KdfParams;
pub use merge::merge;
pub use model::{Entry, EntryKind, Payload};
