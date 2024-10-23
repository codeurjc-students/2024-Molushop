use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_files as fs;
use services::obtain_base_categories;
use uuid::Uuid;

pub mod schema;
pub mod services;
pub mod routes;
pub mod controllers;
pub mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    //let id_base = Uuid::parse_str("95022733-f013-301a-0ada-abc18f151006").unwrap();
    //database::list_tareas(); //print de la base de datos

    HttpServer::new(|| {
        App::new()
            .service(routes::index2)
            .service(routes::categories)
            .service(controllers::prueba_insertar)
            .service(controllers::prueba_modificar)
            .service(controllers::category_children)
            .service(fs::Files::new("/assets", "assets").show_files_listing())
            
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}