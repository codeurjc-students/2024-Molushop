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
    region: String,
}

impl Client {
    /// Construct S3 client wrapper. Bucket y región se leen una vez aquí; las dos
    /// variables ya se comprobaron al arrancar (utils::required_env).
    pub fn new(config: aws_sdk_s3::Config) -> Client {
        Client {
            s3: aws_sdk_s3::Client::from_conf(config),
            bucket_name: std::env::var("AWS_S3_BUCKET_NAME").expect("AWS_S3_BUCKET_NAME is checked at startup"),
            region: std::env::var("AWS_REGION").expect("AWS_REGION is checked at startup"),
        }
    }

    pub fn url(&self, key: &str) -> String {
        let new_key1 = key.replace("+","%2B"); // en la url se cambian los espacios por +
        let new_key = new_key1.replace(" ","+"); // en la url se cambian los espacios por +
        format!("https://{}.s3.{}.amazonaws.com/{new_key}", self.bucket_name, self.region)
    }

    /// La clave del objeto a partir de la URL guardada en la BD (la de `url`).
    /// `None` si la URL no es de este bucket: entonces no se borra nada.
    pub fn key_from_url(&self, url: &str) -> Option<String> {
        let base = format!("https://{}.s3.{}.amazonaws.com/", self.bucket_name, self.region);
        key_from_url(&base, url)
    }

    /// Facilitate the upload of file to s3.
    pub async fn upload(
        &self,
        file: &actix_multipart::form::tempfile::TempFile,
        key_prefix: &str,
    ) -> Result<UploadedFile, UploadError> {
        let filename = file
            .file_name
            .as_deref()
            .filter(|name| !name.trim().is_empty())
            .ok_or(UploadError::MissingFileName)?;

        //obtener la extension del archivo
        let file_type = self.check_extension(filename);

        //si el filename tiene espacios, cambiar los espacios por +

        //hacer un timestamp para el nombre del archivo
        let timestamp = Utc::now().timestamp();

        let key = format!("{key_prefix}{timestamp}{filename}");
        let s3_url = self
            .put_object_from_file(file.file.path(), &key, &file_type)
            .await?;
        Ok(UploadedFile::new(filename, key, s3_url))
    }

    /// Real upload of file to S3
    async fn put_object_from_file(&self, local_path: &Path, key: &str, file_type: &str) -> Result<String, UploadError> {
        let contents = tokio::fs::read(local_path).await.map_err(UploadError::Read)?;

        let mut put_object_request = self
            .s3
            .put_object()
            .bucket(&self.bucket_name)
            .key(key)
            .body(aws_sdk_s3::primitives::ByteStream::from(contents));

        if file_type != "other" {
            put_object_request = put_object_request.content_type(file_type);
        }

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

    fn check_extension(&self, filename: &str) -> String {
        let extension = filename.rsplit('.').next().unwrap_or_default();
        let file_type;
        if extension == "jpg" || extension == "jpeg" {
            file_type = "image/jpeg";
        } else if extension == "png" {
            file_type = "image/png";
        }else{
            file_type = "other";
        }
        file_type.to_string()
    }
}

/// Deshace lo que hace `Client::url` con la clave: los `+` vuelven a ser
/// espacios y los `%2B`, `+` (en ese orden, el inverso).
fn key_from_url(base: &str, url: &str) -> Option<String> {
    let key = url.strip_prefix(base)?;
    if key.is_empty() {
        return None;
    }
    Some(key.replace('+', " ").replace("%2B", "+"))
}

#[cfg(test)]
mod tests {
    use super::key_from_url;

    const BASE: &str = "https://tienda.s3.eu-north-1.amazonaws.com/";

    #[test]
    fn key_from_url_reverses_url_encoding() {
        let url = format!("{BASE}1767296783Mi+foto%2B1.jpg");
        assert_eq!(key_from_url(BASE, &url).as_deref(), Some("1767296783Mi foto+1.jpg"));
    }

    #[test]
    fn key_from_url_rejects_other_hosts() {
        assert_eq!(key_from_url(BASE, "https://otro.s3.eu-north-1.amazonaws.com/a.jpg"), None);
        assert_eq!(key_from_url(BASE, BASE), None);
    }
}
