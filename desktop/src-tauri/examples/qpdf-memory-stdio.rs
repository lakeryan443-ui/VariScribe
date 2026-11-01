#[path = "../src/qpdf_memory.rs"]
mod qpdf_memory;

use std::{
    io::{self, Read, Write},
    path::PathBuf,
};

use zeroize::Zeroizing;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut plain_pdf = Zeroizing::new(Vec::new());
    io::stdin().read_to_end(&mut plain_pdf)?;
    let qpdf_dll = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("qpdf")
        .join("bin")
        .join("qpdf30.dll");
    let encrypted_pdf = qpdf_memory::encrypt_pdf(
        &plain_pdf,
        "variscribe-memory-test-password",
        "variscribe-memory-test-owner",
        &qpdf_dll,
    )
    .map_err(io::Error::other)?;
    io::stdout().write_all(&encrypted_pdf)?;
    Ok(())
}
