use std::fmt;

#[derive(Debug, serde::Serialize, Clone)]
pub struct UploadedFile {
    pub filename: String,
    /// Clave del objeto en el bucket: hace falta para borrarlo si la subida
    /// de otro fichero de la misma petición falla.
    pub s3_key: String,
    pub s3_url: String,
}

impl UploadedFile {
    /// Construct new uploaded file info container.
    pub fn new(
        filename: impl Into<String>,
        s3_key: impl Into<String>,
        s3_url: impl Into<String>,
    ) -> Self {
        Self {
            filename: filename.into(),
            s3_key: s3_key.into(),
            s3_url: s3_url.into(),
        }
    }
}

/// Por qué no se ha podido subir un fichero a S3.
#[derive(Debug)]
pub enum UploadError {
    /// El navegador siempre manda el nombre; sin él, la petición está hecha a
    /// mano. Es culpa de quien la manda (400), no del servidor.
    MissingFileName,
    /// No se ha podido leer el fichero temporal donde actix dejó la subida.
    Read(std::io::Error),
    /// S3 ha rechazado la subida o no ha respondido (red, permisos, caído...).
    S3(String),
}

impl fmt::Display for UploadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UploadError::MissingFileName => f.write_str("the uploaded file has no name"),
            UploadError::Read(e) => write!(f, "could not read the temporary file: {}", e),
            UploadError::S3(e) => write!(f, "S3 rejected the upload: {}", e),
        }
    }
}

impl std::error::Error for UploadError {}
