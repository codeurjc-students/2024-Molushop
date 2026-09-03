use actix_web::{web, HttpResponse, HttpRequest};
use serde::Deserialize;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;
use askama::Template;

use crate::models::pages_models::checkout_page_model::*;
use crate::services::components::nav1_service::*;
use crate::services::components::checkout_get_service::get_checkout_object;
use crate::middleware::auth::{Auth, SessionData};
use crate::services::components::login_base_service;
use crate::services::servicesX::get_user_2;

/// `?lines=id1,id2`: qué líneas de la cesta se están comprando. Sin él se
/// compra el carrito entero.
#[derive(Deserialize)]
pub struct CheckoutQuery {
    pub lines: Option<String>
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .route(
            "/checkout",
            // Sin el wrap, opt_session_data llega None y la página sale vacía
            web::get().to(get_checkout).wrap(Auth::new())
        )
        ;
}

async fn get_checkout(
    query: web::Query<CheckoutQuery>,
    pool_data: web::Data<DbPool>,
    req: HttpRequest,
    opt_session_data: Option<web::ReqData<SessionData>>
) -> HttpResponse {
    let pool = pool_data.get_ref();

    let mut nombre_aux = "".to_string();
    let mut user_logged = false;
    let mut user_id_opt: Option<Uuid> = None;

    if let Some(req_session_data) = opt_session_data {
        let session_data = req_session_data.into_inner();
        let user_id = session_data.id;
        user_id_opt = Some(user_id);

        match get_user_2(&user_id, &pool).await {
            Ok(user) => {
                nombre_aux = user.username;
                user_logged = true;
            }
            Err(e) => {
                println!("Ha ocurrido un error con la base de datos!");
            }
        }
    } else {
        println!("Usuario sin loggear")
    }

    let lines = query.into_inner().lines;

    let checkout_render = CheckoutPage{
        user_logged,
        page_name:"Finalizar compra".to_string(),
        nav1:get_nav1_object(nombre_aux, user_id_opt.as_ref(), pool).await,
        login_base_data:login_base_service::get_login_base_model_data(),
        checkout:get_checkout_object(user_id_opt.as_ref(), lines.as_deref(), pool).await
    }.render().unwrap();

    HttpResponse::Ok().body(checkout_render)
}
