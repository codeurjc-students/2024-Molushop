use askama::Template;
use diesel::dsl::sum;
use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::models::components::nav1_model::*;
use crate::controllers::components::nav1_controller::ROUTES;
use crate::schema::{base_user, cart_products, carts};
use crate::services::components::cart_add_service;
use crate::utils::db::get_connection;

/// La barra de todas las páginas de la tienda y si hay usuario (`user_logged` de
/// la página). Con sesión, el nombre y las unidades del carrito salen en UNA
/// consulta y una conexión: esto se paga en cada página, y antes eran tres.
/// Si la consulta falla, la página sale igual, como sin usuario y con el
/// carrito a 0 (la barra sigue sabiendo que hay sesión).
pub async fn shop_nav(user_id: Option<&Uuid>, pool: &DbPool) -> (bool, Nav1Data) {
    let found = match user_id {
        Some(uid) => match nav_user(uid, pool).await {
            Ok(found) => found,
            Err(e) => {
                tracing::error!(error = ?e, "could not load the logged user");
                None
            }
        },
        None => None,
    };
    let user_logged = found.is_some();
    let (name, cart_count) = found.unwrap_or_default();
    let nav1 = Nav1Data {
        routes: &ROUTES,
        name,
        cart_count,
        logged: user_id.is_some(),
        search: String::new(),
    };
    (user_logged, nav1)
}

/// El nombre del usuario y las unidades de su carrito activo (0 sin carrito).
/// `None` si el usuario ya no existe.
async fn nav_user(user_id: &Uuid, pool: &DbPool) -> Result<Option<(String, i64)>, Error> {
    let connection = &mut get_connection(pool).await?;
    let cart_units = carts::table
        .inner_join(cart_products::table)
        .filter(carts::user_id.eq(user_id))
        .filter(carts::status.eq(1_i16))
        .select(sum(cart_products::quantity))
        .single_value();
    base_user::table
        .filter(base_user::id.eq(user_id))
        .select((base_user::username, cart_units))
        .first::<(String, Option<i64>)>(connection)
        .await
        .optional()
        .map(|row| row.map(|(name, units)| (name, units.unwrap_or(0))))
}

pub async fn get_nav1_object(name: String, user_id: Option<&Uuid>, pool: &DbPool) -> Nav1Data {
    let cart_count = match user_id {
        Some(uid) => cart_add_service::get_cart_item_count(uid, pool).await,
        None => 0,
    };
    Nav1Data {
        routes: &ROUTES,
        name,
        cart_count,
        logged: user_id.is_some(),
        search: String::new(),
    }
}

pub async fn get_nav1_render(name: String, user_id: Option<&Uuid>, pool: &DbPool) -> String {
    let data = get_nav1_object(name, user_id, pool).await;
    Nav1 {
        nav1: data,
        i18n: crate::i18n::I18n::default_ref(),
    }.render().unwrap()
}
