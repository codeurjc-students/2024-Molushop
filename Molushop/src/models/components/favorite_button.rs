use askama::Template;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::controllers::components::favorites_controller::ROUTES;
use crate::i18n::I18n;

/// Corazón de favoritos de un producto. Lo pintan la ficha (y más adelante las
/// tarjetas y el carrito) incluyendo `favorite_button/templates/button.html`
/// con una variable `fav` de este tipo; el endpoint devuelve el mismo botón ya
/// cambiado (`FavoriteButtonBody`).
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FavoriteButton {
    pub product_id: Uuid,
    /// Lleno (ya está en favoritos) o vacío. Sin sesión, siempre vacío: al
    /// pulsarlo, el 401 abre el login.
    pub active: bool,
    /// Late una vez al pintarse: solo el que devuelve "guardar", no los que
    /// ya estaban llenos al cargar la página.
    #[serde(default)]
    pub pop: bool,
    /// En la respuesta de guardar o quitar: sustituye a todos los corazones
    /// del mismo producto de la página (hx-swap-oob), no solo al pulsado. En
    /// el carrito, varias líneas pueden ser variaciones del mismo producto.
    #[serde(default)]
    pub sync: bool,
}

impl FavoriteButton {
    pub fn new(product_id: Uuid, active: bool) -> Self {
        Self { product_id, active, pop: false, sync: false }
    }

    pub fn synced(mut self) -> Self {
        self.sync = true;
        self
    }

    /// El selector de `hx-swap-oob`: todos los corazones de este producto.
    pub fn sync_selector(&self) -> String {
        format!(".fav-btn[data-product-id='{}']", self.product_id)
    }

    pub fn popped(mut self) -> Self {
        self.pop = true;
        self
    }

    /// Lo que hace el botón al pulsarlo: quitar si está lleno, guardar si no.
    /// Una URL por acción (y no un "toggle") para que un doble clic o dos
    /// pestañas no lo dejen al revés de lo que se ve.
    pub fn action_url(&self) -> String {
        let base = if self.active { &ROUTES.remove } else { &ROUTES.add };
        format!("{}/{}", base, self.product_id)
    }
}

/// Respuesta de guardar o quitar: el botón, que sustituye al pulsado y a los
/// demás del mismo producto (`synced`).
#[derive(Template)]
#[template(path = "components/favorite_button/templates/button.html")]
pub struct FavoriteButtonBody<'a> {
    pub fav: FavoriteButton,
    pub i18n: &'a I18n,
}

pub struct FavoritesRoutes {
    pub add: String,
    pub remove: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// El de la respuesta va con hx-swap-oob a todos los de su producto; los de
    /// la página, sin él (si no, htmx los tomaría por un swap al cargarlos)
    #[test]
    fn synced_button_targets_every_heart_of_the_product() {
        let id = Uuid::nil();
        let i18n = I18n::default();
        let html = FavoriteButtonBody { fav: FavoriteButton::new(id, true).synced(), i18n: &i18n }.render().unwrap();
        assert!(html.contains(&format!("data-product-id=\"{}\"", id)), "{}", html);
        assert!(html.contains(&format!("hx-swap-oob=\"outerHTML:.fav-btn[data-product-id='{}']\"", id)), "{}", html);

        let html = FavoriteButtonBody { fav: FavoriteButton::new(id, true), i18n: &i18n }.render().unwrap();
        assert!(!html.contains("hx-swap-oob"), "{}", html);
    }
}
