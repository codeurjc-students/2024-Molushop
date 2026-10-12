pub mod home;
pub mod about_us;
pub mod ui_catalog;
pub mod product;
pub mod cart;
pub mod checkout;
pub mod orders;
pub mod panel;
pub mod panel_products;
pub mod panel_orders;
pub mod admin_stores;
pub mod admin_orders;
pub mod panel_register;

use actix_web::web;
use crate::middleware::auth::Auth;

/// Función centralizada para configurar todas las rutas de la aplicación
pub fn config_routes(cfg: &mut web::ServiceConfig) {
    home::config(cfg);
    about_us::config(cfg);
    ui_catalog::config(cfg);
    product::config(cfg);
    cart::config(cfg);
    checkout::config(cfg);
    orders::config(cfg);
    panel::config(cfg);
}