// task-backend/src/utils/image_optimizer.rs

use crate::error::{AppError, AppResult};
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::Cursor;

/// 画像最適化の設定
#[derive(Debug, Clone)]
pub struct ImageOptimizationConfig {
    /// WebP変換を有効にするか
    pub enable_webp_conversion: bool,
    /// 最大幅（ピクセル）
    pub max_width: u32,
    /// 最大高さ（ピクセル）
    pub max_height: u32,
    /// WebP品質（0-100）
    pub webp_quality: f32,
    /// JPEG品質（0-100）
    pub jpeg_quality: u8,
    /// EXIFデータを除去するか（将来の実装で使用予定）
    #[allow(dead_code)]
    pub strip_metadata: bool,
    /// 元のファイルを保持するか
    pub keep_original: bool,
}

impl Default for ImageOptimizationConfig {
    fn default() -> Self {
        Self {
            enable_webp_conversion: true,
            max_width: 2048,
            max_height: 2048,
            webp_quality: 85.0,
            jpeg_quality: 85,
            strip_metadata: true,
            keep_original: false,
        }
    }
}

/// 画像最適化の結果
pub struct OptimizationResult {
    /// 最適化後のデータ
    pub data: Vec<u8>,
    /// 最適化後のMIMEタイプ
    pub mime_type: String,
    /// 最適化後のファイル名（拡張子が変更される場合）
    pub file_name: String,
    /// 元のサイズ（バイト）
    pub original_size: usize,
    /// 最適化後のサイズ（バイト）
    pub optimized_size: usize,
    /// 圧縮率（％）
    pub compression_ratio: f32,
}

/// 画像を最適化
pub fn optimize_image(
    input_data: &[u8],
    original_mime_type: &str,
    original_file_name: &str,
    config: &ImageOptimizationConfig,
) -> AppResult<OptimizationResult> {
    let original_size = input_data.len();

    // 画像を読み込み
    let img = image::load_from_memory(input_data)
        .map_err(|e| AppError::BadRequest(format!("Invalid image format: {}", e)))?;

    // リサイズが必要かチェック
    let img = resize_if_needed(img, config.max_width, config.max_height);

    // WebP変換が有効で、元がWebPでない場合は変換
    let (output_data, mime_type, file_name) = if config.enable_webp_conversion
        && !original_mime_type.contains("webp")
        && is_webp_suitable(original_mime_type)
    {
        // WebPに変換
        let webp_data = convert_to_webp(&img, config.webp_quality)?;
        let webp_file_name = change_extension(original_file_name, "webp");

        (webp_data, "image/webp".to_string(), webp_file_name)
    } else {
        // 元の形式のまま最適化
        optimize_in_original_format(&img, original_mime_type, original_file_name, config)?
    };

    let optimized_size = output_data.len();
    let compression_ratio =
        ((original_size - optimized_size) as f32 / original_size as f32) * 100.0;

    Ok(OptimizationResult {
        data: output_data,
        mime_type,
        file_name,
        original_size,
        optimized_size,
        compression_ratio,
    })
}

/// 必要に応じて画像をリサイズ
fn resize_if_needed(img: DynamicImage, max_width: u32, max_height: u32) -> DynamicImage {
    let (width, height) = img.dimensions();

    if width <= max_width && height <= max_height {
        return img;
    }

    // アスペクト比を保持してリサイズ
    let ratio_w = max_width as f32 / width as f32;
    let ratio_h = max_height as f32 / height as f32;
    let ratio = ratio_w.min(ratio_h);

    let new_width = (width as f32 * ratio) as u32;
    let new_height = (height as f32 * ratio) as u32;

    img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
}

/// WebPに変換
fn convert_to_webp(img: &DynamicImage, quality: f32) -> AppResult<Vec<u8>> {
    let rgba_image = img.to_rgba8();
    let (width, height) = rgba_image.dimensions();

    // WebPエンコーダーを使用
    let encoder = webp::Encoder::from_rgba(&rgba_image, width, height);
    let webp_data = encoder.encode(quality);

    Ok(webp_data.to_vec())
}

/// 元の形式で最適化
fn optimize_in_original_format(
    img: &DynamicImage,
    mime_type: &str,
    file_name: &str,
    config: &ImageOptimizationConfig,
) -> AppResult<(Vec<u8>, String, String)> {
    let mut buffer = Cursor::new(Vec::new());

    match mime_type {
        "image/jpeg" => {
            // JPEG品質設定を適用
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                &mut buffer,
                config.jpeg_quality,
            );
            img.write_with_encoder(encoder).map_err(|e| {
                AppError::InternalServerError(format!("Failed to encode JPEG: {}", e))
            })?;
        }
        "image/png" => {
            img.write_to(&mut buffer, ImageFormat::Png).map_err(|e| {
                AppError::InternalServerError(format!("Failed to encode PNG: {}", e))
            })?;
        }
        "image/gif" => {
            img.write_to(&mut buffer, ImageFormat::Gif).map_err(|e| {
                AppError::InternalServerError(format!("Failed to encode GIF: {}", e))
            })?;
        }
        _ => {
            // その他の形式はそのまま返す
            return Err(AppError::BadRequest(format!(
                "Unsupported image format: {}",
                mime_type
            )));
        }
    }

    Ok((
        buffer.into_inner(),
        mime_type.to_string(),
        file_name.to_string(),
    ))
}

/// WebP変換に適した画像形式かチェック
fn is_webp_suitable(mime_type: &str) -> bool {
    matches!(mime_type, "image/jpeg" | "image/png" | "image/gif")
}

/// ファイル名の拡張子を変更
fn change_extension(file_name: &str, new_ext: &str) -> String {
    let base_name = file_name
        .rfind('.')
        .map_or(file_name, |pos| &file_name[..pos]);

    format!("{}.{}", base_name, new_ext)
}

/// サポートされている画像形式かチェック
pub fn is_image_mime_type(mime_type: &str) -> bool {
    matches!(
        mime_type,
        "image/jpeg" | "image/png" | "image/gif" | "image/webp"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_extension() {
        assert_eq!(change_extension("photo.jpg", "webp"), "photo.webp");
        assert_eq!(change_extension("image.png", "webp"), "image.webp");
        assert_eq!(
            change_extension("no_extension", "webp"),
            "no_extension.webp"
        );
        assert_eq!(change_extension("file.name.jpg", "webp"), "file.name.webp");
    }

    #[test]
    fn test_is_webp_suitable() {
        assert!(is_webp_suitable("image/jpeg"));
        assert!(is_webp_suitable("image/png"));
        assert!(is_webp_suitable("image/gif"));
        assert!(!is_webp_suitable("image/webp"));
        assert!(!is_webp_suitable("application/pdf"));
    }

    #[test]
    fn test_is_image_mime_type() {
        assert!(is_image_mime_type("image/jpeg"));
        assert!(is_image_mime_type("image/png"));
        assert!(is_image_mime_type("image/gif"));
        assert!(is_image_mime_type("image/webp"));
        assert!(!is_image_mime_type("application/pdf"));
        assert!(!is_image_mime_type("text/plain"));
    }
}
