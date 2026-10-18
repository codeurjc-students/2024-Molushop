use askama::Template;
use uuid::Uuid;
use crate::models::models_x::{Category2,Category};
use crate::models::components::create_product::Routes;
use crate::models::components::panel_nav::PanelNav;

/// `/panel/stores/{id}/products/create`: crear un producto, empezando por
/// elegir su categoría.
#[derive(Template,Clone,Debug)]
#[template(path="pages/panel_product_create/panel_product_create.html")]
pub struct PanelProductCreatePage {
    pub user_logged:bool,
    categories: Vec<Category2>,
    routes: &'static Routes,
    page_name: String,
    panel_nav: PanelNav,
    /// Tienda a cuyo nombre se crea: la página la manda en la cabecera `X-Store-Id`.
    store_id: Uuid,
}

impl PanelProductCreatePage {
    pub fn new_2(
            categories1: Vec<Category>,
            routes: &'static Routes,
            panel_nav: PanelNav,
            store_id: Uuid,
        ) -> Self {
        PanelProductCreatePage {
            user_logged: false,
            categories: categories1.iter().map(|c| c.to_category2().unwrap()).collect(),
            routes, page_name: "Categorias".to_string(),
            panel_nav,
            store_id,
        }
    }

    pub fn set_categories_from_category(&mut self, categories: Vec<Category>) {
        self.categories = categories.iter().map(|c| c.to_category2().unwrap()).collect();
    }
}
