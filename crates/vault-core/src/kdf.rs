//! Turns the two passwords into a key-encryption key.
//!
//! input = DOMAIN ‖ u32be(len P1) ‖ NFC(P1) ‖ u32be(len P2) ‖ NFC(P2)
//! KEK   = HKDF-SHA-512(Argon2id(input, salt), info = "keyplate/wrap/v1")
//!
//! The length prefixes make the encoding unambiguous and order-sensitive:
//! ("ab", "c"), ("a", "bc") and ("c", "ab") all derive different keys.

use std::time::{Duration, Instant};

use argon2::{Algorithm, Argon2, Params, Version};
use hkdf::Hkdf;
use sha2::Sha512;
use unicode_normalization::UnicodeNormalization;
use zeroize::Zeroizing;

use crate::error::{Error, Result};

const DOMAIN: &[u8] = b"Keyplate-v1/passwords";
const WRAP_INFO: &[u8] = b"keyplate/wrap/v1";

pub const SALT_LEN: usize = 32;
pub const KEY_LEN: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    /// Memory in KiB.
    pub m_kib: u32,
    /// Passes over memory.
    pub t: u32,
    /// Lanes.
    pub p: u32,
}

impl KdfParams {
    /// 512 MiB, 3 passes, 4 lanes. Memory is what makes GPU and ASIC guessing expensive.
    pub const DEFAULT: Self = Self { m_kib: 512 * 1024, t: 3, p: 4 };

    // Bounds checked on every open, so a hostile header can neither exhaust memory nor hang the app.
    const MIN_M_KIB: u32 = 8 * 1024;
    const MAX_M_KIB: u32 = 4 * 1024 * 1024;
    const MAX_T: u32 = 64;
    const MAX_P: u32 = 16;

    pub fn validate(&self) -> Result<()> {
        let ok = (Self::MIN_M_KIB..=Self::MAX_M_KIB).contains(&self.m_kib)
            && (1..=Self::MAX_T).contains(&self.t)
            && (1..=Self::MAX_P).contains(&self.p)
            && self.m_kib >= 8 * self.p;
        if ok { Ok(()) } else { Err(Error::BadParams) }
    }
}

fn password_input(p1: &str, p2: &str) -> Zeroizing<Vec<u8>> {
    let n1: Zeroizing<String> = Zeroizing::new(p1.nfc().collect());
    let n2: Zeroizing<String> = Zeroizing::new(p2.nfc().collect());
    let mut buf = Zeroizing::new(Vec::with_capacity(DOMAIN.len() + 8 + n1.len() + n2.len()));
    buf.extend_from_slice(DOMAIN);
    for part in [n1.as_bytes(), n2.as_bytes()] {
        buf.extend_from_slice(&(part.len() as u32).to_be_bytes());
        buf.extend_from_slice(part);
    }
    buf
}

/// True when both passwords are the same after normalisation.
pub fn passwords_equal(p1: &str, p2: &str) -> bool {
    p1.nfc().eq(p2.nfc())
}

pub fn derive_kek(
    p1: &str,
    p2: &str,
    salt: &[u8; SALT_LEN],
    params: KdfParams,
) -> Result<Zeroizing<[u8; KEY_LEN]>> {
    params.validate()?;
    let input = password_input(p1, p2);
    let argon_params =
        Params::new(params.m_kib, params.t, params.p, Some(KEY_LEN)).map_err(|_| Error::BadParams)?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);

    let mut stretched = Zeroizing::new([0u8; KEY_LEN]);
    argon
        .hash_password_into(&input, salt, stretched.as_mut())
        .map_err(|_| Error::Kdf)?;

    let mut kek = Zeroizing::new([0u8; KEY_LEN]);
    Hkdf::<Sha512>::new(None, stretched.as_ref())
        .expand(WRAP_INFO, kek.as_mut())
        .map_err(|_| Error::Kdf)?;
    Ok(kek)
}

/// Picks parameters that take about `target` on this machine, never weaker than DEFAULT.
pub fn calibrate(target: Duration) -> KdfParams {
    let probe = KdfParams { t: 1, ..KdfParams::DEFAULT };
    let salt = [0u8; SALT_LEN];
    let start = Instant::now();
    if derive_kek("calibrate", "calibrate", &salt, probe).is_err() {
        return KdfParams::DEFAULT;
    }
    let per_pass = start.elapsed().as_secs_f64().max(0.001);
    let t = (target.as_secs_f64() / per_pass).ceil() as u32;
    KdfParams { t: t.clamp(KdfParams::DEFAULT.t, 10), ..KdfParams::DEFAULT }
}
