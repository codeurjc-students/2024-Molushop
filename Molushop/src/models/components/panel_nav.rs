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
    /// Tiendas a las que puede saltar desde la barra (las gestionables, más la
    /// actual). Con más de una, el nombre de la tienda es un desplegable.
    pub stores: Vec<NavStore>,
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
            stores: panel_user
                .stores
                .iter()
                .filter(|s| s.is_manageable())
                .map(|s| NavStore { id: s.id, name: s.store_name.clone() })
                .collect(),
            active,
            logout_url: LOGIN_ROUTES.logout,
        }
    }

    pub fn with_store(mut self, store: &PanelStore) -> Self {
        // El admin puede estar en una tienda que no es suya: que salga igual.
        if !self.stores.iter().any(|s| s.id == store.id) {
            self.stores.insert(0, NavStore { id: store.id, name: store.store_name.clone() });
        }
        self.store = Some(NavStore { id: store.id, name: store.store_name.clone() });
        self
    }

    /// ¿Hay a dónde cambiar? Solo en las páginas de una tienda.
    pub fn can_switch_store(&self) -> bool {
        self.store.is_some() && self.stores.len() > 1
    }

    pub fn is_current_store(&self, store: &NavStore) -> bool {
        self.store.as_ref().is_some_and(|s| s.id == store.id)
    }

    /// La misma sección en otra tienda. Desde una ficha (editar producto, un
    /// pedido) se va al listado: el producto o el pedido son de la tienda de antes.
    pub fn store_url(&self, store: &NavStore) -> String {
        let section = match self.active {
            "orders" => "orders",
            "create" => "products/create",
            _ => "products",
        };
        format!("/panel/stores/{}/{}", store.id, section)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nav(active: &'static str, stores: usize) -> PanelNav {
        let stores: Vec<NavStore> = (0..stores)
            .map(|i| NavStore { id: Uuid::new_v4(), name: format!("Tienda {}", i) })
            .collect();
        PanelNav {
            username: "u".to_string(),
            is_admin: false,
            store: stores.first().cloned(),
            stores,
            active,
            logout_url: "/logout",
        }
    }

    #[test]
    fn switch_keeps_the_section() {
        let n = nav("orders", 2);
        let other = &n.stores[1];
        assert_eq!(n.store_url(other), format!("/panel/stores/{}/orders", other.id));
        assert_eq!(nav("create", 2).store_url(other), format!("/panel/stores/{}/products/create", other.id));
        assert_eq!(nav("products", 2).store_url(other), format!("/panel/stores/{}/products", other.id));
    }

    #[test]
    fn no_switch_with_a_single_store() {
        assert!(!nav("products", 1).can_switch_store());
        assert!(nav("products", 2).can_switch_store());
    }
}
