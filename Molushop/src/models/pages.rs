use askama::Template;
use super::models_x::{Category2,Category};
use super::components::create_product::Routes;
use super::get_product::GetProductForm;
use super::data_transfer_objects::product::{Product,RoutesProductPanelGroup};
#[derive(Template,Clone,Debug)]
#[template(path = "pages/create_product/category.html")]
pub struct CategoryTemplate {
    pub user_logged:bool,
    categories: Vec<Category2>,
    routes: &'static Routes,
    page_name: String
}

impl CategoryTemplate {
    pub fn new_2(
            categories1: Vec<Category>,
            routes: &'static Routes,
        ) -> Self {
        CategoryTemplate {
            user_logged: false,
            categories: categories1.iter().map(|c| c.to_category2().unwrap()).collect(),
            routes, page_name: "Categorias".to_string()
        }
    }

    pub fn set_categories_from_category(&mut self, categories: Vec<Category>) {
        self.categories = categories.iter().map(|c| c.to_category2().unwrap()).collect();        
    }
}

#[derive(Template,Clone,Debug)]
#[template(path = "create_product/product.html")]
pub struct EditProductTemplate {
    pub product: GetProductForm
    /* 
    pub variations_titles: Vec<String>,
    pub variations: Vec<Vec<String>>,*/
}


#[derive(Template,Clone,Debug)]
#[template(path = "components/example/ejemplo.html")]
pub struct TemplateEjemplo {
}

#[derive(Template,Clone,Debug)]
#[template(path = "pages/products_panel_prueba/index.html")]
pub struct ProductsPanelPrueba {

}

#[derive(Template,Clone,Debug)]
#[template(path = "pages/products_panel/index.html")]
pub struct ProductsPanel {
    pub user_logged:bool,
    pub products: Vec<Product>, //ProductPanelGroup
    pub routes: &'static RoutesProductPanelGroup,
    pub page_name: String,

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
}


use crate::models::components::login_base_model;
#[derive(Template,Clone,Debug)]
#[template(path = "pages/login_1/login_1.html")]
pub struct Login1 {
    pub user_logged:bool,
    pub page_name: String,
    pub login_base_data: login_base_model::LoginProductData
}
