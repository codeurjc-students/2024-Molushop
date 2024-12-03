use actix_web::{get, post, web,delete, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use aws_config::imds::client;
use bigdecimal::BigDecimal;
use lazy_static::lazy_static;
use tera::Tera;
use actix_files as fs;
use serde::Deserialize;
use serde::Serialize;
use std::{str::FromStr, sync::Mutex};
use std::sync::Arc;

use actix_web::Scope;

use crate::services::aws::s3::client::Client;

use actix_multipart::{
    form::{
        tempfile::{TempFile, TempFileConfig},
        MultipartForm,
        json::Json as MpJson
    },
    Multipart,
};

pub fn scope_s3() -> Scope {
    web::scope("/s3")
        .configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(save_files);
    cfg.service(delete_files);
    cfg.service(delete_all_files_2);
    cfg.service(delete_fail);
    cfg.service(listar_archivos_s3);
}

#[derive(Deserialize)]
struct Delete {
    key: String,
}

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(limit = "100MB")]
    file: TempFile,
}

#[post("/upload-file")]
async fn save_files(
    client_data: web::Data<Arc<Client>>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> impl Responder {
    println!("Vamos a ver si se sube el archivo!");
    let client_data_x = client_data.get_ref().as_ref();
    let uploaded_file=  client_data_x.upload(&form.file, "").await;
    println!("Archivo subido!");

    HttpResponse::Ok().body(format!("File uploaded: {:?}", uploaded_file))
}

#[get("/delete-all")]
async fn delete_all_files_2(
    client_data: web::Data<Arc<Client>>
) -> impl Responder {
    println!("Vamos a borrar un archivo!");
    let client_data_x = client_data.get_ref().as_ref();
    //let uploaded_file=  client_data_x.upload(&form.file, "").await;
    let end =  client_data_x.delete_all_files().await;
    match end {
        Ok(_) => {
            println!("Archivos borrados!");
            return HttpResponse::Ok().body("Archivos borrados");
        },
        Err(e) => {
            println!("Error al borrar archivos: {}", e);
            return HttpResponse::InternalServerError().body("Error al borrar archivos");
        }
    }
}


#[post("/delete-file")]
async fn delete_files(
    client_data: web::Data<Arc<Client>>,
    web::Form(form): web::Form<Delete>
) -> impl Responder {
    println!("Vamos a borrar un archivo!");
    let client_data_x = client_data.get_ref().as_ref();
    //let uploaded_file=  client_data_x.upload(&form.file, "").await;
    let end =  client_data_x.delete_file(&form.key).await;
    let result = if end {"Archivo borrado!"} else {"Error al borrar el archivo"};

    HttpResponse::Ok().body(result)
}


#[get("/delete-fail")]
async fn delete_fail(
    client_data: web::Data<Arc<Client>>
) -> impl Responder{
    let client_data_x = client_data.get_ref().as_ref();
    //let lista = vec!["hola".to_string(),"adios".to_string()];
    let lista = vec!["1733000369bone.jpg".to_string(),"xd".to_string()];

    let end =  client_data_x.delete_files_from_vec(lista).await;
    match end {
        Ok(_) => {
            println!("Archivos borrados!");
            return HttpResponse::Ok().body("Archivos borrados");
        },
        Err(e) => {
            println!("Error al borrar archivos: {}", e);
            return HttpResponse::InternalServerError().body("Error al borrar archivos");
        }
    }
}

#[get("/listar-archivos-s3")]
async fn listar_archivos_s3(
    client_data: web::Data<Arc<Client>>
) -> impl Responder {
    println!("Vamos a listar los archivos!");
    let client_data_x = client_data.get_ref().as_ref();
    //let uploaded_file=  client_data_x.upload(&form.file, "").await;
    let end =  client_data_x.list_objects().await;
    //let result = if end {"Archivos listados!"} else {"Error al listar los archivos"};
    println!("Archivos: {:?}",end);

    HttpResponse::Ok().body("Archivos listados")
}