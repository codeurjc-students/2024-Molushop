use rinja::Template;
use crate::models::data_transfer_objects::product::{Product,RoutesProductPanelGroup};

#[derive(Template,Clone,Debug)]
#[template(path = "components/cart_total_v1/cart_total_v1.html")]
pub struct CartTotalV1{
}