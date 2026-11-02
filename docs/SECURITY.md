# Keyplate security model

Keyplate keeps seed phrases and private text in one encrypted file, `vault.keyplate`. Normally that file sits in
`My Drive\Keyplate` and Google Drive for desktop syncs it. Keyplate itself never touches the network.

## What protects the vault

| Threat | Protection |
|---|---|
| Someone takes over your Google account and downloads the file | It is sealed on your PC before Drive sees it. Without both passwords, in order, it is random bytes. |
| Offline guessing against a stolen file | Argon2id with 512 MiB of memory and about 1.2 s per guess on a modern PC. The app also requires at least 80 bits of estimated combined password strength. |
| Stolen or lost PC | The only copies on disk are the encrypted vault, its encrypted versions, and an encrypted offline copy. The key exists only in memory while the vault is unlocked. |
| Tampering with the file | Every byte is authenticated (AEAD with the header as associated data). A modified file never opens. |
| Screen capture, Recall, screen sharing | The window is excluded from capture (`WDA_EXCLUDEFROMCAPTURE`). |
| Clipboard snooping | Copies are marked to stay out of Win+V history and the cloud clipboard, and are cleared after 30 s (configurable). |
| Walking away | Auto-lock after 5 idle minutes (configurable), and when Windows locks or sleeps. |
| Two PCs editing at once | Entries merge by id; the newest edit wins and deletes sync as tombstones. The last 30 saves are kept in `versions\`. |

## Cryptography

```
input   = "Keyplate-v1/passwords" ‖ u32be(len P1) ‖ NFC(P1) ‖ u32be(len P2) ‖ NFC(P2)
stretch = Argon2id v1.3 (input, 32-byte random salt, m = 512 MiB, t ≥ 3, p = 4)
KEK     = HKDF-SHA-512(stretch, info = "keyplate/wrap/v1")
wrap    = XChaCha20-Poly1305(KEK, random 24-byte nonce, VaultKey, AAD = header)
payload = XChaCha20-Poly1305(VaultKey, fresh random nonce per save, JSON padded to 4 KiB, AAD = header ‖ wrap)
```

* **Order matters.** The length prefixes make ("ab","c"), ("a","bc") and ("c","ab") different inputs.
* **No hint about which password was wrong.** The app can't tell, because both feed one key.
* **Password changes re-wrap the VaultKey.** The encrypted contents keep the same key. Other PCs adopt the new
  envelope through a `keyEpoch` counter inside the encrypted payload.
* **Hostile headers are rejected before any work.** KDF settings are bounded so a crafted file can't exhaust memory.
* **Primitives come from the RustCrypto crates** (`argon2`, `chacha20poly1305`, `hkdf`, `sha2`). Nothing is
  hand-rolled.

## Quantum resistance

* **Shor's algorithm has nothing to attack.** It breaks public-key cryptography (RSA, elliptic curves), and Keyplate
  uses none to protect your data.
* **Grover's algorithm only halves symmetric key strength.** 256-bit keys keep about 128-bit security against a
  quantum attacker.
* **Your passwords remain the real limit.** Argon2id's memory-hardness makes quantum password search impractical, but
  that only helps if the passwords themselves are hard to guess. That is why the strength floor exists and the
  generator gives 55 bits per passphrase.
* **"Harvest now, decrypt later" doesn't apply.** Data is encrypted before upload, so a recorded network session or a
  copied file holds nothing a future machine can decrypt without the passwords.
* **The format can change algorithms.** The header carries a version and algorithm ids. If public-key features such
  as inheritance are ever added, they should use hybrid X25519 + ML-KEM-1024 (FIPS 203).

## Honest limits

* **A compromised, unlocked PC can read what you reveal.** Malware running as you while the vault is open can read
  revealed words or the WebView's memory. Lock the vault when you're not using it.
* **JavaScript strings can't be wiped.** Revealed words reach the UI only on demand and are dropped on hide, after
  60 s, or when the window is hidden. Until garbage collection runs, copies may linger in WebView memory. Keys and
  passwords on the Rust side are zeroised.
* **Forgotten passwords are final.** There is no recovery path, by design.
* **Installers are unsigned.** Windows SmartScreen will warn when you run one. Check the SHA-256 against the recovery
  sheet before running it.
* **Dependencies.** As of 2 Nov 2026:
  * `cargo audit` finds no vulnerabilities. Its five warnings (`glib`, `zbus_names`, `zvariant`, `zvariant_derive`,
    `proc-macro-error`) are all Linux-only parts of Tauri and are not in the Windows build.
  * `npm audit` reports a low-severity advisory in `deepmerge`, which the Vite plugin uses at build time only. It is
    not shipped in the app.

  Run both audits before each release.
