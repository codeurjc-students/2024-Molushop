//! Sugerencias del buscador de nav1 mientras se escribe: el input pide aquí
//! (htmx, con espera) y pinta la respuesta debajo. Público, como la búsqueda.
//! Con menos de `MIN_CHARS` letras responde vacío y la caja se cierra.

use actix_web::{get, web, HttpResponse, Scope};
use askama::Template;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use lazy_static::lazy_static;
use serde::Deserialize;

use super::scope::SCOPE_COMPONENTS;
use crate::i18n::I18n;
use crate::constants::urls::SEARCH_URL;
use crate::models::components::nav1_model::SearchSuggestions;
use crate::models::components::product_card_group_model::CardFilter;
use crate::models::components::product_card_model::ProductCardData;
use crate::models::pages_models::search_page_model::CategoryLink;
use crate::services::search_service::{category_counts, named_like};
use crate::services::servicesX::get_product_cards1;

type DbPool = Pool<AsyncPgConnection>;

pub static SCOPE: &str = "/search-suggestions";

/// Letras mínimas para sugerir: con una sola sale casi todo el catálogo.
const MIN_CHARS: usize = 2;
const MAX_PRODUCTS: i64 = 5;
const MAX_CATEGORIES: usize = 3;

pub struct Routes {
    pub suggestions: String,
}

lazy_static! {
    pub static ref ROUTES: Routes = Routes {
        suggestions: format!("{}{}", SCOPE_COMPONENTS, SCOPE),
    };
}

pub fn scope() -> Scope {
    web::scope(SCOPE).service(suggestions)
}

#[derive(Deserialize)]
pub struct SuggestionsQuery {
    pub q: Option<String>,
}

#[get("")]
async fn suggestions(i18n: I18n, query: web::Query<SuggestionsQuery>, pool_data: web::Data<DbPool>) -> HttpResponse {
    let pool = pool_data.get_ref();
    let filter = CardFilter::new(query.into_inner().q, None);
    let q = match filter.q.as_deref() {
        Some(q) if q.chars().count() >= MIN_CHARS => q,
        _ => return HttpResponse::Ok().content_type("text/html; charset=utf-8").finish(),
    };

    // Si algo falla, se sugiere lo que haya: "Buscar" sigue funcionando igual.
    let products = match get_product_cards1(MAX_PRODUCTS, 0, &filter.like_patterns(), None, pool).await {
        Ok(cards) => cards.into_iter().map(ProductCardData::from).collect(),
        Err(e) => {
            println!("Sugerencias: error cargando los productos: {}", e);
            Vec::new()
        }
    };
    // Las categorías con productos (de cualquier nombre), filtradas aquí por el suyo.
    let categories = match category_counts(&[], i18n.lang, pool).await {
        Ok(all) => named_like(&all, q)
            .into_iter()
            .take(MAX_CATEGORIES)
            .map(|c| CategoryLink {
                name: c.name.clone(),
                url: CardFilter::default().with_category(Some(&c.id)).url(SEARCH_URL, None),
                products: c.products,
            })
            .collect(),
        Err(e) => {
            println!("Sugerencias: error cargando las categorías: {}", e);
            Vec::new()
        }
    };

    let render = SearchSuggestions {
        query: q.to_string(),
        categories,
        products,
        all_url: filter.url(SEARCH_URL, None),
        i18n: &i18n,
    }.render().unwrap_or_else(|e| {
        println!("Error en el renderizado de SearchSuggestions! --> {}", e);
        String::new()
    });
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(render)
}
