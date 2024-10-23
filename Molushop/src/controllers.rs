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
    println!("hola");
    println!("category_id: {}",&category_id);
    let categories = obtain_categories_children(&category_id);
    let mut context = tera::Context::new();
    context.insert("categories",&categories);
    let rendered = TEMPLATES.render("list-category-base.html", &context).unwrap();
    HttpResponse::Ok().body(rendered)
}
