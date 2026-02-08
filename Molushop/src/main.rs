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

pub mod routes;
pub mod controllers;
pub mod services;
pub mod jobs;
pub mod utils;
pub mod middleware;
pub mod constants;

use services::servicesX;
use routes::routes_x;
use controllers::aws::s3::aws_s3;
use controllers::aws::aws;
use controllers::controllersX;
use services::aws::s3::{client, startup};
use controllers::components;

use diesel_async::pooled_connection::AsyncDieselConnectionManager;
//use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::deadpool::Pool;
//use bb8::Pool;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;

use dotenvy::dotenv;

use std::any::type_name;

use actix_jobs::{Job, Scheduler, run_forever};

use jobs::job_config::init_jobs;
use routes::master;

fn print_type_of<T>(_: &T) {
    println!("{}", type_name::<T>());
}

struct MyJob;
impl Job for MyJob {
    fn cron(&self) -> &str {
        "*/2 * * * * * *" // every two seconds
    }

    fn run(&mut self) {
        println!("Sending an email to all our clients...");
    }
}
struct MyJob2;
impl Job for MyJob2 {
    fn cron(&self) -> &str {
        "*/5 * * * * * *" // every two seconds
    }

    fn run(&mut self) {
        println!("Otro job");
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //init_jobs();
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
            //.service(master::scope_master::scope_master(pool_data.clone()))
            .configure(routes::config_routes)
            
            .service(controllersX::prueba_insertar)
            .service(controllersX::prueba_modificar)
            //test
            .service(controllers::test::scope())
            .service(aws::scope_aws()) /*  /aws/s3/xxx  */
            //create_product
            .service(controllers::createProduct::create_product_controllers::scope_create_product())
            .service(components::scope::scope_components())
            // Static files
            //.service(fs::Files::new("/assets", "assets").show_files_listing())
            
            .service(routes_x::example)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}