use actix_web::{web, App, HttpServer};
use std::sync::Arc;

mod config; 
pub mod schema;
pub mod models;

pub mod routes;
pub mod controllers;
pub mod services;
pub mod utils;
pub mod middleware;
pub mod constants;
pub mod i18n;
mod logging;
mod app_env;

use services::aws::s3::{client, startup};
use controllers::components;


use dotenvy::dotenv;
use tracing_actix_web::TracingLogger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    // dev / pre / prod (ver app_env.rs). Si APP_ENV no vale, los logs arrancan
    // con el entorno por defecto de esta compilación para poder escribir el error.
    let app_env = app_env::AppEnv::from_env();
    let current_env = *app_env.as_ref().unwrap_or(&app_env::AppEnv::default_for_build());
    // Vive hasta el final de main: al soltarlo se escriben los logs pendientes.
    let log_guard = logging::init(current_env);

    // Configuración que, si está mal, rompe algo o deja un agujero: mejor no arrancar.
    // Sin clave de sesión nadie podría entrar; con el buzón "log" en producción, el
    // enlace para cambiar la contraseña de cualquiera acabaría en la consola.
    // Primero las variables obligatorias sin valor por defecto (BD y S3).
    let mut problems: Vec<String> = utils::required_env::check_required_vars();
    problems.extend([
        app_env.as_ref().map(|_| ()).map_err(Clone::clone),
        services::paseto_token_session_service::check_local_secret(),
        utils::upload_limits::check_upload_config(),
        // Con APP_ENV inválida no se sabe qué reglas tocan: basta con su error.
        match &app_env {
            Ok(env) => services::mail_service::check_mail_config(*env),
            Err(_) => Ok(()),
        },
    ]
    .into_iter()
    .filter_map(Result::err));
    if !problems.is_empty() {
        for problem in &problems {
            tracing::error!(%problem, "invalid configuration, the server will not start");
        }
        // process::exit no ejecuta los drop: soltar el guard para que salgan los logs.
        drop(log_guard);
        std::process::exit(1);
    }
    tracing::info!(app_env = %current_env, "starting Molushop");
    // Dónde escucha: 127.0.0.1:8080 detrás de un proxy en la misma máquina (lo de
    // siempre); 0.0.0.0:8080 dentro de Docker o en una plataforma (PaaS).
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    // Con TLS si la URL lo pide (Neon: sslmode=require); ver utils/db.rs.
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = utils::db::build_pool(&database_url).expect("could not build the database pool");
    // Borra cada hora (y ahora) las sesiones y enlaces de contraseña caducados hace más de una semana.
    services::expired_rows_cleanup::spawn(pool.clone());
    let pool_data = web::Data::new(pool);

    let client = startup::configure_and_return_s3_client().await;
    let client_data = web::Data::new(Arc::new(client));

    HttpServer::new(move|| {
        App::new()
            // HEAD como GET sin cuerpo (Render hace HEAD / al arrancar). El primer wrap
            // es el de más dentro: el log de peticiones sigue viendo HEAD.
            .wrap(actix_web::middleware::from_fn(middleware::head_as_get::head_as_get))
            // `HttpResponse::Ok().body(html)` no pone Content-Type y el navegador tenía que
            // adivinarlo. DefaultHeaders solo la añade si la respuesta no trae una: JSON,
            // estáticos (actix-files) y los que ya ponen la suya no cambian.
            .wrap(
                actix_web::middleware::DefaultHeaders::new()
                    .add((actix_web::http::header::CONTENT_TYPE, "text/html; charset=utf-8")),
            )
            // Los estáticos salen de actix-files solo con ETag/Last-Modified. Sin Cache-Control
            // el navegador reutiliza a ciegas su copia vieja y un cambio de JS/CSS "no hace nada"
            // hasta un Ctrl+Shift+R. Con no-cache pregunta siempre, y el ETag lo resuelve en 304.
            .wrap_fn(|req, srv| {
                use actix_web::dev::Service;
                let is_static = middleware::request_log::is_static(req.path());
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
            // Una línea de log por petición con su id (X-Request-Id). Lo último que se
            // envuelve va por fuera: TracingLogger abre el span y request_log, dentro, escribe.
            .wrap(actix_web::middleware::from_fn(middleware::request_log::request_log))
            .wrap(TracingLogger::<middleware::request_log::RequestSpan>::new())
            .app_data(pool_data.clone())
            .app_data(client_data.clone())
            // Subidas multipart (imágenes): sin esto, actix acepta 50 MB por petición.
            .app_data(utils::upload_limits::multipart_config())
            .configure(config::static_config)
            .configure(routes::config_routes)
            .service(components::scope::scope_components())
            // Cualquier URL que no exista: la 404 de la tienda, con la sesión
            .default_service(web::to(routes::not_found::default_not_found).wrap(middleware::auth::Auth::new()))
    })
    .bind(bind_addr.as_str())?
    .run()
    .await
}