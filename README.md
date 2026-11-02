# VariScribe

**Every Detail Matters.**

> **IMPORTANT:** Insurance agents are responsible for protecting client information and complying with applicable federal and state laws, regulations, and agency or carrier requirements. VariScribe does not replace those responsibilities.

VariScribe is a Windows desktop intake and documentation tool for insurance professionals. It helps organize client information during calls and export it as a PDF, with optional password protection.

## What It Does

- Collects client contact and application details, including health, beneficiaries, coverage, and payment information.
- Provides an optional call-script qualifier checklist for inbound calls. Its On/Off preference persists locally across app restarts.
- Exports a flattened PDF with editable application data attached as JSON.
- Includes an installer EULA that must be accepted before installation can continue.

## Download and Install

Download the latest Windows installer from [GitHub Releases](https://github.com/lakeryan443-ui/VariScribe/releases/latest). Open the release, download the `VariScribe_<version>_x64-setup.exe` installer from **Assets**, run it, and accept the EULA to continue installation.

## Client Information and PDF Protection

When password protection is selected, the PDF is encrypted with AES-256 and requires that password to reopen. Lost passwords cannot be recovered by the developer. If protection is declined, the PDF and its embedded editable data are not encrypted and can be opened by anyone with access to the file.

VariScribe is intended as an insurance intake tool, **not a document-encryption or file-sharing service**. It does not provide cloud storage or client-record hosting. Client information is saved only when the agent chooses **Save PDF**; the app does not automatically save an unfinished intake. The exported PDF and its embedded JSON contain the same client information and should be protected accordingly.

The agent is responsible for deciding what information may be collected, safeguarding the device, PDF, and password, and following applicable privacy, security, retention, and disclosure requirements.

## Development

Requirements: Node.js and the Rust toolchain. From the repository root:

```powershell
cd desktop
npm install
npm run tauri dev
```

Build the Windows installer with:

```powershell
npm run tauri build
```

The NSIS installer is generated under `src-tauri/target/release/bundle/nsis/`.

Run the PDF encryption round-trip test with:

```powershell
npm run test:pdf-encryption
```

See [`desktop/src-tauri/license.txt`](desktop/src-tauri/license.txt) for the installer EULA. VariScribe is not a legal, medical, underwriting, or regulatory-compliance adviser.