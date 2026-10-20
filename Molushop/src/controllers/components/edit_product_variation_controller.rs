use actix_web::{get, post, web,delete,Scope, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use bigdecimal::BigDecimal;
use diesel::result;
use lazy_static::lazy_static;
use askama::Template;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::json;

//use crate::{models::data_transfer_objects::product::RoutesProductPanelGroup as Routes, schema::admins::id};
use crate::{controllers::createProduct::product, models::components::confirm_modal::ConfirmModal};
use crate::models::components::edit_product_variation_model::Routes;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use super::scope::SCOPE_PANEL_COMPONENTS;
use super::panel_access::variation_access;
use crate::services::panel_role_service::PanelUser;
use crate::services::servicesX::{set_variation_identifiers,update_price_variation,set_stock_variation};
use crate::services::sku_service::{normalize_sku, save_variation_general, SaveGeneral};

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

static SCOPE: &str = "/edit-product-variation";

lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza
    
    pub static ref ROUTES: Routes = Routes{
        //delete_product: Box::leak(format!("{}{}/delete-product",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        //delete_product_modal: Box::leak(format!("{}{}/delete-product-modal",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        edit_general: Box::leak(format!("{}{}/edit-general",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        edit_price: Box::leak(format!("{}{}/edit-price",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        edit_stock: Box::leak(format!("{}{}/edit-stock",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        new_identifier: Box::leak(format!("{}{}/new-identifier",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        edit_identifiers: Box::leak(format!("{}{}/edit-identifiers",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        warning: Box::leak(format!("{}{}/pop-warning",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        delete_identifier: Box::leak(format!("{}{}/delete-identifier",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        //edit_images: Box::leak(format!("{}{}/edit-images",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
    };

}

pub fn scope() -> Scope {
    web::scope(SCOPE)
        .configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    //cfg.service(products_panel_prueba);
    cfg
        .service(spawn)
        .service(edit_identifiers)
        .service(pop_warning)
        .service(new_identifier)
        .service(edit_general)
        .service(edit_price)
        .service(edit_stock)
        .service(delete_identifier);
        //.service(delete_product_modal);
        //.route(format!("{}/{}",DELETE_PRODUCT, "{id}"), web::get().to(products_panel));

}
use crate::models::components::edit_product_variation_model::EditProductVariation;
use crate::services::components::edit_product_variation_service::start;
use crate::models::components::message_modal::MessageModal;

#[get("/spawn/{id}")]
async fn spawn(path: web::Path<Uuid>, panel_user: web::ReqData<PanelUser>, pool_data: web::Data<DbPool>) -> HttpResponse {
    let variation_id = path.into_inner();
    let pool = pool_data.get_ref();
    if let Err(resp) = variation_access(&panel_user, &variation_id, pool).await {
        return resp;
    }
    let result_start=  start(&variation_id,&pool).await;
    match result_start{
        Ok(product_variation)=>{
            //tengo el ProductVariationEdit
            //ahora a cargar el producto con las rutas destinadas
            let edit_product_variation = EditProductVariation{
                variation:product_variation,
                routes_edit_product:&ROUTES
            }.render().unwrap();
            return HttpResponse::Ok().body(edit_product_variation)
        },
        Err(e)=>{
            println!("Error: {}",e);
            let modal_render = MessageModal{
                text:"Ha ocurrido un error!".to_string()
            }.render().unwrap();
            return HttpResponse::InternalServerError().body(modal_render)
        }
    }
}

//parametro con el endpoint correspondiente??
#[get("/new-identifier")]
async fn new_identifier(pool_data:web::Data<DbPool>)->HttpResponse{
    use crate::models::components::identifier_model::IdentifierBase;
    let pool = pool_data.get_ref();
    let identifier_x = IdentifierBase::base(pool).await;
    let render = identifier_x.render().unwrap();
    return HttpResponse::Ok().body(render);
}

/* 
pub struct FormIdentifier{
    identifiers: Vec<Identifier>
} 

pub struct Identifier{
    name: String,
    value: String
}*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeneralForm{
    // Un producto sin variaciones tiene una única variación sin atributos: el
    // formulario no manda ningún `attributes[...]` y la clave ni aparece en el JSON.
    #[serde(default)]
    pub attributes: Vec<AttributeForm>,
    pub status: String,
    #[serde(default)]
    pub sku: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttributeForm{
    name:String,
    value:String
}


#[post("/edit-general/{id_variation}")]
async fn edit_general(pool_data:web::Data<DbPool>,path:web::Path<Uuid>,panel_user: web::ReqData<PanelUser>,form: web::Json<GeneralForm>)-> HttpResponse{
    let form_general = form.into_inner();
    let id_variation = path.into_inner();
    let pool = pool_data.get_ref();
    if let Err(resp) = variation_access(&panel_user, &id_variation, pool).await {
        return resp;
    }
    // Los avisos van con 200: htmx no pinta las respuestas 4xx/5xx.
    let modal = |text: &str| HttpResponse::Ok().body(MessageModal::new(text).render().unwrap());
    let new_status:i16 =  form_general.status.parse().unwrap_or(0);
    let sku = match normalize_sku(&form_general.sku) {
        Ok(sku) => sku,
        Err(text) => return modal(text),
    };
    match save_variation_general(&id_variation, new_status, sku.clone(), pool).await {
        Ok(SaveGeneral::Saved) => {
            let trigger_value = json!(
                {
                    "variation_update": {
                        "target": ".variation-edit-container",
                        "status": form_general.status,
                        "sku": sku.unwrap_or_default()
                    }
                }
            );
            HttpResponse::Ok()
                .insert_header(("HX-Trigger",trigger_value.to_string()))
                .body(MessageModal::new("Variación guardada").render().unwrap())
        }
        Ok(SaveGeneral::SkuTaken(holder)) => {
            modal(&format!("Ese SKU ya lo usa {}: no se ha guardado nada", holder))
        }
        Err(e) => {
            println!("Ha ocurrido un error: {}",e);
            modal("Algo ha ido mal: no se ha guardado nada")
        }
    }
    //aqui añadir headers a la respuesta
    /* 
    println!("{:?}",form_general);
    let modal_respuesta = MessageModal{
        text:"Prueba de edit_general, hecha correctamente".to_string()
    }.render().unwrap();
    HttpResponse::Ok().body(modal_respuesta)*/
}

use crate::models::components::edit_product_variation_model::Identifier;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdentifersForm{
    pub identifiers: Vec<Identifier>,
}

#[post("/edit-identifiers/{id_variation}")]
async fn edit_identifiers(pool_data: web::Data<DbPool>,path:web::Path<Uuid>,panel_user: web::ReqData<PanelUser>,data: web::Json<IdentifersForm>)->HttpResponse{
    //nos van a pasar en el form un json
    let var_id = path.into_inner();
    let pool =pool_data.get_ref();
    if let Err(resp) = variation_access(&panel_user, &var_id, pool).await {
        return resp;
    }
let form_identifiers = data.into_inner();
    let identifiers = form_identifiers.identifiers;
    let result =set_variation_identifiers(&identifiers, &var_id, pool).await;

    match result{
        Ok(numero) =>{
            //aqui añadir a que 
            let modal_respuesta = MessageModal{
                text:"Identificadores editados correctamente".to_string()
            }.render().unwrap();
            println!("Todo correcto");
            HttpResponse::Ok().body(modal_respuesta)
        },
        Err(e)=>{
            println!("Ha ocurrido un error: {}",e);
            let modal_respuesta = MessageModal{
                text:"Ha habido un error al editar los identificadores".to_string()
            }.render().unwrap();
            println!("Todo incorrecto");
            HttpResponse::Ok().body(modal_respuesta)
        }
    }

    //ahora falta editar en la base de datos
    
}
//obtener los parametros 
use crate::models::components::edit_product_variation_model::*;

#[post("/edit-price/{id_variation}")]
async fn edit_price(pool_data:web::Data<DbPool>,path:web::Path<Uuid>,panel_user: web::ReqData<PanelUser>,data:web::Json<PriceForm>)->HttpResponse{
    let form  = data.into_inner();

    let var_id  = path.into_inner();
    let pool = pool_data.get_ref();
    if let Err(resp) = variation_access(&panel_user, &var_id, pool).await {
        return resp;
    }
// Convertir los valores string a BigDecimal
    let regular_price = match form.regular.parse::<BigDecimal>() {
        Ok(price) => price,
        Err(e) => {
            println!("Error al convertir precio regular: {}", e);
            return HttpResponse::BadRequest().body("Precio regular inválido");
        }
    };
    let discounted_price = match form.discount.parse::<BigDecimal>() {
        Ok(price) => price,
        Err(e) => {
            println!("Error al convertir precio regular: {}", e);
            return HttpResponse::BadRequest().body("Precio Descontado inválido");
        }
    };
    let is_active = match form.discount_active.parse::<bool>() {
        Ok(value) => value,
        Err(e) => {
            println!("{}",e);
            false // valor predeterminado
        }
    };
    let option_value: i16 = form.option.parse().unwrap_or(0);
    let form_new = PriceData{
        regular:regular_price.clone(),
        discount_active:is_active,
        discount:discounted_price,
        option:option_value
    };

    // Moneda predeterminada
    let currency = "EUR".to_string();

    // Llamar al servicio para actualizar el precio
    let result_update_price = update_price_variation(&form_new,&var_id, &regular_price, &currency, pool).await;

    match result_update_price {
        Ok(_) => {
            let modal_respuesta = MessageModal {
                text: "Precio actualizado correctamente".to_string()
            }.render().unwrap();
            let trigger = json!({
                "variation_update": { "target": ".variation-edit-container", "price": regular_price.to_string() }
            });
            HttpResponse::Ok()
                .insert_header(("HX-Trigger", trigger.to_string()))
                .body(modal_respuesta)
        },
        Err(e) => {
            println!("Error al actualizar precio: {}", e);
            let modal_respuesta = MessageModal {
                text: "Error al actualizar el precio".to_string()
            }.render().unwrap();
            HttpResponse::InternalServerError().body(modal_respuesta)
        }
    }
}

#[post("/edit-stock/{variation_id}")]
async fn edit_stock(pool_data:web::Data<DbPool>,path:web::Path<Uuid>,panel_user: web::ReqData<PanelUser>,data:web::Json<StockForm>)->HttpResponse{
    let pool = pool_data.get_ref();
    let var_id = path.into_inner();
    if let Err(resp) = variation_access(&panel_user, &var_id, pool).await {
        return resp;
    }
let form = data.into_inner();
    // 200 con el aviso: htmx no pinta las respuestas 4xx/5xx
    let new_stock= match form.stock.parse::<i32>(){
        Ok(stock) if stock >= 0 => stock,
        _ => {
            let modal_respuesta = MessageModal::new("El stock debe ser un número entero, 0 o más").render().unwrap();
            return HttpResponse::Ok().body(modal_respuesta)
        }
    };


    let result_stock = set_stock_variation(&var_id, &new_stock, &pool).await;
    match result_stock{
        Ok(_)=>{
            let modal_respuesta = MessageModal::new("Stock actualizado").render().unwrap();
            let trigger = json!({
                "variation_update": { "target": ".variation-edit-container", "stock": new_stock }
            });
            return HttpResponse::Ok()
                .insert_header(("HX-Trigger", trigger.to_string()))
                .body(modal_respuesta)
        },
        Err(e)=>{
            println!("{}",e);
            let modal_respuesta = MessageModal::new("Ha ocurrido un error").render().unwrap();
            return HttpResponse::InternalServerError().body(modal_respuesta)
        }
    }


}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WarningParameters{
    pub method:String,
    pub message:String,
    pub endpoint:String,
    pub target:String,
    pub swap:String,
    pub hyperscript_action:String,
    pub htmx_active:bool
}


#[get("/pop-warning")]
async fn pop_warning(params: web::Query<WarningParameters>)->HttpResponse{
    //obtener el warning
    /* 
    let warning_modal = ConfirmModal{
        method:params.method.clone(),
        message:"Seguro que quieres eliminar el identificador?".to_string(),
        endpoint:params.endpoint.clone(),
        target:params.target.clone(),
        swap:params.swap.clone(),
        htmx_active:false,
    }.render().unwrap();
    remove target
    */
    println!("{:?}",params);
    let warning_modal = ConfirmModal{
        method:params.method.clone(),
        message:params.message.clone(),
        endpoint:params.endpoint.clone(),
        target:params.target.clone(),
        swap:params.swap.clone(),
        hyperscript_action: params.hyperscript_action.clone(), 
        htmx_active:params.htmx_active,
    }.render().unwrap();

    HttpResponse::Ok().body(warning_modal)
}

#[delete("/delete-identifier/{id_variation}")]
async fn delete_identifier(pool_data:web::Data<DbPool>)->HttpResponse{
    println!("BORRANDO IDENTIFICADOR");
    HttpResponse::Ok().finish()
}

/* 
#[post("/create-product/{category_id}")]
async fn create_product(pool_data: web::Data<DbPool>,path:web::Path<String>,data: web::Json<ProductForm>) -> impl Responder {
*/