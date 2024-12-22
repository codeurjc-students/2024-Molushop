// paginas de la aplicación
use actix_web::{get, post, web,delete, App, HttpResponse, HttpServer, Responder, http::StatusCode};
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
use crate::servicesX::*;
use serde_json::Value;

use crate::models::models_x::*;
use crate::models::pages::CategoryTemplate;

use rinja::Template;

use crate::controllers::createProduct::controllers::ROUTES;
/* 
use rinja::Template;

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


#[get("/")]
async fn index() -> impl Responder {
    let context1 = tera::Context::new();
    
    let page_content: String = TEMPLATES.render("base.html", &context1).unwrap();
    //print!("{}",page_content);
    HttpResponse::Ok().body(page_content)
}

#[get("/")]
async fn index2() -> impl Responder {
 
    let context1 = tera::Context::new();
    
    let page_content: String = TEMPLATES.render("test.html", &context1).unwrap();
    //print!("{}",page_content);
    HttpResponse::Ok().body(page_content)
}

#[get("/category")]
async fn categories() -> impl Responder {
    //obtener vector de datos de la base de datos
    let categories = obtain_base_categories();
    match(categories){
        Ok(categories) => {
            let category_page  = CategoryTemplate::new_2(
                categories,
                &ROUTES
            );
            let render = category_page.render().unwrap();
            HttpResponse::Ok().body(render)
        },
        Err(e) => {
            println!("Error loading categories: {}", e);
            return HttpResponse::InternalServerError().body("Error loading categories");
        }
    }
}


#[get("/products/{product}")]
async fn new_created_product(path:web::Path<Uuid>) -> impl Responder {
    
    let mut context1 = tera::Context::new();
    
    let id_product = path.into_inner();
    //let id_product= Uuid::parse_str(&product).unwrap();
    //print!("{}",product);
    //obtener vector de datos de la base de datos
    let product = get_product(&id_product);
    let mut keys:Vec<&String> = Vec::new();
    match(product){
        Ok(product) => {
            context1.insert("product",&product);
            let cosa = product.variations;
            match cosa {
                Some(variations) => {
                    let var_aux = variations.as_object().unwrap();
                    //println!("Content variations: {:?}",content_variations);
                    keys = var_aux.keys().collect(); 
                    let content_variations:Vec<Vec<String>> = var_aux.iter().map(|(_,v)| {
                        let values = v.as_array().unwrap();
                        let values_str:Vec<String> = values.iter().map(|v| v.as_str().unwrap().to_string()).collect();
                        values_str
                    }).collect();
                    
                    let values_new = combine_tail_recursive(content_variations);
                    println!("Hola");
                    println!("{:?}",values_new);
                    context1.insert("variations", &values_new);
                    context1.insert("variationsTitles", &keys);
                    context1.insert("flag", &true);
                },
                None => {
                    println!("No variations");  
                    keys = Vec::new();
                    let values_new2:Vec<String> = Vec::new();
                    context1.insert("variations", &values_new2);
                    context1.insert("variationsTitles", &keys);
                    context1.insert("flag", &false);
                }
            }
            //context1.insert("variationsTitles", &keys);
            //context1.insert("product",&product);
        },
        Err(e) => {
            println!("Error loading product: {}", e);
            return HttpResponse::InternalServerError().body("Error loading product");
        }
    }

    let page_content: String = TEMPLATES.render("create_product/product.html", &context1).unwrap();
    //print!("{}",page_content);
    HttpResponse::Ok().body(page_content)
}

#[get("/imagen-prueba")]
async fn imagen_prueba() -> impl Responder {
    let context1 = tera::Context::new();
    
    let page_content: String = TEMPLATES.render("imagen-prueba.html", &context1).unwrap();
    //print!("{}",page_content);
    HttpResponse::Ok().body(page_content)
}