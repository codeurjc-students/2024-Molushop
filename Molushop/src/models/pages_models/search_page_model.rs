use askama::Template;
use crate::models::components::nav1_model::*;
use crate::models::components::login_base_model::LoginProductData;

/// Un enlace a la búsqueda con otra categoría (una miga o un chip), que
/// conserva el texto buscado.
#[derive(Clone,Debug)]
pub struct CategoryLink{
    pub name: String,
    pub url: String,
    /// Productos que saldrían al pulsarlo.
    pub products: i64,
}

/// Búsqueda de la tienda (`/search?q=&category=`): el título, las migas de la
/// categoría elegida, las subcategorías para afinar y la rejilla de tarjetas.
#[derive(Template,Clone,Debug)]
#[template(path="pages/search/search.html")]
pub struct SearchPage{
    pub user_logged: bool,
    pub page_name: String,
    pub nav1: Nav1Data,
    pub login_base_data: LoginProductData,
    pub title: String,
    /// De "Todas" a la categoría elegida; vacío sin categoría. La última es
    /// la elegida y no lleva enlace.
    pub breadcrumb: Vec<CategoryLink>,
    /// Las subcategorías de la elegida (o las de arriba del todo) que tienen
    /// algo para lo buscado.
    pub categories: Vec<CategoryLink>,
    /// La misma búsqueda sin el texto (`Some` solo si hay texto).
    pub clear_query_url: Option<String>,
    /// La rejilla ya pintada (`ProductCardGroup`); vacía si no hay nada.
    pub results: String,
    pub has_results: bool,
}
