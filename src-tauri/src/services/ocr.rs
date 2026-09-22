use rusty_tesseract::{Args, Image};
use std::collections::HashMap;

use crate::error::{AppError, AppResult};
use crate::models::OcrResult;

/// Runs Tesseract OCR against a saved screenshot and returns the extracted
/// text together with a mean confidence score.
pub fn extract_text(screenshot_path: &str, language: &str) -> AppResult<OcrResult> {
    let img = Image::from_path(screenshot_path)
        .map_err(|e| AppError::Ocr(format!("failed to load image: {e}")))?;

    let mut config = HashMap::new();
    config.insert("preserve_interword_spaces".to_string(), "1".to_string());

    let args = Args {
        lang: if language.is_empty() {
            "eng".to_string()
        } else {
            language.to_string()
        },
        config_variables: config,
        dpi: Some(150),
        psm: Some(3),
        oem: Some(3),
    };

    let text = rusty_tesseract::image_to_string(&img, &args)
        .map_err(|e| AppError::Ocr(format!("tesseract failed: {e}. Is tesseract installed and on PATH?")))?;

    let confidence = mean_confidence(&img, &args).unwrap_or(0.0);

    Ok(OcrResult {
        text: text.trim().to_string(),
        confidence,
        screenshot_path: screenshot_path.to_string(),
    })
}

fn mean_confidence(img: &Image, args: &Args) -> Option<f32> {
    let data = rusty_tesseract::image_to_data(img, args).ok()?;
    let confidences: Vec<f32> = data
        .data
        .iter()
        .map(|d| d.conf)
        .filter(|c| *c >= 0.0)
        .collect();
    if confidences.is_empty() {
        return None;
    }
    Some(confidences.iter().sum::<f32>() / confidences.len() as f32)
}
