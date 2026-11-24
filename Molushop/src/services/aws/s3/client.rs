// backend/src/uploads/client.rs
use std::path::Path;
use super::upload::{UploadError, UploadedFile};
use chrono::prelude::*;

use aws_sdk_s3::types::{Delete, ObjectIdentifier};
use std::error::Error;

/// S3 client wrapper to expose semantic upload operations.
#[derive(Debug, Clone)]
pub struct Client {
    s3: aws_sdk_s3::Client,
    bucket_name: String,
    /// Dirección pública de los objetos, con `/` al final: la de AWS o, con un S3
    /// simulado en local, `S3_PUBLIC_BASE_URL` (ver `public_base_url`).
    public_base: String,
}

impl Client {
    /// Construct S3 client wrapper. Bucket y región se leen una vez aquí; las dos
    /// variables ya se comprobaron al arrancar (utils::required_env).
    pub fn new(config: aws_sdk_s3::Config) -> Client {
        let bucket_name = std::env::var("AWS_S3_BUCKET_NAME").expect("AWS_S3_BUCKET_NAME is checked at startup");
        let region = std::env::var("AWS_REGION").expect("AWS_REGION is checked at startup");
        let public_base = public_base_url(
            std::env::var("S3_PUBLIC_BASE_URL").ok().as_deref(),
            &bucket_name,
            &region,
        );
        Client {
            s3: aws_sdk_s3::Client::from_conf(config),
            bucket_name,
            public_base,
        }
    }

    pub fn url(&self, key: &str) -> String {
        format!("{}{}", self.public_base, encode_key(key))
    }

    /// La clave del objeto a partir de la URL guardada en la BD (la de `url`).
    /// `None` si la URL no es de este bucket: entonces no se borra nada.
    pub fn key_from_url(&self, url: &str) -> Option<String> {
        key_from_url(&self.public_base, url)
    }

    /// Facilitate the upload of file to s3.
    pub async fn upload(
        &self,
        file: &actix_multipart::form::tempfile::TempFile,
        key_prefix: &str,
        content_type: &str,
    ) -> Result<UploadedFile, UploadError> {
        let filename = file
            .file_name
            .as_deref()
            .filter(|name| !name.trim().is_empty())
            .ok_or(UploadError::MissingFileName)?;

        //si el filename tiene espacios, cambiar los espacios por +

        //hacer un timestamp para el nombre del archivo
        let timestamp = Utc::now().timestamp();

        let key = format!("{key_prefix}{timestamp}{filename}");
        let s3_url = self
            .put_object_from_file(file.file.path(), &key, content_type)
            .await?;
        Ok(UploadedFile::new(filename, key, s3_url))
    }

    /// Real upload of file to S3. `content_type` lo decide quien llama a partir
    /// del contenido del fichero (utils::upload_limits::image_type_of_file).
    async fn put_object_from_file(&self, local_path: &Path, key: &str, content_type: &str) -> Result<String, UploadError> {
        let contents = tokio::fs::read(local_path).await.map_err(UploadError::Read)?;

        let put_object_request = self
            .s3
            .put_object()
            .bucket(&self.bucket_name)
            .key(key)
            .content_type(content_type)
            .body(aws_sdk_s3::primitives::ByteStream::from(contents));

        // Con DisplayErrorContext: el Display del error a secas solo diría
        // "service error", sin el motivo (credenciales, permisos, bucket...).
        put_object_request
            .send()
            .await
            .map_err(|e| UploadError::S3(format!("{}", aws_sdk_s3::error::DisplayErrorContext(e))))?;

        Ok(self.url(key))
    }

    /// Attempts to delete object from S3. Returns true if successful.
    pub async fn delete_file(&self, key: &str) -> bool {
        self.s3
            .delete_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await
            .is_ok()
    }

    pub async fn delete_all_files(&self) -> Result<(), Box<dyn Error>> {
        let keys = self.list_objects().await?;
        self.delete_files_from_vec(keys).await
    }

    /// Attempts to delete all objects from S3. Returns true if successful.
    pub async fn delete_files_from_vec(&self, elements: Vec<String>) -> Result<(), Box<dyn Error>> {
        let objects: Vec<ObjectIdentifier> = elements
            .into_iter()
            .filter_map(|key| ObjectIdentifier::builder().key(key).build().ok())
            .collect();

        let delete = Delete::builder().set_objects(Some(objects)).build().map_err(|err| { tracing::error!(error = ?err, "could not build the S3 delete request"); err })?;

        let fin= self
            .s3
            .delete_objects()
            .bucket(&self.bucket_name)
            .delete(delete)
            .send()
            .await;
        
        match fin {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!(error = ?e, "could not delete the objects from S3");
                Err(Box::new(e))
            }
        }
        
    }

    pub async fn list_objects(&self) -> Result<Vec<String>,Box<dyn Error>> {
        let list_objects = self
            .s3
            .list_objects_v2()
            .bucket(&self.bucket_name)
            .send()
            .await;
           
        match list_objects {
            Ok(list_objects_in) => {
                let keys_pre =list_objects_in
                    .contents;
                
                match keys_pre {
                    Some(keys) => {
                        let keys = keys
                            .into_iter()
                            .filter_map(|object| object.key)
                            .collect();
                        Ok(keys)
                    },
                    None => {
                        tracing::debug!("the S3 bucket is empty");
                        Err("No content")?
                    }
                }
            },
            Err(e) => {
                tracing::error!(error = ?e, "could not list the S3 objects");
                Err(Box::new(e))
            }
        }

    }
}

/// Dirección pública de los objetos del bucket, siempre con `/` al final.
/// Sin `S3_PUBLIC_BASE_URL` (o vacía), la de AWS (Render y el S3 real); con ella,
/// la que se dé (p. ej. la web de Garage en local).
fn public_base_url(configured: Option<&str>, bucket: &str, region: &str) -> String {
    match configured.map(str::trim).filter(|base| !base.is_empty()) {
        Some(base) => format!("{}/", base.trim_end_matches('/')),
        None => format!("https://{bucket}.s3.{region}.amazonaws.com/"),
    }
}

/// La clave para la URL: todo lo que no sea letra o cifra ASCII, `-._~` o `/`
/// va como `%XX` (UTF-8), así que el espacio es `%20`. Antes el espacio iba
/// como `+`, que solo entiende AWS (Garage, como cualquier servidor web, busca
/// un `+` de verdad); esas URL antiguas siguen en la BD, ver `key_from_url`.
fn encode_key(key: &str) -> String {
    let mut out = String::with_capacity(key.len());
    for byte in key.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~/".contains(&byte) {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// La clave del objeto a partir de su URL: deshace `encode_key` y también la
/// forma antigua (espacio = `+`, `+` = `%2B`). Ninguna de las dos deja un `+`
/// literal en la URL, así que todo `+` es un espacio.
fn key_from_url(base: &str, url: &str) -> Option<String> {
    let key = url.strip_prefix(base)?;
    if key.is_empty() {
        return None;
    }
    Some(percent_decode(&key.replace('+', " ")))
}

/// `%XX` -> el byte. Un `%` que no va seguido de dos cifras hex se deja tal cual
/// (podía venir en el nombre de un fichero antiguo).
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes.get(i + 1..i + 3).and_then(|h| std::str::from_utf8(h).ok());
        match (bytes[i], hex.and_then(|h| u8::from_str_radix(h, 16).ok())) {
            (b'%', Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::{encode_key, key_from_url, public_base_url};

    const BASE: &str = "https://tienda.s3.eu-north-1.amazonaws.com/";

    /// Las URL guardadas antes del cambio (espacio = `+`) se siguen reconociendo.
    #[test]
    fn key_from_url_reads_the_old_plus_encoding() {
        let url = format!("{BASE}1767296783Mi+foto%2B1.jpg");
        assert_eq!(key_from_url(BASE, &url).as_deref(), Some("1767296783Mi foto+1.jpg"));
    }

    #[test]
    fn encode_key_uses_percent_encoding() {
        assert_eq!(encode_key("1767296783Mi foto+1 ñ#?.jpg"), "1767296783Mi%20foto%2B1%20%C3%B1%23%3F.jpg");
        assert_eq!(encode_key("carpeta/a-b_c.~1.png"), "carpeta/a-b_c.~1.png");
    }

    #[test]
    fn key_from_url_reverses_encode_key() {
        for key in ["1767296783Mi foto+1 ñ#?.jpg", "simple.png", "50%.jpg", "carpeta/x y.webp"] {
            let url = format!("{BASE}{}", encode_key(key));
            assert_eq!(key_from_url(BASE, &url).as_deref(), Some(key));
        }
    }

    #[test]
    fn key_from_url_rejects_other_hosts() {
        assert_eq!(key_from_url(BASE, "https://otro.s3.eu-north-1.amazonaws.com/a.jpg"), None);
        assert_eq!(key_from_url(BASE, BASE), None);
    }

    #[test]
    fn public_base_defaults_to_aws() {
        assert_eq!(public_base_url(None, "tienda", "eu-north-1"), BASE);
        assert_eq!(public_base_url(Some("  "), "tienda", "eu-north-1"), BASE);
    }

    #[test]
    fn public_base_uses_the_configured_one_with_a_single_slash() {
        let local = "http://molushop-dev.web.localhost:3902/";
        assert_eq!(public_base_url(Some("http://molushop-dev.web.localhost:3902"), "x", "y"), local);
        assert_eq!(public_base_url(Some("http://molushop-dev.web.localhost:3902//"), "x", "y"), local);
    }
}
