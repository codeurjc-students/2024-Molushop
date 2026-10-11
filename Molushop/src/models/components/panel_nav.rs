use uuid::Uuid;

use crate::controllers::components::login_base_controller::ROUTES as LOGIN_ROUTES;
use crate::services::panel_role_service::{PanelStore, PanelUser};

/// Tienda en la que se está trabajando (la de la URL).
#[derive(Clone, Debug)]
pub struct NavStore {
    pub id: Uuid,
    pub name: String,
}

/// Barra de navegación del panel (`layouts/panel.html`). Todas las páginas que
/// extienden ese layout llevan un campo `panel_nav`.
#[derive(Clone, Debug)]
pub struct PanelNav {
    /// Usuario con la sesión abierta.
    pub username: String,
    /// Enseña los enlaces a `/admin/stores` y `/admin/orders`.
    pub is_admin: bool,
    /// `Some` en las páginas de `/panel/stores/{store_id}/...`: enseña el nombre
    /// de la tienda y sus enlaces (Productos, Añadir producto, Pedidos).
    pub store: Option<NavStore>,
    /// Sección marcada: "home", "products", "create", "orders", "admin" o "admin_orders".
    pub active: &'static str,
    /// POST de logout de la tienda (misma sesión y cookie).
    pub logout_url: &'static str,
}

impl PanelNav {
    pub fn new(panel_user: &PanelUser, active: &'static str) -> Self {
        PanelNav {
            username: panel_user.username.clone(),
            is_admin: panel_user.is_admin,
            store: None,
            active,
            logout_url: LOGIN_ROUTES.logout,
        }
    }

    pub fn with_store(mut self, store: &PanelStore) -> Self {
        self.store = Some(NavStore { id: store.id, name: store.store_name.clone() });
        self
    }
}
