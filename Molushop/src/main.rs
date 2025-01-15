use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_files as fs;
use servicesX::obtain_base_categories;
use uuid::Uuid;
use serde_json::Value;
use std::sync::Arc;

mod config; 
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

use diesel_async::pooled_connection::AsyncDieselConnectionManager;
//use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::deadpool::Pool;
//use bb8::Pool;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;

use dotenvy::dotenv;

use std::any::type_name;

fn print_type_of<T>(_: &T) {
    println!("{}", type_name::<T>());
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //hacer el pool de conexiones
    dotenv().ok();

    let config: AsyncDieselConnectionManager<AsyncPgConnection> = AsyncDieselConnectionManager::<AsyncPgConnection>::new(std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"));
    let pool:Pool<AsyncPgConnection> = Pool::builder(config).build().unwrap();
    print_type_of(&pool);
    let pool_data = web::Data::new(pool);

    let client = startup::configure_and_return_s3_client().await;
    let client_data = web::Data::new(Arc::new(client));

    println!("Hello, world!");

    HttpServer::new(move|| {
        App::new()
            .app_data(pool_data.clone())
            .app_data(client_data.clone())
            .configure(config::static_config)
            .configure(routes_x::config)
            //.service(routes_x::index2)
            //.service(routes_x::categories)
            //.service(routes_x::imagen_prueba)
            //.service(routes_x::new_created_product)
            .service(controllersX::prueba_insertar)
            .service(controllersX::prueba_modificar)
            // .service(controllersX::category_children)
            // .service(controllersX::reset_category)
            // // .service(controllersX::next_category)
            // // .service(controllersX::create_product)
            // // .service(controllersX::add_variation)
            // // .service(controllersX::add_variation_value)
            // // .service(controllersX::add_specs)
            // AWS S3
            .service(aws::scope_aws()) /*  /aws/s3/xxx  */
            //create_product
            .service(controllers::createProduct::controllers::scope_create_product())
            // Static files
            .service(fs::Files::new("/assets", "assets").show_files_listing())
            
            .service(routes_x::example)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}