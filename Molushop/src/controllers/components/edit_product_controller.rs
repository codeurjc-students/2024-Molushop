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
use crate::services::servicesX::{edit_product_general,update_image, update_multiple_images_2};
use crate::services::variations_bulk_service::{bulk_update_variations, BulkAction, BulkOutcome};
use crate::services::image_variations_service::set_image_variations;
use crate::services::product_images_service::{set_main_image, delete_image as delete_product_image};
use crate::services::product_specs_service::save_specs;
use crate::models::error::ServiceError;
use crate::services::sku_service::{save_skus, SaveSkus};
use crate::utils::hx::{error_modal, error_response, trigger_json};
use actix_web::http::StatusCode as Status;
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
        bulk_skus: Box::leak(format!("{}{}/bulk-skus",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        image_variations: Box::leak(format!("{}{}/image-variations",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        main_image: Box::leak(format!("{}{}/main-image",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        edit_specs: Box::leak(format!("{}{}/edit-specs",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        delete_image_modal: Box::leak(format!("{}{}/delete-image-modal",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        delete_image: Box::leak(format!("{}{}/delete-image",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
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
        .service(main_image)
        .service(edit_specs)
        .service(bulk_skus)
        .service(delete_image_modal)
        .service(delete_image)
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
            return error_modal(Status::INTERNAL_SERVER_ERROR, "Ha ocurrido un error: no se ha guardado nada");}
        }
}
#[derive(Deserialize)]
pub struct BulkForm{
    pub action: String,
    pub value: String,
    /// Ids de las variaciones filtradas, separados por comas.
    pub ids: String,
}

/// Edición en bloque desde el explorador de variaciones. Responde con el aviso
/// (los errores con su código, `error_modal`) y, si ha ido bien, con
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
        Err(text) => return error_modal(Status::BAD_REQUEST, text),
    };
    let ids: Result<Vec<Uuid>, _> = form.ids.split(',')
        .filter(|id| !id.trim().is_empty())
        .map(|id| Uuid::parse_str(id.trim()))
        .collect();
    let ids = match ids {
        Ok(ids) if !ids.is_empty() => ids,
        _ => return error_modal(Status::BAD_REQUEST, "No hay variaciones seleccionadas"),
    };

    match bulk_update_variations(&product_id, &ids, &action, pool).await {
        Ok(BulkOutcome::Updated(rows)) => {
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
        Ok(BulkOutcome::NotOwned) => error_modal(Status::BAD_REQUEST, "Alguna variación no es de este producto: no se ha cambiado nada"),
        Ok(BulkOutcome::MissingPrice(n)) => {
            let text = if n == 1 {
                "1 de las variaciones no tiene precio y no se puede activar. Ponle precio antes (Fijar precio) o quítala con los filtros: no se ha cambiado nada".to_string()
            } else {
                format!("{} de las variaciones no tienen precio y no se pueden activar. Ponles precio antes (Fijar precio) o quítalas con los filtros: no se ha cambiado nada", n)
            };
            error_modal(Status::CONFLICT, &text)
        }
        Err(e) => {
            println!("Panel: error en la edición en bloque de {}: {}", product_id, e);
            error_modal(Status::INTERNAL_SERVER_ERROR, "Ha ocurrido un error: no se ha cambiado nada")
        }
    }
}

#[derive(Deserialize)]
pub struct SkusForm{
    /// JSON: `[{"id": "...", "sku": "..."}]`, solo las variaciones cambiadas.
    pub changes: String,
}

#[derive(Deserialize)]
struct SkuChange{
    id: Uuid,
    sku: String,
}

/// Los SKU editados en la tabla de variaciones, todos o ninguno. Responde con
/// el aviso (los errores con su código) y un evento para el JS del explorador:
/// `variations_skus_saved` con las filas, o `variations_skus_problems` con lo
/// que hay que marcar en cada una.
#[post("/bulk-skus/{product_id}")]
async fn bulk_skus(path: web::Path<Uuid>,panel_user: web::ReqData<PanelUser>,form:web::Form<SkusForm>,pool_data: web::Data<DbPool>) -> HttpResponse {
    let product_id = path.into_inner();
    let pool = pool_data.get_ref();
    if let Err(resp) = product_access(&panel_user, &product_id, pool).await {
        return resp;
    }
    let modal = |text: &str| MessageModal::new(text).render().unwrap();

    let changes: Vec<(Uuid, String)> = match serde_json::from_str::<Vec<SkuChange>>(&form.changes) {
        Ok(changes) if !changes.is_empty() => changes.into_iter().map(|c| (c.id, c.sku)).collect(),
        _ => return error_modal(Status::BAD_REQUEST, "No hay SKU que guardar"),
    };

    match save_skus(&product_id, &changes, pool).await {
        Ok(SaveSkus::Saved(rows)) => {
            let text = if rows.len() == 1 {
                "Se ha guardado 1 SKU".to_string()
            } else {
                format!("Se han guardado {} SKU", rows.len())
            };
            let trigger = json!({ "variations_skus_saved": { "rows": rows } });
            HttpResponse::Ok()
                .insert_header(("HX-Trigger", trigger_json(&trigger)))
                .body(modal(&text))
        }
        Ok(SaveSkus::Problems(problems)) => {
            let text = match problems.len() {
                0 => "Algún SKU lo acaba de coger otra variación: no se ha guardado nada. Vuelve a intentarlo".to_string(),
                1 => "Hay 1 SKU que no se puede guardar (marcado en la tabla): no se ha guardado nada".to_string(),
                n => format!("Hay {} SKU que no se pueden guardar (marcados en la tabla): no se ha guardado nada", n),
            };
            // Los mensajes llevan el nombre del producto: cabecera en ASCII
            let trigger = json!({ "variations_skus_problems": { "problems": problems } });
            error_response(Status::CONFLICT)
                .insert_header(("HX-Trigger", trigger_json(&trigger)))
                .body(modal(&text))
        }
        Ok(SaveSkus::NotOwned) => error_modal(Status::BAD_REQUEST, "Alguna variación no es de este producto: no se ha cambiado nada"),
        Err(e) => {
            println!("Panel: error al guardar los SKU de {}: {}", product_id, e);
            error_modal(Status::INTERNAL_SERVER_ERROR, "Ha ocurrido un error: no se ha guardado nada")
        }
    }
}

#[derive(Deserialize)]
pub struct SpecsForm{
    /// `specs[Peso]=...` del formulario (json-enc-custom). Sin ninguna, el
    /// formulario no manda la clave: se quedan todas borradas.
    #[serde(default)]
    pub specs: serde_json::Value,
}

/// Pestaña Especificaciones. Responde con el aviso (los errores con su código).
#[post("/edit-specs/{product_id}")]
async fn edit_specs(path: web::Path<Uuid>,panel_user: web::ReqData<PanelUser>,form:web::Json<SpecsForm>,pool_data: web::Data<DbPool>) -> HttpResponse {
    let product_id = path.into_inner();
    let pool = pool_data.get_ref();
    if let Err(resp) = product_access(&panel_user, &product_id, pool).await {
        return resp;
    }
    let modal = |text: &str| HttpResponse::Ok().body(MessageModal::new(text).render().unwrap());
    match save_specs(&product_id, &form.specs, pool).await {
        Ok(()) => modal("Especificaciones guardadas"),
        Err(ServiceError::InvalidSpecs) => error_modal(Status::BAD_REQUEST, "Las especificaciones no tienen un formato válido: no se ha guardado nada"),
        Err(e) => {
            println!("Panel: error al guardar las specs de {}: {}", product_id, e);
            error_modal(Status::INTERNAL_SERVER_ERROR, "Ha ocurrido un error: no se ha guardado nada")
        }
    }
}

#[derive(Deserialize)]
pub struct MainImageForm{
    pub image_id: Uuid,
}

/// "Hacer principal" en la pestaña Imágenes. Responde con el aviso (los errores
/// con su código) y, si ha ido bien, `main_image_update` para que la galería y la cabecera del editor
/// cambien sin recargar (main-image.js).
#[post("/main-image/{product_id}")]
async fn main_image(path: web::Path<Uuid>,panel_user: web::ReqData<PanelUser>,form:web::Form<MainImageForm>,pool_data: web::Data<DbPool>) -> HttpResponse {
    let product_id = path.into_inner();
    let pool = pool_data.get_ref();
    if let Err(resp) = product_access(&panel_user, &product_id, pool).await {
        return resp;
    }
    let modal = |text: &str| MessageModal::new(text).render().unwrap();
    match set_main_image(&product_id, &form.image_id, pool).await {
        Ok(true) => {
            let trigger = json!({ "main_image_update": { "image_id": form.image_id } });
            HttpResponse::Ok()
                .insert_header(("HX-Trigger", trigger.to_string()))
                .body(modal("Imagen principal cambiada"))
        }
        Ok(false) => error_modal(Status::BAD_REQUEST, "Esa imagen no es de este producto: no se ha cambiado nada"),
        Err(e) => {
            println!("Panel: error al cambiar la imagen principal de {}: {}", product_id, e);
            error_modal(Status::INTERNAL_SERVER_ERROR, "Ha ocurrido un error: no se ha cambiado nada")
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
/// bloque: el aviso (los errores con su código) y, si ha ido bien, `image_variations_update`
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
        return error_modal(Status::BAD_REQUEST, "Variaciones no válidas: no se ha cambiado nada");
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
        Ok(false) => error_modal(Status::BAD_REQUEST, "La imagen o alguna variación no son de este producto: no se ha cambiado nada"),
        Err(e) => {
            println!("Panel: error al asignar variaciones a la imagen {}: {}", form.image_id, e);
            error_modal(Status::INTERNAL_SERVER_ERROR, "Ha ocurrido un error: no se ha cambiado nada")
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
            return error_modal(Status::INTERNAL_SERVER_ERROR, "Ha ocurrido un error: no se ha guardado nada");}
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
            return error_modal(Status::INTERNAL_SERVER_ERROR, "Ha ocurrido un error: no se ha guardado nada");}
        }
    */
}

#[derive(Deserialize)]
pub struct DeleteImageQuery{
    pub image_id: Uuid,
}

/// Papelera de una imagen guardada: primero esta confirmación, cuyo "Sí" lanza
/// `delete_image`. La respuesta salta en el botón del modal, así que el JS
/// (images.js) busca la imagen por el id que va en `image_deleted`.
#[get("/delete-image-modal/{product_id}")]
async fn delete_image_modal(path: web::Path<Uuid>,query: web::Query<DeleteImageQuery>) -> HttpResponse {
    let modal = ConfirmModal{
        method: "hx-delete".to_string(),
        message: "¿Borrar esta imagen? No se puede deshacer.".to_string(),
        endpoint: format!("{}/{}?image_id={}", ROUTES.delete_image, path.into_inner(), query.image_id),
        target: "body".to_string(),
        swap: "beforeend".to_string(),
        hyperscript_action: String::new(),
        htmx_active: true,
    };
    HttpResponse::Ok().body(modal.render().unwrap())
}

/// Borra la imagen de la BD y, si nadie más usa la URL (ni otra imagen ni un
/// pedido), su fichero de S3. S3 va después de guardar: si falla, la imagen ya
/// no sale en la tienda y el fichero queda huérfano (se apunta en el log).
#[delete("/delete-image/{product_id}")]
async fn delete_image(path: web::Path<Uuid>,query: web::Query<DeleteImageQuery>,panel_user: web::ReqData<PanelUser>,
    client_data: web::Data<Arc<Client>>,pool_data: web::Data<DbPool>) -> HttpResponse {
    let product_id = path.into_inner();
    let pool = pool_data.get_ref();
    if let Err(resp) = product_access(&panel_user, &product_id, pool).await {
        return resp;
    }

    let deleted = match delete_product_image(&product_id, &query.image_id, pool).await {
        Ok(Some(deleted)) => deleted,
        Ok(None) => return error_modal(Status::NOT_FOUND, "Esa imagen ya no existe o no es de este producto"),
        Err(e) => {
            println!("Panel: error al borrar la imagen {}: {}", query.image_id, e);
            return error_modal(Status::INTERNAL_SERVER_ERROR, "Ha ocurrido un error: no se ha borrado nada");
        }
    };

    if deleted.file_unused {
        let client = client_data.get_ref().as_ref();
        match client.key_from_url(&deleted.image_url) {
            Some(key) => if !client.delete_file(&key).await {
                println!("Panel: no se ha podido borrar de S3 {} (queda huérfano)", key);
            },
            None => println!("Panel: {} no es del bucket, no se borra de S3", deleted.image_url),
        }
    }

    let trigger = json!({ "image_deleted": { "image_id": query.image_id, "new_main": deleted.new_main } });
    HttpResponse::Ok()
        .insert_header(("HX-Trigger", trigger.to_string()))
        .body(MessageModal::new("Imagen borrada").render().unwrap())
}