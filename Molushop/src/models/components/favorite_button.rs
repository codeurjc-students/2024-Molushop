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
}

impl FavoriteButton {
    pub fn new(product_id: Uuid, active: bool) -> Self {
        Self { product_id, active, pop: false }
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

/// Respuesta de guardar o quitar: el botón solo, que sustituye al pulsado.
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
