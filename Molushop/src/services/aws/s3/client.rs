//use serde::de::value::Error;
// backend/src/uploads/client.rs
use tokio::io::AsyncReadExt as _;
//use crate::upload::UploadedFile;
//use crate::services::aws::s3::upload::UploadedFile;
use super::upload::UploadedFile;
use chrono::prelude::*;

use aws_sdk_s3::types::{Delete, ObjectIdentifier};
//use aws_sdk_s3::output::DeleteObjectsOutput;
use std::error::Error;


/// S3 client wrapper to expose semantic upload operations.
#[derive(Debug, Clone)]
pub struct Client {
    s3: aws_sdk_s3::Client,
    bucket_name: String,
}

impl Client {
    /// Construct S3 client wrapper.
    pub fn new(config: aws_sdk_s3::Config) -> Client {
        Client {
            s3: aws_sdk_s3::Client::from_conf(config),
            bucket_name: std::env::var("AWS_S3_BUCKET_NAME").unwrap(),
        }
    }

    pub fn url(&self, key: &str) -> String {
        let new_key1 = key.replace("+","%2B"); // en la url se cambian los espacios por +
        let new_key = new_key1.replace(" ","+"); // en la url se cambian los espacios por +
        format!(
            "https://{}.s3.{}.amazonaws.com/{new_key}",
            std::env::var("AWS_S3_BUCKET_NAME").unwrap(),
            std::env::var("AWS_REGION").unwrap(),
        )
    }

    /// Facilitate the upload of file to s3.
    pub async fn upload(
        &self,
        file: &actix_multipart::form::tempfile::TempFile,
        key_prefix: &str,
    ) -> UploadedFile {
        let filename = file.file_name.as_deref().expect("TODO");

        //obtener la extension del archivo
        let file_type = self.check_extension(filename);

        //si el filename tiene espacios, cambiar los espacios por +
       // let filename_new = filename.replace(" ", "+");
        
        //hacer un timestamp para el nombre del archivo
        let timestamp = Utc::now().timestamp();

        let key = format!("{key_prefix}{timestamp}{filename}");
        let s3_url = self
            .put_object_from_file(file.file.path().to_str().unwrap(), &key, &file_type)
            .await;
        UploadedFile::new(filename, key, s3_url)
    }

    /// Real upload of file to S3
    async fn put_object_from_file(&self, local_path: &str, key: &str, file_type:&str) -> String {
        let mut file = tokio::fs::File::open(local_path).await.unwrap();

        let size_estimate = file
            .metadata()
            .await
            .map(|md| md.len())
            .unwrap_or(1024)
            .try_into()
            .expect("file too big");

        let mut contents = Vec::with_capacity(size_estimate);
        file.read_to_end(&mut contents).await.unwrap();

        let mut put_object_request = self
            .s3
            .put_object()
            .bucket(&self.bucket_name)
            .key(key)
            .body(aws_sdk_s3::primitives::ByteStream::from(contents));

        if file_type != "other" {
            put_object_request = put_object_request.content_type(file_type);
        }

        let _res = put_object_request
            .send()
            .await
            .expect("Failed to put object");

        self.url(key)
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

        let delete = Delete::builder().set_objects(Some(objects)).build().map_err(|err|{println!("Error: {:?}",err); err})?;

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
                println!("Error deleting objects: {:?}",e);
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
                            .map(|object| object.key.expect("No key"))
                            .collect();
                        Ok(keys)
                    },
                    None => {
                        println!("No contents");
                        //Ok(Vec::new())
                        //Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No contents")))
                        Err("No content")?
                    }
                }
            },
            Err(e) => {
                println!("Error listing objects: {:?}",e);
                Err(Box::new(e))
            }
        }
        
        //Ok(keys)
    }

    fn check_extension(&self, filename: &str) -> String {
        let extension = filename.split('.').last().unwrap();
        let file_type;
        if extension == "jpg" || extension == "jpeg" {
            file_type = "image/jpeg";
        } else if(extension == "png") {
            file_type = "image/png";
        }else{
            file_type = "other";
        }
        file_type.to_string()
    }
}