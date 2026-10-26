pub mod home;
pub mod about_us;
pub mod ui_catalog;
pub mod sandbox;
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
pub mod account;
pub mod password_reset;
pub mod not_found;
pub mod search;
pub mod lang;

use actix_web::web;
use crate::middleware::auth::Auth;

/// Función centralizada para configurar todas las rutas de la aplicación
pub fn config_routes(cfg: &mut web::ServiceConfig) {
    // La tienda, en español (sin prefijo) y en inglés (/en). "/en" a secas va
    // antes que el scope: dentro de él solo existe "/en/" (src/i18n.rs).
    shop_routes(cfg);
    cfg.route("/en", web::get().to(home::redirect_home));
    cfg.service(web::scope("/en").configure(shop_routes));
    lang::config(cfg);

    ui_catalog::config(cfg);
    sandbox::config(cfg);
    panel::config(cfg);
}

/// Las páginas de la tienda: las que existen en cada idioma.
fn shop_routes(cfg: &mut web::ServiceConfig) {
    home::config(cfg);
    search::config(cfg);
    about_us::config(cfg);
    product::config(cfg);
    cart::config(cfg);
    checkout::config(cfg);
    orders::config(cfg);
    account::config(cfg);
    password_reset::config(cfg);
}