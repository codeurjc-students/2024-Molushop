use askama::Template;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::models::components::nav1_model::*;
use crate::controllers::components::nav1_controller::ROUTES;
use crate::services::components::cart_add_service;

pub async fn get_nav1_object(name: String, user_id: Option<&Uuid>, pool: &DbPool) -> Nav1Data {
    let cart_count = match user_id {
        Some(uid) => cart_add_service::get_cart_item_count(uid, pool).await,
        None => 0,
    };
    Nav1Data {
        routes: &ROUTES,
        name,
        cart_count,
    }
}

pub async fn get_nav1_render(name: String, user_id: Option<&Uuid>, pool: &DbPool) -> String {
    let objeto = get_nav1_object(name, user_id, pool).await;
    Nav1 {
        nav1: objeto
    }.render().unwrap()
}
