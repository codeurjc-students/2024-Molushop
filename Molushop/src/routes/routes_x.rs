// paginas de la aplicación
use actix_web::{get, post, web,delete, App, HttpResponse,HttpRequest, HttpServer, Responder, http::StatusCode};
use bigdecimal::BigDecimal;
use lazy_static::lazy_static;
use tera::Tera;
use actix_files as fs;
use serde::Deserialize;
use serde::Serialize;
use uuid::timestamp::context;
use std::{str::FromStr, sync::Mutex};
//use crate::database::carrito_numero_productos;
//use crate::database::{self, tiene_productos};
use uuid::Uuid;
use crate::models::data_transfer_objects::product::Product;
use crate::models::get_product::GetProductForm;
use crate::schema::products::variations;
use crate::servicesX::*;
use serde_json::Value;

use crate::models::models_x::*;
//use crate::models::pages::{CategoryTemplate,EditProductTemplate,TemplateEjemplo,ProductsPanelPrueba,Product};
use crate::models::pages::*;

use askama::Template;

use crate::controllers::createProduct::create_product_controllers::ROUTES;
use crate::controllers::components::product_panel_group::ROUTES as ROUTES_PRODUCT_PANEL_GROUP;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use std::time::Instant; //para medir el tiempo de ejecución de una función
/* 
use askama::Template;

#[derive(Template,Clone,Debug)]
#[template(path = "category.html")]
struct CategoryTemplate {
    categories: Vec<Category2>,
}
*/



lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza
    pub static ref TEMPLATES: Tera = {
        let source = "templates/**/*";
        let tera = Tera::new(&source).unwrap();
        tera
    };

    //momentaneo
    static ref ID_BASE:Uuid =Uuid::parse_str("95022733-f013-301a-0ada-abc18f151006").unwrap();
    static ref ID_BASE_JOKER:Uuid =Uuid::parse_str("95023733-f013-301a-0ada-abc18f151006").unwrap();

    //static ref CATEGORY_CHILDREN_URL: String = format!("{}/category-children", SCOPE);
    //static ref NEXT_SELECT_URL: String = format!("{}/next-select", SCOPE);
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
}
use crate::services::products_panel;
use crate::services::components::edit_product;
use crate::controllers::components::edit_product_controller::ROUTES as ROUTES_EDIT_PRODUCT;
use crate::models::components::edit_product::EditProduct;
use crate::models::components::title::Title;

// Las páginas del panel de productos están en `routes/panel_products.rs`.

#[get("/")]
async fn index() -> impl Responder {
    let context1 = tera::Context::new();
    
    let page_content: String = TEMPLATES.render("base.html", &context1).unwrap();
    //print!("{}",page_content);
    HttpResponse::Ok().body(page_content)
}
