//! Búsqueda de la tienda: `/search?q=&category=`. El buscador de nav1 envía
//! aquí y las categorías son enlaces normales (con hx-boost la URL cambia sin
//! recargar), así que la búsqueda se puede compartir y volver atrás funciona.

use actix_web::{web, HttpResponse};
use crate::i18n::I18n;
use askama::Template;
use serde::Deserialize;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::constants::urls::SEARCH_URL;
use crate::middleware::auth::{Auth, SessionData};
use crate::models::components::product_card_group_model::{CardFilter, ProductCardGroup};
use crate::models::pages_models::search_page_model::{CategoryLink, SearchPage};
use crate::services::components::{login_base_service, nav1_service::get_nav1_object};
use crate::services::components::product_card_group_service::get_product_card_group_page;
use crate::services::search_service::{ancestors, category_counts, children};
use crate::services::servicesX::get_user_2;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route(SEARCH_URL, web::get().to(search).wrap(Auth::new()));
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub category: Option<String>,
}

async fn search(i18n: I18n, query: web::Query<SearchQuery>, pool_data: web::Data<DbPool>, opt_session_data: Option<web::ReqData<SessionData>>) -> HttpResponse {
    let pool = pool_data.get_ref();
    let query = query.into_inner();
    let mut filter = CardFilter::new(query.q, query.category);

    let categories = match category_counts(&filter.like_patterns(), i18n.lang, pool).await {
        Ok(categories) => categories,
        Err(e) => {
            println!("Búsqueda: error cargando las categorías: {}", e);
            Vec::new()
        }
    };

    // Una categoría que no existe (enlace viejo o escrito a mano) se ignora:
    // mejor todos los productos que una búsqueda vacía sin explicación.
    let path = filter.category.as_deref().map(|id| ancestors(&categories, id)).unwrap_or_default();
    if path.is_empty() {
        filter.category = None;
    }
    let selected = path.last().copied();

    let link = |id: Option<&str>, name: &str, products: i64| CategoryLink {
        name: name.to_string(),
        url: filter.with_category(id).url(SEARCH_URL, None),
        products,
    };
    let mut breadcrumb = Vec::new();
    if selected.is_some() {
        breadcrumb.push(link(None, &i18n.t("search-all-categories"), 0));
        breadcrumb.extend(path.iter().map(|c| link(Some(&c.id), &c.name, c.products)));
    }
    let subcategories = children(&categories, selected.map(|c| c.id.as_str()))
        .into_iter()
        .map(|c| link(Some(&c.id), &c.name, c.products))
        .collect();

    let user_id = opt_session_data.map(|session_data| session_data.into_inner().id);
    let (products, more_url) = get_product_card_group_page(1, &filter, user_id.as_ref(), pool).await;
    let has_results = !products.is_empty();
    let results = if has_results {
        ProductCardGroup { products, more_url, i18n: &i18n }.render().unwrap_or_else(|e| {
            println!("Error en el renderizado de ProductCardGroup! --> {}", e);
            String::new()
        })
    } else {
        String::new()
    };

    let mut user_name = String::new();
    let mut user_logged = false;
    if let Some(user_id) = user_id.as_ref() {
        if let Ok(user) = get_user_2(user_id, pool).await {
            user_name = user.username;
            user_logged = true;
        }
    }
    let mut nav1 = get_nav1_object(user_name, user_id.as_ref(), pool).await;
    nav1.search = filter.q.clone().unwrap_or_default();

    let render = SearchPage {
        user_logged,
        nav1,
        login_base_data: login_base_service::get_login_base_model_data(),
        i18n: i18n,
        query: filter.q.clone(),
        category: selected.map(|c| c.name.clone()),
        breadcrumb,
        categories: subcategories,
        clear_query_url: filter.q.as_ref().map(|_| filter.without_query().url(SEARCH_URL, None)),
        results,
        has_results,
    }.render().unwrap();

    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(render)
}
