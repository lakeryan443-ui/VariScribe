# VariScribe Session Recovery - 2026-11-02

## Current Project State

- Repository: https://github.com/lakeryan443-ui/VariScribe
- Default branch: `main`
- Current main commit: `d7f60bd` (`Prepare VariScribe v1.0.3 release`)
- Latest public release: `v1.0.3`
- Windows installer: https://github.com/lakeryan443-ui/VariScribe/releases/download/v1.0.3/VariScribe_1.0.3_x64-setup.exe
- The v1.0.3 installer includes the installer EULA, optional PDF password protection, plain-PDF export when protection is declined, and the release-build GUI subsystem setting.
- PDF encryption round-trip tests passed for protected and unprotected PDFs, including editable JSON attachment recovery and wrong-password rejection.

## App Behavior

- VariScribe is a local insurance intake and documentation tool, not a document-encryption or file-sharing service.
- Save PDF asks whether to protect the export. Yes opens the password flow and encrypts the PDF with AES-256; No saves it without encryption. An unprotected PDF and its embedded JSON can be opened by anyone with the file.
- Edit Existing PDF should only ask for a password when the selected PDF is protected.
- The call-script qualifier preference is stored locally and persists across app restarts.
- Windows installation requires accepting the EULA.
- The v1.0.3 release is public. A future release should use a new version/tag rather than replacing an existing release asset.

## Work Between Laptop and Desktop

Use GitHub as the source-of-truth for the project folder. Do not live-sync the repository with OneDrive, Dropbox, or another file-sync service while editing on both computers; this can conflict with `.git`, builds, or active edits.

On the Acer, check changes before switching machines:

```powershell
git status --short
git add <specific source files>
git commit -m "Describe the change"
git push origin main
```

On the desktop, clone once:

```powershell
git clone https://github.com/lakeryan443-ui/VariScribe.git
cd VariScribe
git switch main
```

Before starting work on either computer, sync the latest commits:

```powershell
git switch main
git pull --ff-only origin main
```

Commit and push before moving to the other machine. For larger changes, use a feature branch and merge it into `main` after testing.

## VS Code Settings Sync

Sign in to VS Code with the same account on both computers, open the gear menu, and choose **Turn on Settings Sync**. Select settings, extensions, keyboard shortcuts, snippets, and UI state as desired. Settings Sync does not sync the project folder or replace Git; the repository must still be cloned and pulled separately.

## Desktop Development Setup

Install Git for Windows, Node.js LTS, Rust via rustup, and the Visual Studio C++ Build Tools needed by Tauri. Then, from the cloned repository:

```powershell
cd desktop
npm ci
npm run tauri dev
```

Build and test with:

```powershell
npm run build
npm run test:pdf-encryption
npm run tauri build
```

The Windows installer is generated under `desktop/src-tauri/target/release/bundle/nsis/`.

## Local Files to Review Before Syncing

The laptop currently has untracked client/business PDFs, a root `package-lock.json`, and an older demo PDF under `desktop/examples/`. They are not part of the current GitHub release. Do not stage the entire repository blindly; inspect `git status --short` and only add intended source files. Keep real client information out of GitHub unless it has been securely de-identified and explicitly approved for publication.
