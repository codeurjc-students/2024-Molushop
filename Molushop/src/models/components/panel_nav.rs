use crate::controllers::components::login_base_controller::ROUTES as LOGIN_ROUTES;
use crate::services::panel_role_service::PanelUser;

/// Barra de navegación del panel (`layouts/panel.html`). Todas las páginas que
/// extienden ese layout llevan un campo `panel_nav`.
#[derive(Clone, Debug)]
pub struct PanelNav {
    /// Usuario con la sesión abierta.
    pub username: String,
    /// Enseña el enlace a `/admin`.
    pub is_admin: bool,
    /// Sin tienda (admin sin fila en `seller`) no se puede crear: se oculta el enlace.
    pub has_store: bool,
    /// Sección marcada: "home", "products", "create" o "admin".
    pub active: &'static str,
    /// POST de logout de la tienda (misma sesión y cookie).
    pub logout_url: &'static str,
}

impl PanelNav {
    pub fn new(panel_user: &PanelUser, active: &'static str) -> Self {
        PanelNav {
            username: panel_user.username.clone(),
            is_admin: panel_user.is_admin,
            has_store: panel_user.seller_id.is_some(),
            active,
            logout_url: LOGIN_ROUTES.logout,
        }
    }
}
