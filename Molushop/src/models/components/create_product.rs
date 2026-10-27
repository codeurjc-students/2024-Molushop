// todos los modelos de la pagina que se harán en el context de create product se harán aquí
use askama::Template;
use serde::Serialize;
use super::super::models_x::{Category2,Category};
use super::super::extra::{BaseSpecs};
//const BASE_PATH: &str = "create_product/";
//poner una variable como path de la plantilla?
use serde_json::Value;
use serde_json::json;

#[derive(Serialize,Clone,Debug)]
pub struct Routes{
    pub category_children: &'static str,
    pub reset_category: &'static str,
    pub next_select: &'static str,
    pub create_product: &'static str,
}




#[derive(Template,Clone,Debug)]
#[template(path = "components/create_product/templates/list-category-base.html")]
pub struct List_category_base{
    padres : Vec<Category2>,
    categories: Vec<Category2>,
    routes: &'static Routes,
}

impl List_category_base {
    pub fn new_2(
            padresx: Vec<Category>,
            categoriesx: Vec<Category>,
            routes: &'static Routes,
        ) -> Self {
        List_category_base {
            padres: padresx.iter().map(|c| c.to_category2().unwrap()).collect(),
            categories: categoriesx.iter().map(|c| c.to_category2().unwrap()).collect(),
            routes
        }
    }
}

#[derive(Template,Clone,Debug)]
#[template(path = "components/create_product/templates/base-category.html")]
pub struct Base_category{
    categories: Vec<Category2>,
    routes: &'static Routes,
}

impl Base_category{
    pub fn new(categoriesx: Vec<Category>,routes: &'static Routes) -> Self {
        Base_category {
            categories: categoriesx.iter().map(|c| c.to_category2().unwrap()).collect(),
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
    padres: Vec<Category2>,
}

impl BaseProducto{
    pub fn new(specsx: BaseSpecs,id: String, routes: &'static Routes, padresx: Vec<Category>) -> Self {
        BaseProducto {
            specs: specsx.specs,
            category_id: id,
            routes,
            padres: padresx.iter().map(|c| c.to_category2().unwrap()).collect(),
        }
    }

    /// "Cambiar": vuelve a la lista de la que se eligió la categoría (sus
    /// hermanas) o, si era de primer nivel, al principio.
    pub fn change_url(&self) -> String {
        match self.padres.len().checked_sub(2).and_then(|i| self.padres.get(i)) {
            Some(parent) => format!("{}/{}", self.routes.category_children, parent.id),
            None => self.routes.reset_category.to_string(),
        }
    }
}
