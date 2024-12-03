use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_files as fs;
use servicesX::obtain_base_categories;
use uuid::Uuid;
use serde_json::Value;
use std::sync::Arc;

pub mod schema;
//pub mod servicesX;
//pub mod routes;
//pub mod controllersX;
pub mod models;
//pub mod client;
//pub mod upload;
//pub mod startup;

pub mod  routes;
pub mod controllers;
pub mod services;

use services::servicesX;
use routes::routes_x;
use controllers::aws::s3::aws_s3;
use controllers::aws::aws;
use controllers::controllersX;
use services::aws::s3::{client, startup};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    

    //let id_base = Uuid::parse_str("95022733-f013-301a-0ada-abc18f151006").unwrap();
    //database::list_tareas(); //print de la base de datos
    //let ancestor_str = String::from("ACCESS");
    //services::obtainAncestors(&ancestor_str);
    /*let lists = vec![
        vec![1, 2, 3],
        vec![4, 5],
        vec![6, 7]
    ];
    let result = services::combine_tail_recursive(lists);
    println!("{:?}",result);*/

    /*Probar lo de AWS */
    //let s3_client = actix_web::web::Data::new(configure_and_return_s3_client().await);
    //obtener el cliente
    

    let client = startup::configure_and_return_s3_client().await;
    let client_data = web::Data::new(Arc::new(client));

    println!("Hello, world!");

    HttpServer::new(move|| {
        App::new()
            .app_data(client_data.clone())
            .service(routes_x::index2)
            .service(routes_x::categories)
            .service(routes_x::imagen_prueba)
            .service(routes_x::new_created_product)
            .service(controllersX::prueba_insertar)
            .service(controllersX::prueba_modificar)
            .service(controllersX::reset_category)
            .service(controllersX::category_children)
            .service(controllersX::next_category)
            .service(controllersX::create_product)
            .service(controllersX::add_variation)
            .service(controllersX::add_variation_value)
            .service(controllersX::add_specs)
            // AWS S3
            .service(aws::scope_aws()) /*  /aws/s3/xxx  */
            // Static files
            .service(fs::Files::new("/assets", "assets").show_files_listing())
            
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}