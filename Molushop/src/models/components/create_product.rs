// todos los modelos de la pagina que se harán en el context de create product se harán aquí
use askama::Template;
use serde::Serialize;
use super::super::models_x::{Category2,Category};
use super::super::extra::{BaseSpecs};
//const BASE_PATH: &str = "create_product/";
//poner una variable como path de la plantilla?

#[derive(Serialize,Clone,Debug)]
pub struct Routes{
    pub category_children: &'static str,
    pub reset_category: &'static str,
    pub next_select: &'static str,
    pub create_product: &'static str,
}




#[derive(Template,Clone,Debug)]
#[template(path = "components/create_product/templates/list-category-base.html")]
pub struct ListCategoryBase{
    ancestors : Vec<Category2>,
    categories: Vec<Category2>,
    routes: &'static Routes,
}

impl ListCategoryBase {
    pub fn new_2(
            ancestors: Vec<Category>,
            categories: Vec<Category>,
            routes: &'static Routes,
        ) -> Self {
        ListCategoryBase {
            ancestors: ancestors.iter().map(|c| c.to_category2().unwrap()).collect(),
            categories: categories.iter().map(|c| c.to_category2().unwrap()).collect(),
            routes
        }
    }
}

#[derive(Template,Clone,Debug)]
#[template(path = "components/create_product/templates/base-category.html")]
pub struct BaseCategory{
    categories: Vec<Category2>,
    routes: &'static Routes,
}

impl BaseCategory{
    pub fn new(categories: Vec<Category>,routes: &'static Routes) -> Self {
        BaseCategory {
            categories: categories.iter().map(|c| c.to_category2().unwrap()).collect(),
            routes
        }
    }
}

#[derive(Template,Clone,Debug)]
#[template(path = "components/create_product/templates/base-producto.html")]
pub struct BaseProducto{
    specs: Vec<String>,
    category_id: String,
    routes : &'static Routes,
    /// De la raíz a la categoría elegida (incluida), para la miga de pan.
    ancestors: Vec<Category2>,
}

impl BaseProducto{
    pub fn new(specsx: BaseSpecs,id: String, routes: &'static Routes, ancestors: Vec<Category>) -> Self {
        BaseProducto {
            specs: specsx.specs,
            category_id: id,
            routes,
            ancestors: ancestors.iter().map(|c| c.to_category2().unwrap()).collect(),
        }
    }

    /// "Cambiar": vuelve a la lista de la que se eligió la categoría (sus
    /// hermanas) o, si era de primer nivel, al principio.
    pub fn change_url(&self) -> String {
        match self.ancestors.len().checked_sub(2).and_then(|i| self.ancestors.get(i)) {
            Some(parent) => format!("{}/{}", self.routes.category_children, parent.id),
            None => self.routes.reset_category.to_string(),
        }
    }
}
