use askama::Template;
use uuid::Uuid;
use super::models_x::{Category2,Category};
use super::components::create_product::Routes;
use super::components::panel_nav::PanelNav;
use super::data_transfer_objects::product::{Product,RoutesProductPanelGroup};
#[derive(Template,Clone,Debug)]
#[template(path = "pages/create_product/category.html")]
pub struct CategoryTemplate {
    pub user_logged:bool,
    categories: Vec<Category2>,
    routes: &'static Routes,
    page_name: String,
    panel_nav: PanelNav,
    /// Tienda a cuyo nombre se crea: la página la manda en la cabecera `X-Store-Id`.
    store_id: Uuid,
}

impl CategoryTemplate {
    pub fn new_2(
            categories1: Vec<Category>,
            routes: &'static Routes,
            panel_nav: PanelNav,
            store_id: Uuid,
        ) -> Self {
        CategoryTemplate {
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

#[derive(Template,Clone,Debug)]
#[template(path = "pages/products_panel/index.html")]
pub struct ProductsPanel {
    pub user_logged:bool,
    pub products: Vec<Product>, //ProductPanelGroup
    pub routes: &'static RoutesProductPanelGroup,
    pub page_name: String,
    /// La tienda de la URL.
    pub store_id: Uuid,
    pub store_name: String,
    pub panel_nav: PanelNav,
}

use crate::models::components::edit_product::{ProductEdit,ProductEdit2};
use crate::models::components::edit_product::Routes as Routes_edit_product;

#[derive(Template,Clone,Debug)]
#[template(path = "pages/products-panel_edit/products-panel_edit.html")]
pub struct ProductsPanelEdit {
    pub user_logged:bool,
    pub product: ProductEdit2,
    pub routes_edit_product: &'static Routes_edit_product,
    pub page_name: String,
    pub panel_nav: PanelNav,
    pub store_id: Uuid,
}
