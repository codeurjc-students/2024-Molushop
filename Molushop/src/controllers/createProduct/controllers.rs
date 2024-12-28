use actix_web::{get, post, web,delete,Scope, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use aws_config::imds::client;
use bigdecimal::BigDecimal;
use lazy_static::lazy_static;
use tera::Tera;
use actix_files as fs;
use serde::Deserialize;
use serde::Serialize;
use std::{str::FromStr, sync::Mutex};
use std::sync::Arc;

use rinja::Template;

use uuid::Uuid;

use crate::services::servicesX::*;
use crate::services;
use crate::models::models_x::{ProductForm,Category};
//use crate::models::get_product::ProductForm;

use std::any::type_name;

use crate::models::htmx::create_product::*;

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


fn print_type_of<T>(_: &T) {
    println!("El tipo de dato es: {}", type_name::<T>());
}

static SCOPE: &str = "/create-product";

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
        category_children: Box::leak(format!("{}/category-children", SCOPE).into_boxed_str()),
            reset_category: Box::leak(format!("{}/reset-category", SCOPE).into_boxed_str()),
            next_select: Box::leak(format!("{}/next-select", SCOPE).into_boxed_str()),
            create_product: Box::leak(format!("{}/create-product", SCOPE).into_boxed_str()),
            add_variation: Box::leak(format!("{}/add-variation", SCOPE).into_boxed_str()),
            add_variation_value: Box::leak(format!("{}/add-variation-value", SCOPE).into_boxed_str()),
            add_specs: Box::leak(format!("{}/add-specs", SCOPE).into_boxed_str()),
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
    cfg.service(add_variation);
    cfg.service(add_variation_value);
    cfg.service(add_specs);
}


#[get("/category-children/{category_id}")]
async fn category_children(path: web::Path<String>) -> impl Responder {
    //testeo tiempo
    let now = Instant::now();
    //
    let category_id= path.into_inner();
    let categories_result = obtain_categories_children(&category_id);
    let ancestors_result: Result<Vec<crate::models::models_x::Category>, diesel::result::Error> = obtain_ancestors(&category_id);
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
async fn reset_category() -> impl Responder {
    let categories = obtain_base_categories();
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
            let specs_option = category.base_specs;
            let name  = category.name.unwrap_or_default();
            match specs_option{
                Some(specs) =>{
                    let base_specs = serde_json::from_value(specs.clone()).unwrap();
                    let base_product = BaseProducto::new(base_specs,category_id, &ROUTES,name);       
                    let render  = base_product.render().unwrap();
                    
                    return HttpResponse::Ok().body(render)
                },
                None => {
                    println!("Error loading specs");
                    return HttpResponse::InternalServerError().body("Error loading specs");
                }
            }

           

        },
        Err(e) => {
            println!("Error loading category: {}", e);
            return HttpResponse::InternalServerError().body("Error loading category");
        }
    }
}

#[post("/create-product/{category_id}")]
async fn create_product(path:web::Path<String>,data: web::Json<ProductForm>) -> impl Responder {
    let category_id= path.into_inner();
    println!("{:?}",category_id);
    
    //print_type_of(&data);
    //hacer print del tipo que es data
    let datox = data.into_inner();
    println!("{:?}",datox);
    match insert_new_product(&datox){
        Ok(_) => {
            println!("Producto insertado");
            HttpResponse::Ok().body("Producto creado")
        },
        Err(e) => {
            println!("Error al insertar producto: {}", e);
            return HttpResponse::InternalServerError().body("Error al insertar producto");
        }
    }
}

#[get("/add-variation")]
async fn add_variation() -> impl Responder {
    //let context = tera::Context::new();
    let variations_input = VariationsInput::new(&ROUTES);
    let rendered = variations_input.render().unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/add-variation-value")] //ESTO QUE HACE?
async fn add_variation_value() -> impl Responder {
    // let context = tera::Context::new();
    //let rendered = TEMPLATES.render("create_product/variations-input-extra.html", &context).unwrap();
    let variations_input_extra = VariationsInputExtra::new();
    let rendered = variations_input_extra.render().unwrap();
    HttpResponse::Ok()
        .insert_header(("HX-Trigger","update_num"))
        .body(rendered)
}

#[get("/add-specs")]
async fn add_specs() -> impl Responder {
    let specs_input = SpecsInput::new();
    let rendered = specs_input.render().unwrap();
    
    HttpResponse::Ok().body(rendered)
}