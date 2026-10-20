use actix_web::{get, post, web,delete,Scope, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use diesel::result;
use lazy_static::lazy_static;
use askama::Template;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::json;

//use crate::{models::data_transfer_objects::product::RoutesProductPanelGroup as Routes, schema::admins::id};
use crate::{controllers::createProduct::product, models::components::confirm_modal::ConfirmModal};
use crate::models::components::edit_product::Routes;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;

use super::scope::SCOPE_PANEL_COMPONENTS;
use super::panel_access::product_access;
use crate::services::panel_role_service::PanelUser;
use crate::services::servicesX::{edit_product_general,update_image, update_multiple_images_2,delete_all_images};
use crate::services::variations_bulk_service::{bulk_update_variations, BulkAction};
use crate::services::image_variations_service::set_image_variations;
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
        edit_general: Box::leak(format!("{}{}/edit-general",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        edit_images: Box::leak(format!("{}{}/edit-images",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        bulk_variations: Box::leak(format!("{}{}/bulk-variations",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        image_variations: Box::leak(format!("{}{}/image-variations",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
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
        .service(bulk_variations)
        .service(image_variations)
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

use crate::models::components::message_modal::MessageModal;

#[post("/edit-general/{product_id}")]
async fn edit_general(path: web::Path<Uuid>,panel_user: web::ReqData<PanelUser>,form:web::Form<FormGeneral>,pool_data: web::Data<DbPool>) -> HttpResponse {
    let product_id = path.into_inner();
    if let Err(resp) = product_access(&panel_user, &product_id, pool_data.get_ref()).await {
        return resp;
    }
    let form_complete = form.into_inner();
    let status = form_complete.status;
    let result = edit_product_general(&product_id,form_complete, pool_data.get_ref()).await;
match result{
        Ok(_) => {
            let modal_render = MessageModal{
                text:"Producto actualizado correctamente!".to_string()
            }.render().unwrap();
            // La cabecera del editor se actualiza con lo guardado (edit-product.js).
            // Solo va el estado: el nombre y la marca los lee del formulario, y
            // así la cabecera HTTP no lleva texto del vendedor.
            let trigger = json!({ "product_update": { "status": status } });
            HttpResponse::Ok()
                .insert_header(("HX-Trigger", trigger.to_string()))
                .body(modal_render)
        },
        Err(e) => {
            println!("Error! -> {}",e);
            let modal_render = MessageModal{
                text:"Ha ocurrido un error!".to_string()
            }.render().unwrap();
            return HttpResponse::InternalServerError().body(modal_render);}
        }
}
#[derive(Deserialize)]
pub struct BulkForm{
    pub action: String,
    pub value: String,
    /// Ids de las variaciones filtradas, separados por comas.
    pub ids: String,
}

/// Edición en bloque desde el explorador de variaciones. Responde siempre 200
/// con el aviso (htmx no pinta las respuestas 4xx/5xx) y, si ha ido bien, con
/// `variations_bulk_update`: cómo ha quedado cada fila, para el JS del explorador.
#[post("/bulk-variations/{product_id}")]
async fn bulk_variations(path: web::Path<Uuid>,panel_user: web::ReqData<PanelUser>,form:web::Form<BulkForm>,pool_data: web::Data<DbPool>) -> HttpResponse {
    let product_id = path.into_inner();
    let pool = pool_data.get_ref();
    if let Err(resp) = product_access(&panel_user, &product_id, pool).await {
        return resp;
    }
    let modal = |text: &str| MessageModal::new(text).render().unwrap();

    let action = match BulkAction::parse(&form.action, &form.value) {
        Ok(action) => action,
        Err(text) => return HttpResponse::Ok().body(modal(text)),
    };
    let ids: Result<Vec<Uuid>, _> = form.ids.split(',')
        .filter(|id| !id.trim().is_empty())
        .map(|id| Uuid::parse_str(id.trim()))
        .collect();
    let ids = match ids {
        Ok(ids) if !ids.is_empty() => ids,
        _ => return HttpResponse::Ok().body(modal("No hay variaciones seleccionadas")),
    };

    match bulk_update_variations(&product_id, &ids, &action, pool).await {
        Ok(Some(rows)) => {
            let text = if rows.len() == 1 {
                "Se ha actualizado 1 variación".to_string()
            } else {
                format!("Se han actualizado {} variaciones", rows.len())
            };
            let trigger = json!({ "variations_bulk_update": { "rows": rows } });
            HttpResponse::Ok()
                .insert_header(("HX-Trigger", trigger.to_string()))
                .body(modal(&text))
        }
        Ok(None) => HttpResponse::Ok().body(modal("Alguna variación no es de este producto: no se ha cambiado nada")),
        Err(e) => {
            println!("Panel: error en la edición en bloque de {}: {}", product_id, e);
            HttpResponse::Ok().body(modal("Ha ocurrido un error: no se ha cambiado nada"))
        }
    }
}

#[derive(Deserialize)]
pub struct ImageVariationsForm{
    pub image_id: Uuid,
    /// Ids de las variaciones en las que sale, separados por comas; vacío: en todas.
    pub ids: String,
}

/// En qué variaciones sale una imagen (pestaña Imágenes). Como la edición en
/// bloque: siempre 200 con el aviso y, si ha ido bien, `image_variations_update`
/// con los ids guardados para el JS de la galería (image-variations.js).
#[post("/image-variations/{product_id}")]
async fn image_variations(path: web::Path<Uuid>,panel_user: web::ReqData<PanelUser>,form:web::Form<ImageVariationsForm>,pool_data: web::Data<DbPool>) -> HttpResponse {
    let product_id = path.into_inner();
    let pool = pool_data.get_ref();
    if let Err(resp) = product_access(&panel_user, &product_id, pool).await {
        return resp;
    }
    let modal = |text: &str| MessageModal::new(text).render().unwrap();

    let ids: Result<Vec<Uuid>, _> = form.ids.split(',')
        .filter(|id| !id.trim().is_empty())
        .map(|id| Uuid::parse_str(id.trim()))
        .collect();
    let Ok(ids) = ids else {
        return HttpResponse::Ok().body(modal("Variaciones no válidas: no se ha cambiado nada"));
    };

    match set_image_variations(&product_id, &form.image_id, &ids, pool).await {
        Ok(true) => {
            let text = match ids.len() {
                0 => "La imagen sale ahora en todas las variaciones".to_string(),
                1 => "La imagen sale ahora en 1 variación".to_string(),
                n => format!("La imagen sale ahora en {} variaciones", n),
            };
            let trigger = json!({ "image_variations_update": { "image_id": form.image_id, "ids": ids } });
            HttpResponse::Ok()
                .insert_header(("HX-Trigger", trigger.to_string()))
                .body(modal(&text))
        }
        Ok(false) => HttpResponse::Ok().body(modal("La imagen o alguna variación no son de este producto: no se ha cambiado nada")),
        Err(e) => {
            println!("Panel: error al asignar variaciones a la imagen {}: {}", form.image_id, e);
            HttpResponse::Ok().body(modal("Ha ocurrido un error: no se ha cambiado nada"))
        }
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

#[derive(Debug)]
pub struct ImageData2{
    pub url: String,
    pub main: bool,
}
use crate::models::models_x::NewImageProduct2;
#[post("/edit-images/{product_id}")]
async fn edit_images(path: web::Path<Uuid>,panel_user: web::ReqData<PanelUser>,client_data: web::Data<Arc<Client>>,
    MultipartForm(form): MultipartForm<UploadForm>,pool_data: web::Data<DbPool>) -> HttpResponse {
    let product_id = path.into_inner();
    let pool = pool_data.get_ref();
    // Antes de subir nada a S3.
    if let Err(resp) = product_access(&panel_user, &product_id, pool).await {
        return resp;
    }
let client_data_x = client_data.get_ref().as_ref();
    let mut vec_images:Vec<ImageData> = Vec::new();
    let mut vec_images2:Vec<NewImageProduct2> = Vec::new();

    for f in form.files {
        let uploaded_file = client_data_x.upload(&f,"").await;
        vec_images2.push(NewImageProduct2{
            id: Uuid::new_v4(),
            product_id: product_id.clone(),
            image_url: uploaded_file.s3_url.clone(),
            is_main: false,
            display_order: 1
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
    let result_multiple = update_multiple_images_2(vec_images2, pool).await;
    match result_multiple{
        Ok(_) => {
            println!("Producto editado correctamente");
            let modal_render = MessageModal{
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
            let modal_render = MessageModal{
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
            let modal_render = MessageModal{
                text:"Imagen actualizada correctamente!".to_string()
            }.render().unwrap();
            HttpResponse::Ok().body(modal_render)
        },
        Err(e) => {
            println!("Error! -> {}",e);
            let modal_render = MessageModal{
                text:"Ha ocurrido un error!".to_string()
            }.render().unwrap();
            return HttpResponse::InternalServerError().body(modal_render);}
        }
    */
}

#[delete("/delete-all-images/{product_id}")]
async fn delete_all_imgs(path: web::Path<Uuid>,panel_user: web::ReqData<PanelUser>,client_data: web::Data<Arc<Client>>,pool_data: web::Data<DbPool>) -> HttpResponse {
    let product_id = path.into_inner();
    let pool = pool_data.get_ref();
    if let Err(resp) = product_access(&panel_user, &product_id, pool).await {
        return resp;
    }
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
            /*let modal_render = MessageModal{
            text:"Ha ocurrido un error!".to_string()
            }.render().unwrap();*/
            HttpResponse::InternalServerError().body("Algo ha ido mal")
        }
    }    
}