use actix_web::{web, HttpResponse};
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;
use askama::Template;

use crate::models::pages_models::about_us_page_model::*;
use crate::services::components::nav1_service::*;
use crate::middleware::auth::{Auth, SessionData};
use crate::services::components::login_base_service;
use crate::services::servicesX::get_user_2;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .route(
            "/about-us",
            // Sin el wrap, opt_session_data llega None y el nav sale sin sesión
            web::get().to(get_about_us).wrap(Auth::new())
        )
        ;
}

async fn get_about_us(
    pool_data: web::Data<DbPool>,
    opt_session_data: Option<web::ReqData<SessionData>>
) -> HttpResponse {
    let pool = pool_data.get_ref();

    let user_id_opt: Option<Uuid> = opt_session_data.map(|d| d.into_inner().id);

    let mut nombre_aux = "".to_string();
    let mut user_logged = false;

    if let Some(user_id) = user_id_opt.as_ref() {
        if let Ok(user) = get_user_2(user_id, pool).await {
            nombre_aux = user.username;
            user_logged = true;
        }
    }

    let about_us_render = AboutUsPage{
        user_logged,
        page_name:"Sobre nosotros".to_string(),
        nav1:get_nav1_object(nombre_aux, user_id_opt.as_ref(), pool).await,
        login_base_data:login_base_service::get_login_base_model_data()
    }.render().unwrap();

    HttpResponse::Ok().body(about_us_render)
}
