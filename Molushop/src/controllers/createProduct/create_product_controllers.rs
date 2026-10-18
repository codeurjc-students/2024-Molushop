use actix_web::{get, post, web,delete,Scope, App, HttpRequest, HttpResponse, HttpServer, Responder, http::StatusCode};
use aws_config::imds::client;
use bigdecimal::BigDecimal;
use lazy_static::lazy_static;
use actix_files as fs;
use serde::Deserialize;
use serde::Serialize;
use std::{str::FromStr, sync::Mutex};
use std::sync::Arc;

use askama::Template;

use uuid::Uuid;

use crate::services::servicesX::*;
use crate::services::panel_role_service::{resolve_store, PanelUser};

use crate::services;
use crate::models::models_x::{ProductForm,Category};
use crate::models::extra::BaseSpecs;
//use crate::models::product_variation::{VariationValue,Attribute};
//use crate::models::get_product::ProductForm;

use std::any::type_name;

use crate::models::components::create_product::*;

use actix_multipart::{
    form::{
        tempfile::{TempFile, TempFileConfig},
        MultipartForm,
        json::Json as MpJson
    },
    Multipart,
};

use serde_json::json;
use std::time::Instant;
use std::collections::HashMap;

//use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pg::AsyncPgConnection;
//use bb8::Pool;
use diesel_async::RunQueryDsl;
//type DbPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
type DbPool = Pool<AsyncPgConnection>;

fn print_type_of<T>(_: &T) {
    println!("El tipo de dato es: {}", type_name::<T>());
}

static SCOPE: &str = "/create-product";
/// Se registra dentro del scope `/panel` (`routes/panel.rs`); `ROUTES` lleva la URL completa.
static SCOPE_FULL: &str = "/panel/create-product";
/// Cabecera con la tienda a cuyo nombre se crea el producto.
pub static STORE_HEADER: &str = "X-Store-Id";
/// La cabecera falta, no es un UUID o es de una tienda que el usuario no gestiona.
static SIN_TIENDA: &str = "No se puede crear el producto: tienda no válida.";

lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza
  
    //IDs de ejemplo
    static ref ID_BASE:Uuid =Uuid::parse_str("95022733-f013-301a-0ada-abc18f151006").unwrap();
    static ref ID_BASE_JOKER:Uuid =Uuid::parse_str("95023733-f013-301a-0ada-abc18f151006").unwrap();  
    /* 
    static ref CATEGORY_CHILDREN_URL: String = format!("{}/category-children", SCOPE);
    static ref RESET_CATEGORY_URL: String = format!("{}/reset-category", SCOPE);
    static ref NEXT_SELECT_URL: String = format!("{}/next-select", SCOPE);
    static ref CREATE_PRODUCT_URL: String = format!("{}/create-product", SCOPE);
    static ref ADD_VARIATION_URL: String = format!("{}/add-variations", SCOPE);
    static ref ADD_VARIATION_VALUE_URL: String = format!("{}/add-variations-attributes", SCOPE);
    static ref ADD_SPECS_URL: String = format!("{}/add-specs", SCOPE);
    */
    pub static ref ROUTES: Routes = Routes{
        category_children: Box::leak(format!("{}/category-children", SCOPE_FULL).into_boxed_str()),
            reset_category: Box::leak(format!("{}/reset-category", SCOPE_FULL).into_boxed_str()),
            next_select: Box::leak(format!("{}/next-select", SCOPE_FULL).into_boxed_str()),
            create_product: Box::leak(format!("{}/create-product", SCOPE_FULL).into_boxed_str()),
            add_specs: Box::leak(format!("{}/add-specs", SCOPE_FULL).into_boxed_str()),
    };

}

pub fn scope_create_product() -> Scope {
    web::scope(SCOPE)
        .configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(category_children);
    cfg.service(reset_category);
    cfg.service(next_category);
    cfg.service(create_product);
    cfg.service(add_specs);
}


#[get("/category-children/{category_id}")]
async fn category_children(path: web::Path<String>, pool_data:web::Data<DbPool>) -> impl Responder {
    let pool = pool_data.get_ref();
    //testeo tiempo
    let now = Instant::now();
    //
    let category_id= path.into_inner();
    let categories_result = obtain_categories_children(&category_id,pool).await;
    let ancestors_result: Result<Vec<crate::models::models_x::Category>, diesel::result::Error> = obtain_ancestors(&category_id,pool).await;
    match (categories_result, ancestors_result) {
        (Ok(categories), Ok(ancestors)) => {

            let list_category_base = List_category_base::new_2(
                ancestors,
                categories,
                &ROUTES
            );
            let rendered = list_category_base.render().unwrap();
            
            ///Testeo tiempo
            let elapsed = now.elapsed();
            println!("Elapsed: {:.2?}", elapsed);
            ///Testeo tiempo
            HttpResponse::Ok().body(rendered)
        },
        (Err(e), Ok(_)) => {
            println!("Error loading categories: {}", e);
            return HttpResponse::InternalServerError().body("Error loading categories");
        },
        (Ok(_), Err(e)) => {
            println!("Error loading ancestors: {}", e);
            return HttpResponse::InternalServerError().body("Error loading ancestors");
        },
        (Err(e1), Err(e2)) => {
            println!("Error loading categories: {}", e1);
            println!("Error loading ancestors: {}", e2);
            return HttpResponse::InternalServerError().body("Error loading categories and ancestors");
        }
    }
}

#[get("/reset-category")]
async fn reset_category(pool_data: web::Data<DbPool>) -> impl Responder {
    let pool = pool_data.get_ref();
    let categories = obtain_base_categories(pool).await;
    match categories{
        Ok(categories) => {
            let base_category = Base_category::new(categories,&ROUTES); 
            let rendered = base_category.render().unwrap();
            HttpResponse::Ok().body(rendered)
        },
        Err(e) => {
            println!("Error loading categories: {}", e);
            return HttpResponse::InternalServerError().body("Error loading categories");
        }
    }
}

#[get("/next-select/{category_id}")]
async fn next_category(path: web::Path<String>) -> impl Responder {
    let category_id= path.into_inner();
    //vamos a obtener el json de la plantilla de la categoría
    //let specs_result= obtain_base_specs(&category_id);
    //let nombre_result = services::product::get_name(&category_id);
    let category_result = services::category::get_category(&category_id);
    match category_result{
        Ok(category) => {
            // Las specs son opcionales: una categoría sin plantilla (o con una
            // que no se entiende) abre el formulario sin specs sugeridas.
            let base_specs = category.base_specs
                .and_then(|specs| match serde_json::from_value::<BaseSpecs>(specs) {
                    Ok(base_specs) => Some(base_specs),
                    Err(e) => {
                        println!("base_specs de {} con formato no válido: {}", category_id, e);
                        None
                    }
                })
                .unwrap_or(BaseSpecs { specs: Vec::new() });
            let base_product = BaseProducto::new(base_specs,category_id, &ROUTES,category.name);
            HttpResponse::Ok().body(base_product.render().unwrap())
        },
        Err(e) => {
            println!("Error loading category: {}", e);
            return HttpResponse::InternalServerError().body("Error loading category");
        }
    }
}

#[post("/create-product/{category_id}")]
async fn create_product(pool_data: web::Data<DbPool>,path:web::Path<String>,panel_user: web::ReqData<PanelUser>,req: HttpRequest,data: web::Json<ProductForm>) -> impl Responder {
    // El producto se crea a nombre de la tienda desde cuya página se manda el
    // formulario (`hx-headers` en `pages/panel_product_create/panel_product_create.html`). La
    // cabecera la pone el cliente, así que se comprueba como la tienda de una URL.
    let store_id = req.headers().get(STORE_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok());
    let seller_id = match store_id {
        Some(store_id) => match resolve_store(&panel_user, &store_id, pool_data.get_ref()).await {
            Ok(Some(store)) => store.id,
            Ok(None) => return HttpResponse::Forbidden().body(SIN_TIENDA),
            Err(e) => {
                println!("Panel: {}", e);
                return HttpResponse::InternalServerError().body("Error al crear el producto");
            }
        },
        None => return HttpResponse::Forbidden().body(SIN_TIENDA),
    };

    let category_id= path.into_inner();
    println!("{:?}",category_id);
    let pool = pool_data.get_ref();
    //print_type_of(&data);
    //hacer print del tipo que es data
    let datox = data.into_inner();
    println!("{:?}",datox);
    //match insert_new_product(&datox,pool).await{
    // Producto, vendedor, categoría, variaciones y precios van en UNA
    // transacción: o se crea todo o no se crea nada.
    match insert_new_product_complete(seller_id,&datox,&category_id,pool).await{
        Ok(new_id) => {
            println!("Producto creado con sus variaciones: {:?}",&new_id);
            HttpResponse::Ok().body("Producto creado")
        },
        Err(crate::models::error::ServiceError::InvalidVariations) => {
            HttpResponse::BadRequest().body("Las variaciones del producto no tienen un formato válido")
        },
        Err(crate::models::error::ServiceError::InvalidSpecs) => {
            HttpResponse::BadRequest().body("Las specs del producto no tienen un formato válido")
        },
        Err(e) => {
            println!("Error al crear el producto (no se ha guardado nada): {}", e);
            HttpResponse::InternalServerError().body("Error al crear el producto")
        }
    }
}

#[get("/add-specs")]
async fn add_specs() -> impl Responder {
    let specs_input = SpecsInput::new();
    let rendered = specs_input.render().unwrap();
    
    HttpResponse::Ok().body(rendered)
}