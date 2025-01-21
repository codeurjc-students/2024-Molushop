use rinja::Template;
use serde::Serialize;
use crate::models::data_transfer_objects::product::{Product,RoutesProductPanelGroup};

#[derive(Template,Clone,Debug)]
#[template(path = "components/product_panel_group/templates/base.html")]
pub struct ProductPanelGroup{
    products: Vec<Product>,
    routes: &'static RoutesProductPanelGroup,
}

//hace falta la logica para implementar el template