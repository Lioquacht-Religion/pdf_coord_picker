// pdf_load.rs

use std::path::Path;

use pdfium_render::prelude::{PdfDocument, Pdfium, PdfiumError};

#[cfg(not(target_arch = "wasm32"))]
pub fn load_pdf_native<'a>(
    pdfium: &'a Pdfium,
    path: &Path,
) -> Result<PdfDocument<'a>, PdfiumError> {
    pdfium.load_pdf_from_file(path, None)
}

#[cfg(target_arch = "wasm32")]
use eframe::web_sys::Blob;

#[cfg(target_arch = "wasm32")]
pub async fn load_pdf_web<'a>(
    pdfium: &'a Pdfium,
    blob: Blob,
) -> Result<PdfDocument<'a>, PdfiumError> {
    pdfium.load_pdf_from_blob(blob, None).await
}
