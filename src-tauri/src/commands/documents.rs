use crate::error::{AppError, AppResult};

/// Extracts plain text from a PDF file (used for resume parsing). Runs on a
/// blocking thread since extraction is CPU-bound.
#[tauri::command]
pub async fn extract_pdf_text(path: String) -> AppResult<String> {
    tauri::async_runtime::spawn_blocking(move || {
        pdf_extract::extract_text(&path)
            .map_err(|e| AppError::Other(format!("failed to read PDF: {e}")))
    })
    .await
    .map_err(|e| AppError::Other(e.to_string()))?
    .map(|text| text.trim().to_string())
}
