use actix_web::{web, HttpResponse,HttpRequest};
use crate::i18n::I18n;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;
use askama::Template;

use crate::models::pages_models::product_page_model::*;
use crate::routes::not_found::not_found_page;
use crate::services::components::{
    nav1_service::*,
    product_detail_principal_service::get_product_object,
    product_reviews_service::get_product_reviews_object
};
use crate::middleware::auth::SessionData;
use crate::services::components::login_base_service;
use crate::services::favorites_service::favorite_ids_among;
use crate::middleware::auth::Auth;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        // En inglés es /en/producto/{id}: routes::config_routes la registra
        // también bajo /en (src/i18n.rs).
        .route(
            "/producto/{product_id}",
            web::get().to(get_product).wrap(Auth::new())
        );

}

/// El id llega como texto: uno mal escrito da la misma 404 de la ficha que un
/// producto que no existe (con `Path<Uuid>`, actix respondía un 404 en texto
/// plano con el error de parseo: "UUID parsing failed...").
async fn get_product(i18n: I18n, path: web::Path<String>, pool_data:web::Data<DbPool>,_req: HttpRequest,opt_session_data:Option<web::ReqData<SessionData>>)-> HttpResponse{
    let pool = pool_data.get_ref();
    let user_id_opt: Option<Uuid> = opt_session_data.map(|d| d.into_inner().id);

    let product = match Uuid::parse_str(&path.into_inner()) {
        Ok(product_id) => match get_product_object(&product_id,&pool).await {
            Ok(product_data) => product_data.map(|product_data| (product_id, product_data)),
            Err(_) => return HttpResponse::InternalServerError().finish(),
        },
        Err(_) => None,
    };
    // No está a la venta (no existe, desactivado o tienda inactiva) o el id no es válido
    let Some((product_id, mut product_data)) = product else {
        return not_found_page(
            i18n,
            "product-notfound-page-title",
            "product-notfound-title",
            "product-notfound-text",
            user_id_opt.as_ref(),
            pool
        ).await;
    };

    // El corazón lleno si ya lo tiene guardado. Si la consulta falla, la ficha
    // sale igual, con el corazón vacío.
    if let Some(user_id) = user_id_opt.as_ref() {
        match favorite_ids_among(user_id, &[product_id], pool).await {
            Ok(ids) => product_data.is_favorite = ids.contains(&product_id),
            Err(e) => tracing::error!(error = %e, "could not load the favorites of the product page"),
        }
    }

    let (user_logged, nav1) = shop_nav(user_id_opt.as_ref(), pool).await;

    let home_render = ProductPage{
        user_logged,    
        page_name:format!("{} | Molushop", product_data.name),
        nav1,
        login_base_data: login_base_service::get_login_base_model_data(),
        i18n: i18n,
        product_data,
        // Primera página de opiniones; el "ver más" pide las siguientes por HTMX.
        reviews: get_product_reviews_object(&product_id, 1, user_id_opt.as_ref(), &pool).await
    }.render().unwrap();

    HttpResponse::Ok().body(home_render)

}   