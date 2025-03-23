use actix_web::{get, post, web,delete,Scope, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use diesel::result;
use lazy_static::lazy_static;
use rinja::Template;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::json;

//use crate::{models::data_transfer_objects::product::RoutesProductPanelGroup as Routes, schema::admins::id};
use crate::{controllers::createProduct::product, models::components::warning_modal::WarningModal};
use crate::models::components::edit_product::Routes;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;

use super::scope::SCOPE_COMPONENTS;
use crate::services::servicesX::{delete_seller_product,edit_product_general,update_image, update_multiple_images,delete_all_images};
type DbPool = Pool<AsyncPgConnection>;

use std::sync::Arc;
use crate::services::aws::s3::client::Client;
use actix_multipart::{
    form::{
        tempfile::{TempFile, TempFileConfig},
        MultipartForm,
        json::Json as MpJson
    },
    Multipart,
};

static SCOPE: &str = "/edit-product";
static DELETE_PRODUCT : &str = "/delete-product";
lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza
    
    pub static ref ROUTES: Routes = Routes{
        //delete_product: Box::leak(format!("{}{}/delete-product",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        //delete_product_modal: Box::leak(format!("{}{}/delete-product-modal",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        edit_general: Box::leak(format!("{}{}/edit-general",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        edit_images: Box::leak(format!("{}{}/edit-images",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
    };

}

pub fn scope() -> Scope {
    web::scope(SCOPE)
        .configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    //cfg.service(products_panel_prueba);
    cfg
        .service(edit_general)
        .service(delete_all_imgs)
        .service(edit_images);
        //.service(delete_product_modal);
        //.route(format!("{}/{}",DELETE_PRODUCT, "{id}"), web::get().to(products_panel));

}

#[derive(Deserialize)]
pub struct FormGeneral{
    pub name: String,
    pub description: String,
    pub brand: String,
    pub status: i16
}

use crate::models::components::modal_1_model::Modal1;
use crate::models::components::title::Title;

#[post("/edit-general/{product_id}")]
async fn edit_general(path: web::Path<Uuid>,form:web::Form<FormGeneral>,pool_data: web::Data<DbPool>) -> HttpResponse { 
    let product_id = path.into_inner();
    let seller_prueba = Uuid::parse_str("2064d62a-4978-4fe7-bef2-7690ff09bdc8").unwrap();
    // Lógica para borrar el producto
    let form_complete = form.into_inner();
    //let result = delete_seller_product(&product_id,&seller_prueba, pool_data.get_ref()).await;
    let result = edit_product_general(&product_id,form_complete, pool_data.get_ref()).await;
    //TODO: Verificar si el proudcto pertece al vendedor
    match result{
        Ok(_) => {
            println!("Producto editado correctamente");
            let modal_render = Modal1{
                text:"Producto actualizado correctamente!".to_string()
            }.render().unwrap();
            HttpResponse::Ok().body(modal_render)
        },
        Err(e) => {
            println!("Error! -> {}",e);
            let modal_render = Modal1{
                text:"Ha ocurrido un error!".to_string()
            }.render().unwrap();
            return HttpResponse::InternalServerError().body(modal_render);}
        }
}
/* 
#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(limit = "100MB")]
    file: TempFile,
}
*/
#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file")]
    files: Vec<TempFile>,
}


#[derive(Debug)]
pub struct ImageData {
    pub tipo: String,
    pub url: String,
}

#[post("/edit-images/{product_id}")]
async fn edit_images(path: web::Path<Uuid>,client_data: web::Data<Arc<Client>>,
    MultipartForm(form): MultipartForm<UploadForm>,pool_data: web::Data<DbPool>) -> HttpResponse { 
    let product_id = path.into_inner();
    let pool = pool_data.get_ref();
    let client_data_x = client_data.get_ref().as_ref();
    let mut vec_images:Vec<ImageData> = Vec::new();

    for f in form.files {
        let uploaded_file = client_data_x.upload(&f,"").await;
        vec_images.push(ImageData{
            tipo: "principal".to_string(),
            url: uploaded_file.s3_url,
        });
        println!("Archivo subido!");
    }

    /* 
    let texto = format!("Nombre del archivo: {} y link del archivo: {}", uploaded_file.filename, uploaded_file.s3_url);
    let image_url = uploaded_file.s3_url;
    let tipo = "TipoPrueba".to_string();
    /* Prueba */
    let images = vec![
        ImageData {
            tipo: "principal".to_string(),
            url: "https://ejemplo.com/principal.jpg".to_string(),
        },
        ImageData {
            tipo: "secundaria".to_string(),
            url: "https://ejemplo.com/secundaria.jpg".to_string(),
        },
        ImageData {
            tipo: "thumbnail".to_string(),
            url: "https://ejemplo.com/thumbnail.jpg".to_string(),
        },
    ];
    */
    let result_multiple = update_multiple_images(&product_id, &vec_images, pool).await;
    match result_multiple{
        Ok(_) => {
            println!("Producto editado correctamente");
            let modal_render = Modal1{
                text:"Imagen actualizada correctamente!".to_string()
            }.render().unwrap();

            let trigger_value = json!(
                {
                    "image_update":{
                        "target" : ".product-edit-container",
                        "id_value" : product_id
                    }
                }
            );

            HttpResponse::Ok()
                .insert_header(("HX-Trigger",trigger_value.to_string()))
                .body(modal_render)
        },
        Err(e) => {
            println!("Error! -> {}",e);
            let modal_render = Modal1{
                text:"Ha ocurrido un error!".to_string()
            }.render().unwrap();
            return HttpResponse::InternalServerError().body(modal_render);}
        }
    /* Prueba */

    /* 
    let result_update= update_image(&product_id, &tipo, &image_url, pool).await;
    match result_update{
        Ok(_) => {
            println!("Producto editado correctamente");
            let modal_render = Modal1{
                text:"Imagen actualizada correctamente!".to_string()
            }.render().unwrap();
            HttpResponse::Ok().body(modal_render)
        },
        Err(e) => {
            println!("Error! -> {}",e);
            let modal_render = Modal1{
                text:"Ha ocurrido un error!".to_string()
            }.render().unwrap();
            return HttpResponse::InternalServerError().body(modal_render);}
        }
    */
}

#[delete("/delete-all-images/{product_id}")]
async fn delete_all_imgs(path: web::Path<Uuid>,client_data: web::Data<Arc<Client>>,pool_data: web::Data<DbPool>) -> HttpResponse { 
    let product_id = path.into_inner();
    let pool = pool_data.get_ref();
    //let client_data_x = client_data.get_ref().as_ref();
    //let uploaded_file = client_data_x.upload(&form.file,"").await;
  
    let result_delete = delete_all_images(&product_id, pool).await;
    match result_delete{
        Ok(num)=>{
            let resp = format!("Imagenes eliminadas correctamente: {}",num);
            HttpResponse::Ok().body(resp) 
        },
        Err(e) => {
            println!("Error! -> {}",e);
            /*let modal_render = Modal1{
            text:"Ha ocurrido un error!".to_string()
            }.render().unwrap();*/
            HttpResponse::InternalServerError().body("Algo ha ido mal")
        }
    }    
}