# Keyplate

An encrypted vault for crypto seed phrases and any private text. It runs on Windows, opens with two passwords in
order, and syncs a sealed file through Google Drive for desktop. The encryption is quantum-resistant by design.

* [docs/SECURITY.md](docs/SECURITY.md): threat model, cryptography, quantum resistance, honest limits
* [docs/RECOVERY.md](docs/RECOVERY.md): restoring on a new PC

## Layout

| Path | What |
|---|---|
| `crates/vault-core` | Pure Rust: file format, Argon2id + HKDF key derivation, XChaCha20-Poly1305, merge, BIP39 checks, password rules. No I/O. |
| `src-tauri` | Tauri 2 shell: commands, unlocked session, auto-lock, Drive-folder storage, Windows hardening. |
| `src` | Svelte 5 UI in the Steel Plate design. `src/lib/tokens.css` holds every color, font and space. |

## Develop

```bash
npm install
cargo test --workspace          # crypto, format, merge, storage and two-PC sync tests
npm run check                   # Svelte + TypeScript
npm run dev                     # UI only, in a browser, with mock data (?stage=welcome|locked|unlocked)
npm run tauri dev               # the real app
```

## Build the installer

```bash
npm run tauri build
```

The per-user NSIS installer lands in `target/release/bundle/nsis/`. To make it available on new PCs, copy it and its
hash into the Drive folder:

```powershell
$exe = Get-ChildItem target\release\bundle\nsis\*.exe | Select-Object -First 1
New-Item -ItemType Directory -Force "G:\My Drive\Keyplate\installer" | Out-Null
Copy-Item $exe.FullName "G:\My Drive\Keyplate\installer\"
(Get-FileHash $exe.FullName).Hash.ToLower() + "  " + $exe.Name | Out-File -Encoding ascii "G:\My Drive\Keyplate\installer\SHA256.txt"
```

Once `SHA256.txt` exists, the in-app recovery sheet prints the hash.

## License

[MIT](LICENSE)

## Contact

For business inquiries: jayknocks12@gmail.com
