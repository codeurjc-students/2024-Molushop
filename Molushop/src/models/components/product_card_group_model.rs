use rinja::Template;
use crate::models::components::product_card_model::*;

#[derive(Template,Clone,Debug)]
#[template(path="components/product_card_group/templates/product_card_group.html")]
pub struct ProductCardGroup{
    pub products: Vec<ProductCardData>
}