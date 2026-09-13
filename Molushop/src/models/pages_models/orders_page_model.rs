use askama::Template;
use crate::models::components::nav1_model::*;
use crate::models::components::login_base_model::LoginProductData;
use crate::models::components::order_list_v1::OrderListData;

/// "Mis pedidos". Fuera del árbol muerto `master/es/`, igual que el carrito, el
/// checkout y la ficha del pedido.
#[derive(Template,Clone,Debug)]
#[template(path="pages/orders/orders.html")]
pub struct OrdersPage{
    pub user_logged: bool,
    pub page_name:String,
    pub nav1:Nav1Data,
    pub login_base_data:LoginProductData,
    pub order_list:OrderListData
}
