//! Límites de la subida de imágenes de producto (Editar producto → Imágenes).
//!
//! - Tamaño por imagen: `UPLOAD_MAX_IMAGE_MB` (opcional, 5 por defecto, de 1 a
//!   20). Es el que depende de la máquina: cada imagen se lee entera en memoria
//!   para mandarla a S3, y el plan gratis de Render tiene 512 MB.
//! - Imágenes por subida (10) y formatos (JPG, PNG, WebP): fijos, son reglas del
//!   producto. El formato se decide por el contenido del fichero (sus primeros
//!   bytes), no por el nombre: es el `Content-Type` con el que se guarda en S3.
//!
//! El límite de la petición entera (`total_limit_bytes`) se calcula a partir de
//! los dos primeros, para que no haya dos números que puedan contradecirse.

use std::env;
use std::io::Read;
use std::path::Path;

use lazy_static::lazy_static;

pub const MAX_IMAGES_PER_UPLOAD: usize = 10;

const DEFAULT_MAX_IMAGE_MB: usize = 5;
/// Tope de `UPLOAD_MAX_IMAGE_MB`: un `500` puesto por error volvería a dejar
/// que unas pocas subidas llenaran la memoria.
const MAX_ALLOWED_MB: usize = 20;
const MB: usize = 1024 * 1024;
/// Lo que ocupa el multipart además de los ficheros (separadores, cabeceras).
const MULTIPART_OVERHEAD: usize = MB;

lazy_static! {
    static ref MAX_IMAGE_MB: usize = parse_max_image_mb(env::var("UPLOAD_MAX_IMAGE_MB").ok().as_deref())
        .unwrap_or(DEFAULT_MAX_IMAGE_MB);
}

/// Para la comprobación de `main`: sin la variable vale el defecto; con un
/// valor que no es un número de 1 a 20, la app no arranca.
pub fn check_upload_config() -> Result<(), String> {
    parse_max_image_mb(env::var("UPLOAD_MAX_IMAGE_MB").ok().as_deref()).map(|_| ())
}

fn parse_max_image_mb(value: Option<&str>) -> Result<usize, String> {
    let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(DEFAULT_MAX_IMAGE_MB);
    };
    match value.parse::<usize>() {
        Ok(mb) if (1..=MAX_ALLOWED_MB).contains(&mb) => Ok(mb),
        _ => Err(format!("UPLOAD_MAX_IMAGE_MB \"{}\" is not valid (a whole number of MB from 1 to {})", value, MAX_ALLOWED_MB)),
    }
}

/// Megas por imagen (para los textos y el JS del formulario).
pub fn max_image_mb() -> usize {
    *MAX_IMAGE_MB
}

pub fn max_image_bytes() -> usize {
    max_image_mb() * MB
}

/// Para `MultipartFormConfig::total_limit`: lo que más puede ocupar una subida
/// válida. Pasado esto, actix deja de leer la petición.
pub fn total_limit_bytes() -> usize {
    max_image_bytes() * MAX_IMAGES_PER_UPLOAD + MULTIPART_OVERHEAD
}

/// Configuración de los extractores `MultipartForm` (en `main.rs`, `.app_data`).
/// Si la petición pasa del límite o viene mal formada, el aviso normal del panel
/// en vez de la respuesta en texto plano de actix, que htmx no pintaría.
pub fn multipart_config() -> actix_multipart::form::MultipartFormConfig {
    actix_multipart::form::MultipartFormConfig::default()
        .total_limit(total_limit_bytes())
        .error_handler(|err, _req| {
            // Pasarse del límite es `PayloadError::Overflow` ("...size limit"), suelto o
            // envuelto en el error del campo donde se cortó: se mira el texto entero.
            let too_big = format!("{err:?} {err}").contains("Overflow") || err.to_string().contains("size limit");
            let (status, text) = if too_big {
                (
                    actix_web::http::StatusCode::PAYLOAD_TOO_LARGE,
                    format!("La subida es demasiado grande: como mucho {} imágenes de {} MB cada una", MAX_IMAGES_PER_UPLOAD, max_image_mb()),
                )
            } else {
                (actix_web::http::StatusCode::BAD_REQUEST, "No se ha podido leer la subida: inténtalo de nuevo".to_string())
            };
            tracing::warn!(error = %err, too_big, "multipart upload refused");
            actix_web::error::InternalError::from_response(err, crate::utils::hx::error_modal(status, &text)).into()
        })
}

/// El `Content-Type` de una imagen admitida según sus primeros bytes, o `None`
/// si no es JPG, PNG ni WebP (aunque el nombre diga `.jpg`).
pub fn sniff_image_type(header: &[u8]) -> Option<&'static str> {
    if header.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if header.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some("image/png")
    } else if header.len() >= 12 && &header[0..4] == b"RIFF" && &header[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

/// Lee los primeros bytes del fichero subido (en disco, en el temporal de
/// actix) y dice su tipo. Un error de lectura cuenta como "no es una imagen".
pub fn image_type_of_file(path: &Path) -> Option<&'static str> {
    let mut header = [0u8; 12];
    let mut file = std::fs::File::open(path).ok()?;
    let read = file.read(&mut header).ok()?;
    sniff_image_type(&header[..read])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_or_blank_uses_the_default() {
        assert_eq!(parse_max_image_mb(None), Ok(DEFAULT_MAX_IMAGE_MB));
        assert_eq!(parse_max_image_mb(Some("  ")), Ok(DEFAULT_MAX_IMAGE_MB));
    }

    #[test]
    fn accepts_whole_numbers_from_1_to_20() {
        assert_eq!(parse_max_image_mb(Some("1")), Ok(1));
        assert_eq!(parse_max_image_mb(Some(" 8 ")), Ok(8));
        assert_eq!(parse_max_image_mb(Some("20")), Ok(20));
    }

    #[test]
    fn refuses_out_of_range_or_not_a_number() {
        for bad in ["0", "21", "500", "abc", "2.5", "-3"] {
            assert!(parse_max_image_mb(Some(bad)).is_err(), "{bad} should be refused");
        }
    }

    #[test]
    fn sniffs_jpeg_png_and_webp() {
        assert_eq!(sniff_image_type(&[0xFF, 0xD8, 0xFF, 0xE0]), Some("image/jpeg"));
        assert_eq!(sniff_image_type(b"\x89PNG\r\n\x1a\n...."), Some("image/png"));
        assert_eq!(sniff_image_type(b"RIFF\x00\x00\x00\x00WEBPVP8 "), Some("image/webp"));
    }

    /// Lo que no es una de las tres se rechaza aunque se llame .jpg (GIF, PDF, texto).
    #[test]
    fn refuses_other_content() {
        assert_eq!(sniff_image_type(b"GIF89a......"), None);
        assert_eq!(sniff_image_type(b"%PDF-1.7...."), None);
        assert_eq!(sniff_image_type(b"hello"), None);
        assert_eq!(sniff_image_type(b""), None);
        assert_eq!(sniff_image_type(b"RIFF\x00\x00\x00\x00WAVE"), None);
    }

    #[test]
    fn total_limit_fits_the_largest_valid_upload() {
        assert!(total_limit_bytes() >= max_image_bytes() * MAX_IMAGES_PER_UPLOAD);
    }
}
