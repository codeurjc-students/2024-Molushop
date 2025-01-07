use rinja::Template;
use super::models_x::{Category2,Category};
use super::components::create_product::Routes;
use super::get_product::GetProductForm;
#[derive(Template,Clone,Debug)]
#[template(path = "pages/create_product/category.html")]
pub struct CategoryTemplate {
    categories: Vec<Category2>,
    routes: &'static Routes,
}

impl CategoryTemplate {
    pub fn new_2(
            categories1: Vec<Category>,
            routes: &'static Routes,
        ) -> Self {
        CategoryTemplate {
            categories: categories1.iter().map(|c| c.to_category2().unwrap()).collect(),
            routes
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
#[template(path = "pages/products_panel/index.html")]
pub struct ProductsPanel {
}
