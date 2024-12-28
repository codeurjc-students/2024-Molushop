// todos los modelos de la pagina que se harán en el context de create product se harán aquí
use rinja::Template;
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
    pub add_variation: &'static str,
    pub add_variation_value: &'static str,
    pub add_specs: &'static str,
}




#[derive(Template,Clone,Debug)]
#[template(path = "create_product/list-category-base.html")]
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
#[template(path = "create_product/base-category.html")]
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
#[template(path = "create_product/base-producto.html")]
pub struct BaseProducto{
    specs: Vec<String>,
    category_id: String,
    routes : &'static Routes,
    name: String,
}

impl BaseProducto{
    pub fn new(specsx: BaseSpecs,id: String, routes: &'static Routes,name: String) -> Self {
        BaseProducto {
            specs: specsx.specs,   
            category_id: id,
            routes,
            name
        }
    }
}


#[derive(Template,Clone,Debug)]
#[template(path = "create_product/variations-input.html")]
pub struct VariationsInput{
    routes: &'static Routes,
}
impl VariationsInput{
    pub fn new(routes: &'static Routes) -> Self {
        VariationsInput {
            routes
        }
    }
}

#[derive(Template,Clone,Debug)]
#[template(path = "create_product/variations-input-extra.html")]
pub struct VariationsInputExtra{
}
impl VariationsInputExtra{
    pub fn new() -> Self {
        VariationsInputExtra {
        }
    }
}

#[derive(Template,Clone,Debug)]
#[template(path = "create_product/specs-input.html")]
pub struct SpecsInput{
}
impl SpecsInput{
    pub fn new() -> Self {
        SpecsInput {
        }
    }
}