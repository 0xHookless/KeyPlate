# Restoring Keyplate on a new PC

You need three things:

1. **Your Google account**, the one that ran Google Drive for desktop on the old PC.
2. **Both passwords, in order.**
3. **The Keyplate installer.** It is in `My Drive\Keyplate\installer\` (or in the GitHub releases, if you set that up).

## Steps

1. **Install Google Drive for desktop** from Google and sign in.
2. **Wait for `My Drive\Keyplate` to appear.** Then right-click the folder and choose **Available offline**.
3. **Check the installer before running it.** In PowerShell:
   ```powershell
   Get-FileHash "G:\My Drive\Keyplate\installer\Keyplate_0.1.0_x64-setup.exe"
   ```
   The hash must match `SHA256.txt` in the same folder **and** the one printed on your recovery sheet. If the two
   files agree with each other but not with the sheet, don't run it.
4. **Run the installer.** It installs for your user only and doesn't need admin rights. SmartScreen may say "Windows
   protected your PC". Choose *More info*, then *Run anyway*, but only after the hash matched.
5. **Open the vault.** Open Keyplate, choose **Open an existing vault**, and pick `My Drive\Keyplate`.
6. **Unlock.** Type the first password, then the second.

## If it doesn't open

* **"These passwords don't open this vault."** Check Caps Lock, the keyboard layout, and the order. If you changed
  the passwords recently, try the new ones. Older files in `versions\` still open with the old ones.
* **"Keyplate can't reach the vault folder."** Google Drive isn't running, or the folder hasn't synced yet.
* **"This vault file is damaged."** Copy an earlier file from `versions\` over `vault.keyplate`, or use Google
  Drive's version history (right-click the file, then *File information*, then *Manage versions*).

## Where things live on each PC

| What | Where |
|---|---|
| The vault | `My Drive\Keyplate\vault.keyplate` |
| Earlier saves (last 30) | `My Drive\Keyplate\versions\` |
| Encrypted offline copy | `%APPDATA%\Keyplate\cache.keyplate` |
| Settings (no secrets) | `%APPDATA%\Keyplate\config.json` |
