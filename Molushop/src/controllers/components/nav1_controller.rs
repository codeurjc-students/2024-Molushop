use actix_web::{get, web, HttpResponse, Scope};
use lazy_static::lazy_static;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::models::components::nav1_model::*;
use super::{login_base_controller, search_suggestions_controller};
use crate::services::components::nav1_service::*;
use crate::constants::urls::{ACCOUNT_URL, FAVORITES_URL, ORDERS_URL, SEARCH_URL};

static SCOPE: &str = "/nav1";

lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza
    
    pub static ref ROUTES: Routes = Routes{
        //delete_product: Box::leak(format!("{}{}/delete-product",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        //delete_product_modal: Box::leak(format!("{}{}/delete-product-modal",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        home: "/home".to_string(),
        cart: "/cart".to_string(),
        orders: ORDERS_URL.to_string(),
        favorites: FAVORITES_URL.to_string(),
        account: ACCOUNT_URL.to_string(),
        logout: login_base_controller::ROUTES.logout.to_string(),
        search: SEARCH_URL.to_string(),
        suggestions: search_suggestions_controller::ROUTES.suggestions.clone()
        //login: Box::leak(format!("{}{}/login",SCOPE_COMPONENTS, SCOPE1).into_boxed_str())
    };

}

pub fn scope() -> Scope {
    web::scope(SCOPE)
    .configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(spawn);
}

#[get("/spawn")]
async fn spawn(pool_data: web::Data<DbPool>) -> HttpResponse {
    let render_login = get_nav1_render("Ejemplo".to_string(), None, pool_data.get_ref()).await;
    HttpResponse::Ok().body(render_login)
}
