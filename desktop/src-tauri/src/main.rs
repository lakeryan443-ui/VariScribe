#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;
use zeroize::Zeroizing;

mod qpdf_memory;

const MAX_PDF_BYTES: u64 = 100 * 1024 * 1024;

#[tauri::command]
async fn pick_pdf_bytes(app: AppHandle) -> Result<Option<Vec<u8>>, String> {
    let selected = app
        .dialog()
        .file()
        .add_filter("PDF", &["pdf"])
        .blocking_pick_file();
    let Some(selected) = selected else {
        return Ok(None);
    };
    let path = selected
        .into_path()
        .map_err(|error| format!("Could not access selected PDF: {error}"))?;
    let metadata = fs::metadata(&path).map_err(|error| format!("Could not read PDF: {error}"))?;
    if metadata.len() > MAX_PDF_BYTES {
        return Err("The selected PDF is larger than 100 MB.".into());
    }
    let bytes = fs::read(path).map_err(|error| format!("Could not read PDF: {error}"))?;
    Ok(Some(bytes))
}

#[tauri::command]
async fn save_pdf(
    app: AppHandle,
    pdf_bytes: Vec<u8>,
    password: Option<String>,
) -> Result<Option<String>, String> {
    let pdf_bytes = Zeroizing::new(pdf_bytes);
    let password = password.map(Zeroizing::new);
    if pdf_bytes.is_empty() || pdf_bytes.len() as u64 > MAX_PDF_BYTES {
        return Err("The generated PDF is empty or larger than 100 MB.".into());
    }
    if let Some(password) = &password {
        validate_password(password.as_str())?;
    }

    let selected = app
        .dialog()
        .file()
        .add_filter("PDF", &["pdf"])
        .set_file_name("VariScribe.pdf")
        .blocking_save_file();
    let Some(selected) = selected else {
        return Ok(None);
    };
    let mut destination = selected
        .into_path()
        .map_err(|error| format!("Could not access save location: {error}"))?;
    if destination.extension().is_none() {
        destination.set_extension("pdf");
    }

    if let Some(password) = password {
        let mut owner_bytes = Zeroizing::new([0_u8; 32]);
        getrandom::fill(&mut owner_bytes[..])
            .map_err(|error| format!("Could not generate encryption credentials: {error}"))?;
        let owner_password = Zeroizing::new(
            owner_bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
        );
        let dll_path = qpdf_dll_path(&app)?;
        encrypt_and_save_pdf(
            &pdf_bytes,
            &password,
            &owner_password,
            &dll_path,
            &destination,
        )?;
    } else {
        write_pdf_atomic(&destination, &pdf_bytes)?;
    }
    Ok(Some(destination.display().to_string()))
}

fn validate_password(password: &str) -> Result<(), String> {
    if !(12..=127).contains(&password.len())
        || !password.bytes().all(|byte| (0x20..=0x7e).contains(&byte))
    {
        return Err("Use a password of 12 to 127 printable ASCII characters.".into());
    }
    Ok(())
}

fn qpdf_dll_path(app: &AppHandle) -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("VARISCRIBE_QPDF_DLL_PATH") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
    }

    let bundled_path = app
        .path()
        .resource_dir()
        .map_err(|error| format!("Could not locate application resources: {error}"))?
        .join("qpdf")
        .join("bin")
        .join("qpdf30.dll");
    if bundled_path.is_file() {
        return Ok(bundled_path);
    }

    let development_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("qpdf")
        .join("bin")
        .join("qpdf30.dll");
    if development_path.is_file() {
        return Ok(development_path);
    }
    Err("The bundled PDF encryption component is missing.".into())
}

fn write_pdf_atomic(destination: &Path, pdf_bytes: &[u8]) -> Result<(), String> {
    let directory = destination
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut staged = tempfile::Builder::new()
        .prefix(".variscribe-")
        .suffix(".pdf.tmp")
        .tempfile_in(directory)
        .map_err(|error| format!("Could not stage PDF: {error}"))?;
    staged
        .write_all(pdf_bytes)
        .map_err(|error| format!("Could not write PDF: {error}"))?;
    staged
        .as_file()
        .sync_all()
        .map_err(|error| format!("Could not flush PDF: {error}"))?;
    staged
        .persist(destination)
        .map_err(|error| format!("Could not save PDF: {}", error.error))?;
    Ok(())
}

fn encrypt_and_save_pdf(
    pdf_bytes: &[u8],
    password: &str,
    owner_password: &str,
    qpdf_dll: &Path,
    destination: &Path,
) -> Result<(), String> {
    let encrypted_pdf = qpdf_memory::encrypt_pdf(pdf_bytes, password, owner_password, qpdf_dll)?;
    write_pdf_atomic(destination, &encrypted_pdf)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![pick_pdf_bytes, save_pdf])
        .run(tauri::generate_context!())
        .expect("error while running VariScribe");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn interrupted_save_leaves_only_staged_pdf_bytes() {
        let directory = tempfile::tempdir().expect("temporary test directory");
        let destination = directory.path().join("client.pdf");
        let pdf_bytes = b"PDF-bytes-marker";

        let mut staged = tempfile::Builder::new()
            .prefix(".variscribe-")
            .suffix(".pdf.tmp")
            .tempfile_in(directory.path())
            .expect("stage output");
        staged.write_all(pdf_bytes).expect("write PDF bytes");
        staged.as_file().sync_all().expect("flush PDF bytes");
        let staged_path = staged.path().to_path_buf();
        std::mem::forget(staged);

        assert!(!destination.exists());
        assert_eq!(
            fs::read(&staged_path).expect("read interrupted stage"),
            pdf_bytes
        );
        assert_eq!(
            fs::read_dir(directory.path())
                .expect("list temp directory")
                .count(),
            1
        );
    }

    #[test]
    fn atomic_save_replaces_only_the_final_pdf_file() {
        let directory = tempfile::tempdir().expect("temporary test directory");
        let destination = directory.path().join("client.pdf");
        fs::write(&destination, b"previous PDF bytes").expect("create existing destination");

        write_pdf_atomic(&destination, b"new PDF bytes").expect("atomic PDF save");

        assert_eq!(
            fs::read(&destination).expect("read saved PDF"),
            b"new PDF bytes"
        );
        assert_eq!(
            fs::read_dir(directory.path())
                .expect("list temp directory")
                .count(),
            1
        );
    }

    #[test]
    fn failed_final_save_removes_the_staged_pdf() {
        let directory = tempfile::tempdir().expect("temporary test directory");
        let destination = directory.path().join("existing-directory");
        fs::create_dir(&destination).expect("create conflicting destination");

        assert!(write_pdf_atomic(&destination, b"PDF-marker").is_err());
        assert!(destination.is_dir());
        assert_eq!(
            fs::read_dir(directory.path())
                .expect("list temp directory")
                .count(),
            1
        );
    }

    #[test]
    fn atomic_save_preserves_unprotected_pdf_bytes() {
        let directory = tempfile::tempdir().expect("temporary test directory");
        let destination = directory.path().join("client.pdf");
        let pdf_bytes = b"%PDF-1.7\nunprotected test PDF";

        write_pdf_atomic(&destination, pdf_bytes).expect("save unprotected PDF");

        assert_eq!(fs::read(&destination).expect("read saved PDF"), pdf_bytes);
    }

    #[test]
    fn qpdf_failure_creates_no_files() {
        let directory = tempfile::tempdir().expect("temporary test directory");
        let destination = directory.path().join("client.pdf");
        let qpdf_dll = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("qpdf")
            .join("bin")
            .join("qpdf30.dll");

        let result = encrypt_and_save_pdf(
            b"not a PDF",
            "qpdf-memory-test-password",
            "qpdf-memory-test-owner",
            &qpdf_dll,
            &destination,
        );

        assert!(result.is_err());
        assert_eq!(
            fs::read_dir(directory.path())
                .expect("list temp directory")
                .count(),
            0
        );
    }
}
