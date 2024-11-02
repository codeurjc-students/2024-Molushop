// logica de las llamadas al server
use actix_web::{get, post, web,delete, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use bigdecimal::BigDecimal;
use lazy_static::lazy_static;
use tera::Tera;
use actix_files as fs;
use serde::Deserialize;
use serde::Serialize;
use std::{str::FromStr, sync::Mutex};
//use crate::database::carrito_numero_productos;
//use crate::database::{self, tiene_productos};
use uuid::Uuid;
use crate::services::*;


lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza
    pub static ref TEMPLATES: Tera = {
        let source = "templates/**/*";
        let tera = Tera::new(&source).unwrap();
        tera
    };

    //momentaneo
    static ref ID_BASE:Uuid =Uuid::parse_str("95022733-f013-301a-0ada-abc18f151006").unwrap();
    static ref ID_BASE_JOKER:Uuid =Uuid::parse_str("95023733-f013-301a-0ada-abc18f151006").unwrap();

    //static ref TASK_COUNTER: Mutex<i32> = Mutex::new(0);
    //static ref NEXT_ID: AtomicI32 = AtomicI32::new(1);
}

#[get("/prueba-insertar")]
async fn prueba_insertar() -> impl Responder {
    
    if(insert_data_test()){
        HttpResponse::Ok().body("Datos insertados")
    }else{
        HttpResponse::Ok().body("Error al insertar datos")
    }

    //HttpResponse::Ok().body(database::total_carrito(&ID_BASE).to_string()+" €")
}

#[get("/prueba-modificar")]
async fn prueba_modificar() -> impl Responder {
    
    if(modify_data_test()){
        HttpResponse::Ok().body("Datos modificados")
    }else{
        HttpResponse::Ok().body("Error al modificar datos")
    }

    //HttpResponse::Ok().body(database::total_carrito(&ID_BASE).to_string()+" €")
}

#[get("/category-children/{category_id}")]
async fn category_children(path: web::Path<String>) -> impl Responder {
    let category_id= path.into_inner();
    //println!("category_id: {}",&category_id);
    //println!("hola");
    //println!("category_id: {}",&category_id);
    
    let mut context = tera::Context::new();

    let categories_result = obtain_categories_children(&category_id);
    let ancestors_result: Result<Vec<crate::models::Category>, diesel::result::Error> = obtain_ancestors(&category_id);
    match (categories_result, ancestors_result) {
        (Ok(categories), Ok(ancestors)) => {
            context.insert("categories", &categories);
            context.insert("padres", &ancestors);
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
    //revisar los mensajes de error
    let rendered = TEMPLATES.render("create_product/list-category-base2.html", &context).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/reset-category")]
async fn reset_category() -> impl Responder {
    let categories = obtain_base_categories();
    match categories{
        Ok(categories) => {
            let mut context = tera::Context::new();
            context.insert("categories",&categories);
            let rendered = TEMPLATES.render("create_product/base-category.html", &context).unwrap();
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
    let specs_result= obtain_base_specs(&category_id);
    match specs_result {
        Ok(specs) => {
            let mut context = tera::Context::new();
            context.insert("category_id", &category_id);
            //obtenemos el primer elemento de la lista
            
            let first = specs.first().unwrap();
            match(first){
                Some(spec) => {
                    //obtener los valores de "specs""
                    //let value = &spec.get("specs").unwrap();
                    //println!("{}",value);
                    //println!("que pasa");
                    //context.insert("spec", &spec);

                    if let Some(specs_array) = spec.get("specs").and_then(|s| s.as_array()) {
                        /*let spec_vec: Vec<String> = specs_array.iter()
                            .filter_map(|s| s.as_str().map(|s| s.to_string()))
                            .collect();
                        */
                        context.insert("specs", &specs_array);
                    }
        
                    let rendered = TEMPLATES.render("create_product/base-producto.html", &context).unwrap();
                    return HttpResponse::Ok().body(rendered)
                },
                None => {
                    println!("Error loading specs");
                    return HttpResponse::InternalServerError().body("Error loading specs");
                }
            }

            //context.insert("specs", &specs);
            let rendered = TEMPLATES.render("create_product/base-producto.html", &context).unwrap();
            return HttpResponse::Ok().body(rendered)
        },
        Err(e) => {
            println!("Error loading specs: {}", e);
            return HttpResponse::InternalServerError().body("Error loading specs");
        }
    }
    
    let mut context = tera::Context::new();
    //context.insert("category_id", &category_id);
    let rendered = TEMPLATES.render("create_product/base-producto.html", &context).unwrap();
    HttpResponse::Ok().body(rendered)
}

