//! The `.keyplate` file format, version 1.
//!
//! ```text
//! offset len  field
//! 0      8    magic "KEYPLATE"
//! 8      2    format version, u16 BE (1)
//! 10     1    KDF id (1 = Argon2id v1.3)
//! 11     4    memory KiB, u32 BE
//! 15     4    passes, u32 BE
//! 19     4    lanes, u32 BE
//! 23     32   salt
//! 55     1    AEAD id (1 = XChaCha20-Poly1305)
//! ----------- header ends at 56; it is the AAD of the key wrap
//! 56     24   wrap nonce
//! 80     48   VaultKey sealed under the KEK (32 + 16 tag)
//! ----------- envelope ends at 128; it is the AAD of the payload
//! 128    24   payload nonce
//! 152    ..   payload sealed under the VaultKey:
//!             u32be(json len) ‖ json ‖ zero padding to a multiple of 4 KiB, then a 16-byte tag
//! ```
//!
//! Every byte is authenticated: changing any of them makes `unlock` or `open_with_key` fail.

use chacha20poly1305::aead::{Aead, KeyInit, Payload as AeadMsg};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use zeroize::Zeroizing;

use crate::error::{Error, Result};
use crate::kdf::{self, KdfParams, KEY_LEN, SALT_LEN};
use crate::model::{Payload, SCHEMA};

pub const MAGIC: &[u8; 8] = b"KEYPLATE";
pub const VERSION: u16 = 1;
const KDF_ARGON2ID: u8 = 1;
const AEAD_XCHACHA20POLY1305: u8 = 1;

pub const HEADER_LEN: usize = 56;
pub const ENVELOPE_LEN: usize = 128;
const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 16;
const PAYLOAD_START: usize = ENVELOPE_LEN + NONCE_LEN;
const PAD_BLOCK: usize = 4096;

/// The random key that encrypts the vault contents. Never leaves memory unencrypted.
pub struct VaultKey(Zeroizing<[u8; KEY_LEN]>);

impl VaultKey {
    pub fn generate() -> Result<Self> {
        Ok(Self(Zeroizing::new(random()?)))
    }
}

impl std::fmt::Debug for VaultKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("VaultKey(..)")
    }
}

/// Header plus wrapped VaultKey: everything needed to unlock with the passwords.
#[derive(Clone, PartialEq, Eq)]
pub struct Envelope([u8; ENVELOPE_LEN]);

impl Envelope {
    pub fn header(&self) -> Header {
        // An Envelope is only ever built from a header that already parsed.
        parse_header(&self.0).expect("envelope holds a valid header")
    }

    pub fn as_bytes(&self) -> &[u8; ENVELOPE_LEN] {
        &self.0
    }
}

impl std::fmt::Debug for Envelope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Envelope").field(&self.header()).finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub version: u16,
    pub params: KdfParams,
    pub salt: [u8; SALT_LEN],
}

/// Everything `unlock` recovers from a file.
#[derive(Debug)]
pub struct Unlocked {
    pub envelope: Envelope,
    pub key: VaultKey,
    pub payload: Payload,
}

fn random<const N: usize>() -> Result<[u8; N]> {
    let mut buf = [0u8; N];
    getrandom::getrandom(&mut buf).map_err(|_| Error::Random)?;
    Ok(buf)
}

fn cipher(key: &[u8; KEY_LEN]) -> XChaCha20Poly1305 {
    XChaCha20Poly1305::new(Key::from_slice(key))
}

fn encode_header(params: KdfParams, salt: &[u8; SALT_LEN]) -> [u8; HEADER_LEN] {
    let mut h = [0u8; HEADER_LEN];
    h[0..8].copy_from_slice(MAGIC);
    h[8..10].copy_from_slice(&VERSION.to_be_bytes());
    h[10] = KDF_ARGON2ID;
    h[11..15].copy_from_slice(&params.m_kib.to_be_bytes());
    h[15..19].copy_from_slice(&params.t.to_be_bytes());
    h[19..23].copy_from_slice(&params.p.to_be_bytes());
    h[23..55].copy_from_slice(salt);
    h[55] = AEAD_XCHACHA20POLY1305;
    h
}

fn be_u32(b: &[u8]) -> u32 {
    u32::from_be_bytes(b.try_into().expect("4 bytes"))
}

fn parse_header(file: &[u8]) -> Result<Header> {
    if file.len() < HEADER_LEN || &file[0..8] != MAGIC {
        return Err(Error::NotAVault);
    }
    let version = u16::from_be_bytes([file[8], file[9]]);
    if version != VERSION {
        return Err(Error::UnsupportedVersion(version));
    }
    if file[10] != KDF_ARGON2ID || file[55] != AEAD_XCHACHA20POLY1305 {
        return Err(Error::BadParams);
    }
    let params = KdfParams { m_kib: be_u32(&file[11..15]), t: be_u32(&file[15..19]), p: be_u32(&file[19..23]) };
    params.validate()?;
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&file[23..55]);
    Ok(Header { version, params, salt })
}

/// Reads the header without any secrets, e.g. to show the KDF settings.
pub fn read_header(file: &[u8]) -> Result<Header> {
    parse_header(file)
}

/// Seals `key` under a fresh salt and the two passwords, in order.
pub fn wrap(key: &VaultKey, p1: &str, p2: &str, params: KdfParams) -> Result<Envelope> {
    if kdf::passwords_equal(p1, p2) {
        return Err(Error::PasswordsMustDiffer);
    }
    let salt: [u8; SALT_LEN] = random()?;
    let header = encode_header(params, &salt);
    let kek = kdf::derive_kek(p1, p2, &salt, params)?;
    let nonce: [u8; NONCE_LEN] = random()?;
    let sealed = cipher(&kek)
        .encrypt(XNonce::from_slice(&nonce), AeadMsg { msg: key.0.as_ref(), aad: &header })
        .map_err(|_| Error::Kdf)?;

    let mut env = [0u8; ENVELOPE_LEN];
    env[..HEADER_LEN].copy_from_slice(&header);
    env[HEADER_LEN..HEADER_LEN + NONCE_LEN].copy_from_slice(&nonce);
    env[HEADER_LEN + NONCE_LEN..].copy_from_slice(&sealed);
    Ok(Envelope(env))
}

/// Recovers the VaultKey from the start of a file. Fails with `WrongPasswords` on any mismatch.
pub fn unwrap(file: &[u8], p1: &str, p2: &str) -> Result<(Envelope, VaultKey)> {
    let header = parse_header(file)?;
    if file.len() < ENVELOPE_LEN {
        return Err(Error::NotAVault);
    }
    let kek = kdf::derive_kek(p1, p2, &header.salt, header.params)?;
    let nonce = &file[HEADER_LEN..HEADER_LEN + NONCE_LEN];
    let sealed = &file[HEADER_LEN + NONCE_LEN..ENVELOPE_LEN];
    let opened = Zeroizing::new(
        cipher(&kek)
            .decrypt(XNonce::from_slice(nonce), AeadMsg { msg: sealed, aad: &file[..HEADER_LEN] })
            .map_err(|_| Error::WrongPasswords)?,
    );
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    key.copy_from_slice(&opened);
    let mut env = [0u8; ENVELOPE_LEN];
    env.copy_from_slice(&file[..ENVELOPE_LEN]);
    Ok((Envelope(env), VaultKey(key)))
}

/// Encrypts `payload` into a complete file, with a fresh nonce.
pub fn seal(envelope: &Envelope, key: &VaultKey, payload: &Payload) -> Result<Vec<u8>> {
    let json = Zeroizing::new(serde_json::to_vec(payload).map_err(|_| Error::Corrupted)?);
    let used = 4 + json.len();
    let padded_len = used.div_ceil(PAD_BLOCK).max(1) * PAD_BLOCK;
    let mut plain = Zeroizing::new(Vec::with_capacity(padded_len));
    plain.extend_from_slice(&(json.len() as u32).to_be_bytes());
    plain.extend_from_slice(&json);
    plain.resize(padded_len, 0);

    let nonce: [u8; NONCE_LEN] = random()?;
    let sealed = cipher(&key.0)
        .encrypt(XNonce::from_slice(&nonce), AeadMsg { msg: &plain, aad: envelope.as_bytes() })
        .map_err(|_| Error::Corrupted)?;

    let mut file = Vec::with_capacity(PAYLOAD_START + sealed.len());
    file.extend_from_slice(envelope.as_bytes());
    file.extend_from_slice(&nonce);
    file.extend_from_slice(&sealed);
    Ok(file)
}

/// Decrypts a file with a key already in memory (used to merge changes from another device).
/// A file from a different vault fails with `Corrupted`.
pub fn open_with_key(file: &[u8], key: &VaultKey) -> Result<(Envelope, Payload)> {
    parse_header(file)?;
    if file.len() < PAYLOAD_START + TAG_LEN {
        return Err(Error::Corrupted);
    }
    let mut env = [0u8; ENVELOPE_LEN];
    env.copy_from_slice(&file[..ENVELOPE_LEN]);
    let nonce = &file[ENVELOPE_LEN..PAYLOAD_START];
    let plain = Zeroizing::new(
        cipher(&key.0)
            .decrypt(XNonce::from_slice(nonce), AeadMsg { msg: &file[PAYLOAD_START..], aad: &env })
            .map_err(|_| Error::Corrupted)?,
    );
    if plain.len() < 4 {
        return Err(Error::Corrupted);
    }
    let len = be_u32(&plain[..4]) as usize;
    let json = plain.get(4..4 + len).ok_or(Error::Corrupted)?;
    let payload: Payload = serde_json::from_slice(json).map_err(|_| Error::Corrupted)?;
    if payload.schema > SCHEMA {
        return Err(Error::UnsupportedVersion(payload.schema as u16));
    }
    Ok((Envelope(env), payload))
}

/// Opens a file with the two passwords, in order.
pub fn unlock(file: &[u8], p1: &str, p2: &str) -> Result<Unlocked> {
    let (envelope, key) = unwrap(file, p1, p2)?;
    let (_, payload) = open_with_key(file, &key)?;
    Ok(Unlocked { envelope, key, payload })
}

/// Creates a new vault: a fresh VaultKey wrapped under the two passwords.
pub fn create(p1: &str, p2: &str, params: KdfParams) -> Result<(Envelope, VaultKey)> {
    let key = VaultKey::generate()?;
    let envelope = wrap(&key, p1, p2, params)?;
    Ok((envelope, key))
}
