//! "Mis favoritos": `/favorites` (y `/en/favorites`). Las tarjetas de los
//! productos guardados con el corazón; sin sesión, el aviso para identificarse.

use actix_web::{web, HttpResponse};
use crate::i18n::I18n;
use askama::Template;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::constants::urls::{FAVORITES_URL, HOME_URL};
use crate::middleware::auth::{Auth, SessionData};
use crate::models::pages_models::favorites_page_model::FavoritesPage;
use crate::services::components::{login_base_service, nav1_service::get_nav1_object};
use crate::services::favorites_service::favorite_cards;
use crate::services::servicesX::get_user_2;

pub fn config(cfg: &mut web::ServiceConfig) {
    // Sin el wrap, opt_session_data llega None y la lista sale vacía
    cfg.route(FAVORITES_URL, web::get().to(get_favorites).wrap(Auth::new()));
}

async fn get_favorites(
    i18n: I18n,
    pool_data: web::Data<DbPool>,
    opt_session_data: Option<web::ReqData<SessionData>>
) -> HttpResponse {
    let pool = pool_data.get_ref();

    let user_id_opt: Option<Uuid> = opt_session_data.map(|d| d.into_inner().id);

    let mut user_name = "".to_string();
    let mut user_logged = false;

    if let Some(user_id) = user_id_opt.as_ref() {
        if let Ok(user) = get_user_2(user_id, pool).await {
            user_name = user.username;
            user_logged = true;
        }
    }

    let products = match user_id_opt.as_ref() {
        Some(user_id) => match favorite_cards(user_id, pool).await {
            Ok(products) => products,
            Err(e) => {
                println!("Error cargando los favoritos: {}", e);
                return HttpResponse::InternalServerError().finish();
            }
        },
        None => Vec::new(),
    };

    let render = FavoritesPage{
        user_logged,
        page_name: i18n.t("favorites-page-title"),
        nav1: get_nav1_object(user_name, user_id_opt.as_ref(), pool).await,
        login_base_data: login_base_service::get_login_base_model_data(),
        i18n,
        products,
        home_url: HOME_URL,
    }.render().unwrap();

    HttpResponse::Ok().body(render)
}
