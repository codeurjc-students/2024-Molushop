use actix_web::{web, App, HttpServer};
use std::sync::Arc;

mod config; 
pub mod schema;
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
pub mod i18n;

use services::aws::s3::{client, startup};
use controllers::components;

use diesel_async::pooled_connection::AsyncDieselConnectionManager;
//use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::deadpool::Pool;
//use bb8::Pool;
use diesel_async::AsyncPgConnection;

use dotenvy::dotenv;

use std::any::type_name;



fn print_type_of<T>(_: &T) {
    println!("{}", type_name::<T>());
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
            // Los estáticos salen de actix-files solo con ETag/Last-Modified. Sin Cache-Control
            // el navegador reutiliza a ciegas su copia vieja y un cambio de JS/CSS "no hace nada"
            // hasta un Ctrl+Shift+R. Con no-cache pregunta siempre, y el ETag lo resuelve en 304.
            .wrap_fn(|req, srv| {
                use actix_web::dev::Service;
                let is_static = ["/static/", "/assets/", "/css/"]
                    .iter()
                    .any(|prefix| req.path().starts_with(prefix));
                let fut = srv.call(req);
                async move {
                    let mut res = fut.await?;
                    if is_static {
                        res.headers_mut().insert(
                            actix_web::http::header::CACHE_CONTROL,
                            actix_web::http::header::HeaderValue::from_static("no-cache"),
                        );
                    }
                    Ok(res)
                }
            })
            // Contra CSRF: los POST/PUT/PATCH/DELETE tienen que venir de esta web.
            .wrap(actix_web::middleware::from_fn(middleware::origin_check::origin_check))
            .app_data(pool_data.clone())
            .app_data(client_data.clone())
            .configure(config::static_config)
            .configure(routes::config_routes)
            
            //create_product
.service(components::scope::scope_components())
            // Cualquier URL que no exista: la 404 de la tienda, con la sesión
            .default_service(web::to(routes::not_found::default_not_found).wrap(middleware::auth::Auth::new()))
            // Static files
            //.service(fs::Files::new("/assets", "assets").show_files_listing())
            
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}