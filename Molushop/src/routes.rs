// paginas de la aplicación
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


#[get("/")]
async fn index() -> impl Responder {
    /* 
    add_products(); // se añaden productos a la base de datos
    database::list_productos();
    let mut products = Vec::new();
    let mut context1 = tera::Context::new();
    match tiene_productos(){
        (true,results) => {
            //los resultados los vot a poner en un nuevo objeto
            
            for product in results{
                let p = Product{
                    id: product.id_producto.to_string(),
                    nombre: product.nombre_producto,
                    descripcion: product.descripcion.unwrap(),
                    precio: product.precio.to_string(),
                    imagen: product.imagen.unwrap(),
                };
                products.push(p);
            }
            

            context1.insert("products",&products);
        },
        (false,_) => {
            context1.insert("lista_productos",&products);
        }
    }
    //ver si hay contenido en el carrito
    match database::existe_en_carrito_user(&ID_BASE){
        Ok(true) => {
            context1.insert("num_carrito", &carrito_numero_productos(&ID_BASE));
        },
        Ok(false) => {
            context1.insert("num_carrito", &0);
        },
        Err(_) => {
            println!("Error al buscar en carrito");
        }
    }
    */
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