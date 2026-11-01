VARISCRIBE DESKTOP APP
======================

DEVELOPMENT
Open a terminal in the desktop folder and run:
  npm install
  npm run tauri dev

WINDOWS INSTALLER
Build the installer with:
  npm run tauri build

The NSIS installer is written to:
  src-tauri/target/release/bundle/nsis/

PDF EXPORT
Exported PDFs are password-protected with AES-256 encryption and include an
editable JSON attachment. Use synthetic information for demonstrations.