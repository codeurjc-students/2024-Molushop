pub mod routes_x;
pub mod master;
pub mod home;
pub mod product;
pub mod cart;
pub mod checkout;

use actix_web::web;
use crate::middleware::auth::Auth;

/// Función centralizada para configurar todas las rutas de la aplicación
pub fn config_routes(cfg: &mut web::ServiceConfig) {
    home::config(cfg);
    product::config(cfg);
    cart::config(cfg);
    checkout::config(cfg);
}