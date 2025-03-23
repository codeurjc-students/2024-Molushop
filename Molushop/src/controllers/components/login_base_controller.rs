use actix_web::{get, post, web,delete,Scope, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use bigdecimal::BigDecimal;
use diesel::result;
use lazy_static::lazy_static;
use rinja::Template;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{controllers::createProduct::product, models::components::warning_modal::WarningModal};
use crate::models::components::edit_product_variation_model::Routes;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use super::scope::SCOPE_COMPONENTS;
use crate::services::servicesX::{set_variation_identifiers,update_variation_status,update_price_variation,set_stock_variation};
use crate::models::components::login_base_model::*;

static SCOPE1: &str = "/login-base-login";
static SCOPE2: &str = "/login-base-register";

pub fn scope() -> Scope {
    web::scope(SCOPE1)
        .configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    //cfg.service(products_panel_prueba);
    cfg
        .service(spawn);

}

#[get("/spawn")]
async fn spawn()->HttpResponse	{
    
    let render_login = LoginProduct{}.render().unwrap();
    HttpResponse::Ok().body(render_login)

}