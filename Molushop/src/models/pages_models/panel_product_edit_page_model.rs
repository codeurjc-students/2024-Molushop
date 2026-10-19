use askama::Template;
use uuid::Uuid;
use crate::models::components::panel_nav::PanelNav;
use crate::models::components::edit_product::ProductEdit2;
use crate::models::components::edit_product::Routes as Routes_edit_product;

/// `/panel/stores/{id}/products/edit/{id}`: editar un producto. Desde la lista
/// se llega por htmx; esta es la página entera, para cuando se entra por URL.
#[derive(Template,Clone,Debug)]
#[template(path="pages/panel_product_edit/panel_product_edit.html")]
pub struct PanelProductEditPage {
    pub user_logged:bool,
    pub product: ProductEdit2,
    pub routes_edit_product: &'static Routes_edit_product,
    pub page_name: String,
    pub panel_nav: PanelNav,
    pub store_id: Uuid,
    pub store_name: String,
}
