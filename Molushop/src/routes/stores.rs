//! Página pública de una tienda: `/stores/{id}?q=&category=`. Es la búsqueda
//! (`routes::search::render_search`) filtrada a sus productos, con su
//! cabecera y su propio buscador. Solo las tiendas ACTIVAS; las demás, 404.

use actix_web::{web, HttpResponse};
use chrono::Datelike;
use fluent_templates::fluent_bundle::FluentValue;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::constants::urls::STORE_URL_PREFIX;
use crate::i18n::I18n;
use crate::middleware::auth::{Auth, SessionData};
use crate::models::components::product_card_group_model::CardFilter;
use crate::models::pages_models::search_page_model::StoreHeader;
use crate::routes::not_found::not_found_page;
use crate::routes::search::{render_search, SearchQuery};
use crate::services::public_store_service::{get_active_store, PublicStore};

pub fn config(cfg: &mut web::ServiceConfig) {
    // En inglés, /en/stores/{id}: la registra shop_routes también bajo /en.
    cfg.route("/stores/{store_id}", web::get().to(store_page).wrap(Auth::new()));
}

/// El id llega como texto: uno mal escrito da la misma 404 de la tienda que
/// una tienda que no existe (con `Path<Uuid>`, actix respondería un 404 vacío).
async fn store_page(i18n: I18n, path: web::Path<String>, query: web::Query<SearchQuery>, pool_data: web::Data<DbPool>, opt_session_data: Option<web::ReqData<SessionData>>) -> HttpResponse {
    let pool = pool_data.get_ref();
    let user_id = opt_session_data.map(|session_data| session_data.into_inner().id);

    let store = match Uuid::parse_str(&path.into_inner()) {
        Ok(store_id) => match get_active_store(&store_id, pool).await {
            Ok(store) => store,
            Err(e) => {
                tracing::error!(store_id = %store_id, error = %e, "could not load the store");
                return HttpResponse::InternalServerError().finish();
            }
        },
        Err(_) => None,
    };
    let Some(store) = store else {
        return not_found_page(
            i18n,
            "store-notfound-page-title",
            "store-notfound-title",
            "store-notfound-text",
            user_id.as_ref(),
            pool,
        ).await;
    };

    let query = query.into_inner();
    let filter = CardFilter::new(query.q, query.category).with_store(Some(store.id));
    let header = store_header(&i18n, store);
    render_search(i18n, filter, Some(header), user_id.as_ref(), pool).await
}

fn store_header(i18n: &I18n, store: PublicStore) -> StoreHeader {
    // El mes va como número: el .ftl elige su nombre (store-since).
    let since = i18n.t_args("store-since", &[
        ("month", FluentValue::from(store.created_at.month() as i64)),
        ("year", FluentValue::from(store.created_at.year().to_string())),
    ]);
    StoreHeader {
        name: store.store_name,
        url: store_url(&store.id),
        since,
        products: store.products,
    }
}

pub fn store_url(store_id: &Uuid) -> String {
    format!("{}{}", STORE_URL_PREFIX, store_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Lang;
    use chrono::TimeZone;

    /// El mes sale con su nombre en cada idioma y el año sin separador de miles.
    #[test]
    fn store_header_names_the_month_in_each_language() {
        let store = PublicStore {
            id: Uuid::nil(),
            store_name: "Moluxo Store".into(),
            created_at: chrono::Utc.with_ymd_and_hms(2026, 11, 3, 10, 0, 0).unwrap(),
            products: 7,
        };
        let es = store_header(&I18n::default(), store.clone());
        assert_eq!(es.since, "En Molushop desde noviembre de 2026");
        assert_eq!(es.url, format!("/stores/{}", Uuid::nil()));
        let en = store_header(&I18n::new(Lang::En, "/", ""), store);
        assert_eq!(en.since, "On Molushop since November 2026");
    }
}
