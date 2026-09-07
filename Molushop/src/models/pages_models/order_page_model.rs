use askama::Template;
use crate::models::components::nav1_model::*;
use crate::models::components::login_base_model::LoginProductData;
use crate::models::components::order_detail_v1::OrderData;

/// Página de un pedido confirmado. Fuera del árbol muerto `master/es/`, igual
/// que el carrito y el checkout.
#[derive(Template,Clone,Debug)]
#[template(path="pages/orders/order.html")]
pub struct OrderPage{
    pub user_logged: bool,
    pub page_name:String,
    pub nav1:Nav1Data,
    pub login_base_data:LoginProductData,
    pub order:OrderData
}
