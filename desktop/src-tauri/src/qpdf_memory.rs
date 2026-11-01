use std::{
    ffi::{c_char, c_int, c_void, CStr},
    path::Path,
    ptr, slice,
};

use libloading::os::windows::{
    Library, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
};
use zeroize::Zeroizing;

type QpdfData = *mut c_void;
type QpdfError = *mut c_void;
#[cfg(test)]
type QpdfObject = u32;

type InitFn = unsafe extern "C" fn() -> QpdfData;
type CleanupFn = unsafe extern "C" fn(*mut QpdfData);
type ReadMemoryFn =
    unsafe extern "C" fn(QpdfData, *const c_char, *const c_char, u64, *const c_char) -> c_int;
type SetSuppressWarningsFn = unsafe extern "C" fn(QpdfData, c_int);
type HasErrorFn = unsafe extern "C" fn(QpdfData) -> c_int;
type GetErrorFn = unsafe extern "C" fn(QpdfData) -> QpdfError;
type GetErrorTextFn = unsafe extern "C" fn(QpdfData, QpdfError) -> *const c_char;
type SetR6EncryptionFn = unsafe extern "C" fn(
    QpdfData,
    *const c_char,
    *const c_char,
    c_int,
    c_int,
    c_int,
    c_int,
    c_int,
    c_int,
    c_int,
    c_int,
);
type InitWriteMemoryFn = unsafe extern "C" fn(QpdfData) -> c_int;
type WriteFn = unsafe extern "C" fn(QpdfData) -> c_int;
type GetBufferLengthFn = unsafe extern "C" fn(QpdfData) -> usize;
type GetBufferFn = unsafe extern "C" fn(QpdfData) -> *const u8;

#[cfg(test)]
type GetRootFn = unsafe extern "C" fn(QpdfData) -> QpdfObject;
#[cfg(test)]
type GetKeyFn = unsafe extern "C" fn(QpdfData, QpdfObject, *const c_char) -> QpdfObject;
#[cfg(test)]
type IsInitializedFn = unsafe extern "C" fn(QpdfData, QpdfObject) -> c_int;
#[cfg(test)]
type GetArrayLengthFn = unsafe extern "C" fn(QpdfData, QpdfObject) -> c_int;
#[cfg(test)]
type GetArrayItemFn = unsafe extern "C" fn(QpdfData, QpdfObject, c_int) -> QpdfObject;
#[cfg(test)]
type GetUtf8ValueFn = unsafe extern "C" fn(QpdfData, QpdfObject) -> *const c_char;
#[cfg(test)]
type GetStreamDataFn = unsafe extern "C" fn(
    QpdfData,
    QpdfObject,
    c_int,
    *mut c_int,
    *mut *mut u8,
    *mut usize,
) -> c_int;
#[cfg(test)]
type FreeBufferFn = unsafe extern "C" fn(*mut *mut u8);

struct QpdfApi {
    _library: Library,
    init: InitFn,
    cleanup: CleanupFn,
    read_memory: ReadMemoryFn,
    set_suppress_warnings: SetSuppressWarningsFn,
    has_error: HasErrorFn,
    get_error: GetErrorFn,
    get_error_text: GetErrorTextFn,
    set_r6_encryption: SetR6EncryptionFn,
    init_write_memory: InitWriteMemoryFn,
    write: WriteFn,
    get_buffer_length: GetBufferLengthFn,
    get_buffer: GetBufferFn,
    #[cfg(test)]
    get_root: GetRootFn,
    #[cfg(test)]
    get_key: GetKeyFn,
    #[cfg(test)]
    is_initialized: IsInitializedFn,
    #[cfg(test)]
    get_array_length: GetArrayLengthFn,
    #[cfg(test)]
    get_array_item: GetArrayItemFn,
    #[cfg(test)]
    get_utf8_value: GetUtf8ValueFn,
    #[cfg(test)]
    get_stream_data: GetStreamDataFn,
    #[cfg(test)]
    free_buffer: FreeBufferFn,
}

impl QpdfApi {
    fn load(library_path: &Path) -> Result<Self, String> {
        let flags = LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS;
        let library = unsafe { Library::load_with_flags(library_path, flags) }
            .map_err(|error| format!("Could not load bundled QPDF library: {error}"))?;
        let api = unsafe {
            Self {
                init: load_symbol(&library, b"qpdf_init\0")?,
                cleanup: load_symbol(&library, b"qpdf_cleanup\0")?,
                read_memory: load_symbol(&library, b"qpdf_read_memory\0")?,
                set_suppress_warnings: load_symbol(&library, b"qpdf_set_suppress_warnings\0")?,
                has_error: load_symbol(&library, b"qpdf_has_error\0")?,
                get_error: load_symbol(&library, b"qpdf_get_error\0")?,
                get_error_text: load_symbol(&library, b"qpdf_get_error_full_text\0")?,
                set_r6_encryption: load_symbol(&library, b"qpdf_set_r6_encryption_parameters2\0")?,
                init_write_memory: load_symbol(&library, b"qpdf_init_write_memory\0")?,
                write: load_symbol(&library, b"qpdf_write\0")?,
                get_buffer_length: load_symbol(&library, b"qpdf_get_buffer_length\0")?,
                get_buffer: load_symbol(&library, b"qpdf_get_buffer\0")?,
                #[cfg(test)]
                get_root: load_symbol(&library, b"qpdf_get_root\0")?,
                #[cfg(test)]
                get_key: load_symbol(&library, b"qpdf_oh_get_key\0")?,
                #[cfg(test)]
                is_initialized: load_symbol(&library, b"qpdf_oh_is_initialized\0")?,
                #[cfg(test)]
                get_array_length: load_symbol(&library, b"qpdf_oh_get_array_n_items\0")?,
                #[cfg(test)]
                get_array_item: load_symbol(&library, b"qpdf_oh_get_array_item\0")?,
                #[cfg(test)]
                get_utf8_value: load_symbol(&library, b"qpdf_oh_get_utf8_value\0")?,
                #[cfg(test)]
                get_stream_data: load_symbol(&library, b"qpdf_oh_get_stream_data\0")?,
                #[cfg(test)]
                free_buffer: load_symbol(&library, b"qpdf_oh_free_buffer\0")?,
                _library: library,
            }
        };
        Ok(api)
    }

    fn check(&self, data: QpdfData, status: c_int, operation: &str) -> Result<(), String> {
        let has_error = unsafe { (self.has_error)(data) != 0 };
        if status & 0b10 == 0 && !has_error {
            return Ok(());
        }

        let error = unsafe { (self.get_error)(data) };
        let message = if error.is_null() {
            "QPDF reported an unspecified error.".to_string()
        } else {
            let text = unsafe { (self.get_error_text)(data, error) };
            if text.is_null() {
                "QPDF reported an unspecified error.".to_string()
            } else {
                unsafe { CStr::from_ptr(text) }
                    .to_string_lossy()
                    .into_owned()
            }
        };
        Err(format!("QPDF {operation} failed: {message}"))
    }

    fn open_memory<'a>(
        &'a self,
        bytes: &[u8],
        password: Option<&[u8]>,
        description: &'static [u8],
    ) -> Result<QpdfDocument<'a>, String> {
        let document = QpdfDocument::new(self)?;
        unsafe { (self.set_suppress_warnings)(document.data, 1) };
        let password = password.map_or(ptr::null(), |value| value.as_ptr().cast());
        let status = unsafe {
            (self.read_memory)(
                document.data,
                description.as_ptr().cast(),
                bytes.as_ptr().cast(),
                bytes.len() as u64,
                password,
            )
        };
        self.check(document.data, status, "reading PDF in memory")?;
        Ok(document)
    }

    #[cfg(test)]
    fn attachment_content(
        &self,
        bytes: &[u8],
        password: Option<&[u8]>,
        filename: &str,
    ) -> Result<Vec<u8>, String> {
        let document = self.open_memory(bytes, password, b"QPDF attachment test\0")?;
        let data = document.data;
        let root = unsafe { (self.get_root)(data) };
        let names = unsafe { (self.get_key)(data, root, b"/Names\0".as_ptr().cast()) };
        let embedded = unsafe { (self.get_key)(data, names, b"/EmbeddedFiles\0".as_ptr().cast()) };
        let files = unsafe { (self.get_key)(data, embedded, b"/Names\0".as_ptr().cast()) };
        if unsafe { (self.is_initialized)(data, files) } == 0 {
            return Err("The embedded-file name tree was not found.".into());
        }

        let count = unsafe { (self.get_array_length)(data, files) };
        for index in (0..count).step_by(2) {
            let name = unsafe { (self.get_array_item)(data, files, index) };
            let name = unsafe { (self.get_utf8_value)(data, name) };
            if name.is_null() || unsafe { CStr::from_ptr(name) }.to_bytes() != filename.as_bytes() {
                continue;
            }

            let specification = unsafe { (self.get_array_item)(data, files, index + 1) };
            let embedded_file =
                unsafe { (self.get_key)(data, specification, b"/EF\0".as_ptr().cast()) };
            let stream = unsafe { (self.get_key)(data, embedded_file, b"/F\0".as_ptr().cast()) };
            let mut filtered = 0;
            let mut buffer = ptr::null_mut();
            let mut length = 0;
            let status = unsafe {
                (self.get_stream_data)(data, stream, 1, &mut filtered, &mut buffer, &mut length)
            };
            let status_result = self.check(data, status, "reading attachment in memory");
            if let Err(error) = status_result {
                if !buffer.is_null() {
                    unsafe { (self.free_buffer)(&mut buffer) };
                }
                return Err(error);
            }
            if filtered == 0 || (length > 0 && buffer.is_null()) {
                if !buffer.is_null() {
                    unsafe { (self.free_buffer)(&mut buffer) };
                }
                return Err("QPDF could not decode the embedded JSON attachment.".into());
            }
            let content = unsafe { slice::from_raw_parts(buffer, length) }.to_vec();
            unsafe { (self.free_buffer)(&mut buffer) };
            return Ok(content);
        }
        Err(format!("The {filename} attachment was not found."))
    }
}

struct QpdfDocument<'a> {
    api: &'a QpdfApi,
    data: QpdfData,
}

impl<'a> QpdfDocument<'a> {
    fn new(api: &'a QpdfApi) -> Result<Self, String> {
        let data = unsafe { (api.init)() };
        if data.is_null() {
            return Err("QPDF could not allocate a document context.".into());
        }
        Ok(Self { api, data })
    }
}

impl Drop for QpdfDocument<'_> {
    fn drop(&mut self) {
        unsafe { (self.api.cleanup)(&mut self.data) };
    }
}

unsafe fn load_symbol<T: Copy>(library: &Library, name: &'static [u8]) -> Result<T, String> {
    library
        .get::<T>(name)
        .map(|symbol| *symbol)
        .map_err(|error| {
            format!(
                "QPDF is missing API symbol {}: {error}",
                String::from_utf8_lossy(name)
            )
        })
}

pub fn encrypt_pdf(
    bytes: &[u8],
    user_password: &str,
    owner_password: &str,
    library_path: &Path,
) -> Result<Zeroizing<Vec<u8>>, String> {
    let api = QpdfApi::load(library_path)?;
    let user_password = nul_terminated(user_password.as_bytes());
    let owner_password = nul_terminated(owner_password.as_bytes());
    let document = api.open_memory(bytes, None, b"VariScribe generated PDF\0")?;
    let status = unsafe { (api.init_write_memory)(document.data) };
    api.check(document.data, status, "preparing encrypted output")?;
    unsafe {
        (api.set_r6_encryption)(
            document.data,
            user_password.as_ptr().cast(),
            owner_password.as_ptr().cast(),
            1,
            1,
            1,
            1,
            1,
            1,
            0,
            1,
        );
    }
    api.check(document.data, 0, "configuring AES-256 encryption")?;
    let status = unsafe { (api.write)(document.data) };
    api.check(document.data, status, "encrypting PDF")?;

    let length = unsafe { (api.get_buffer_length)(document.data) };
    let buffer = unsafe { (api.get_buffer)(document.data) };
    if length == 0 || buffer.is_null() {
        return Err("QPDF produced an empty encrypted PDF.".into());
    }
    let encrypted = unsafe { slice::from_raw_parts(buffer, length) }.to_vec();
    Ok(Zeroizing::new(encrypted))
}

fn nul_terminated(value: &[u8]) -> Zeroizing<Vec<u8>> {
    let mut buffer = Zeroizing::new(value.to_vec());
    buffer.push(0);
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    const USER_PASSWORD: &str = "qpdf-memory-test-password";
    const OWNER_PASSWORD: &str = "qpdf-memory-test-owner";
    const JSON: &[u8] = b"{\"schemaVersion\":1,\"marker\":\"variscribe-memory-roundtrip\"}";

    #[test]
    fn memory_encryption_preserves_attachment_and_requires_password() {
        let library_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("qpdf")
            .join("bin")
            .join("qpdf30.dll");
        let plain_pdf = pdf_with_attachment(JSON);
        let encrypted_pdf = encrypt_pdf(&plain_pdf, USER_PASSWORD, OWNER_PASSWORD, &library_path)
            .expect("QPDF should encrypt from memory");
        assert!(!contains(&encrypted_pdf, JSON));

        let api = QpdfApi::load(&library_path).expect("QPDF DLL should load");
        let user_password = nul_terminated(USER_PASSWORD.as_bytes());
        let content = api
            .attachment_content(&encrypted_pdf, Some(&user_password), "formora-data.json")
            .expect("the encrypted attachment should reopen with its password");
        assert_eq!(content, JSON);

        let incorrect_password = nul_terminated(b"incorrect-password");
        assert!(api
            .attachment_content(
                &encrypted_pdf,
                Some(&incorrect_password),
                "formora-data.json"
            )
            .is_err());
    }

    fn contains(bytes: &[u8], needle: &[u8]) -> bool {
        bytes.windows(needle.len()).any(|window| window == needle)
    }

    fn pdf_with_attachment(contents: &[u8]) -> Vec<u8> {
        let mut embedded_stream = format!(
            "<< /Type /EmbeddedFile /Subtype /application#2Fjson /Length {} >>\nstream\n",
            contents.len()
        )
        .into_bytes();
        embedded_stream.extend_from_slice(contents);
        embedded_stream.extend_from_slice(b"\nendstream");

        let objects = [
            b"<< /Type /Catalog /Pages 2 0 R /Names << /EmbeddedFiles << /Names [ (formora-data.json) 4 0 R ] >> >> /AF [4 0 R] >>".to_vec(),
            b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
            b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>".to_vec(),
            b"<< /Type /Filespec /F (formora-data.json) /UF (formora-data.json) /EF << /F 5 0 R >> >>".to_vec(),
            embedded_stream,
        ];
        let mut pdf = b"%PDF-1.7\n".to_vec();
        let mut offsets = vec![0_usize];
        for (index, object) in objects.iter().enumerate() {
            offsets.push(pdf.len());
            pdf.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
            pdf.extend_from_slice(object);
            pdf.extend_from_slice(b"\nendobj\n");
        }
        let xref_offset = pdf.len();
        pdf.extend_from_slice(format!("xref\n0 {}\n", offsets.len()).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for offset in offsets.iter().skip(1) {
            pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
                offsets.len()
            )
            .as_bytes(),
        );
        pdf
    }
}
