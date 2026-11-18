use actix_web::{web, HttpResponse,HttpRequest};
use crate::i18n::I18n;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;
use askama::Template;

use crate::models::pages_models::cart_page_model::*;
use crate::services::components::{
    nav1_service::*,
    cart_get_service::*
};
use crate::middleware::auth::{Auth, SessionData};
use crate::services::components::login_base_service;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .route(
            "/cart",
            web::get().to(get_cart).wrap(Auth::new())
        )
        ;

}

async fn get_cart(i18n: I18n, pool_data:web::Data<DbPool>,_req: HttpRequest,opt_session_data:Option<web::ReqData<SessionData>>)-> HttpResponse{
    let pool = pool_data.get_ref();
    
    let user_id_opt: Option<Uuid> = opt_session_data.map(|d| d.into_inner().id);
    let (user_logged, nav1) = shop_nav(user_id_opt.as_ref(), pool).await;

    let cart_render = CartPage{
        user_logged,
        page_name:i18n.t("cart-page-title"),
        nav1,
        login_base_data: login_base_service::get_login_base_model_data(),
        i18n: i18n,
        cart_total:get_cart_total_object(user_id_opt.as_ref(), pool).await
    }.render().unwrap();

    HttpResponse::Ok().body(cart_render)

}
