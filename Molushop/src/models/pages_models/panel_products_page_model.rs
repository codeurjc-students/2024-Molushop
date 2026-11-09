use askama::Template;
use uuid::Uuid;
use crate::models::components::panel_nav::PanelNav;
use crate::models::data_transfer_objects::product::{Product,RoutesProductPanelGroup};

/// `/panel/stores/{id}/products`: los productos de una tienda.
#[derive(Template,Clone,Debug)]
#[template(path="pages/panel_products/panel_products.html")]
pub struct PanelProductsPage {
    pub user_logged:bool,
    pub products: Vec<Product>, //ProductPanelGroup
    pub routes: &'static RoutesProductPanelGroup,
    pub page_name: String,
    /// La tienda de la URL.
    pub store_id: Uuid,
    pub store_name: String,
    pub panel_nav: PanelNav,
}
