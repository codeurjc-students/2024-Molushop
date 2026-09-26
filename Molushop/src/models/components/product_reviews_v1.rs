use askama::Template;
use chrono::NaiveDateTime;
use serde::Serialize;
use uuid::Uuid;

use crate::controllers::components::product_reviews_controller::ROUTES;
use crate::models::models_x::RatingSummaryRow;

/// Tope del comentario. El mismo número que el CHECK de la tabla: la
/// validación del servidor y la de la BD tienen que decir lo mismo, y el
/// `maxlength` del textarea lo repite para que ni se llegue a enviar.
pub const MAX_COMMENT_LEN: usize = 2000;

/// Cuántas opiniones entran de una tacada. El "ver más" pide la página
/// siguiente y el servicio devuelve TODAS las anteriores más esa: la sección se
/// repinta entera, así que la lista tiene que venir completa.
pub const PAGE_SIZE: i64 = 5;

#[derive(Template, Clone, Debug)]
#[template(path = "components/product_reviews_v1/product_reviews_v1.html")]
pub struct ProductReviewsV1 {
    pub reviews: ProductReviewsData,
}

/// Sólo el cuerpo, sin el `<link>` del wrapper. Es lo que devuelve el "ver más":
/// reinyectar el wrapper metería otra copia de la hoja de estilos en el body en
/// cada pulsación. Mismo reparto que en `cart_total_v1`.
#[derive(Template, Clone, Debug)]
#[template(path = "components/product_reviews_v1/templates/base.html")]
pub struct ProductReviewsV1Body {
    pub reviews: ProductReviewsData,
}

/// Las URLs del componente, rellenadas por el controlador (`lazy_static`), no
/// escritas a mano en la plantilla.
#[derive(Serialize, Debug, Clone)]
pub struct ProductReviewsRoutes {
    pub list: String,
    pub save: String,
    pub delete: String,
}

/// Una opinión tal como se pinta. `author` es el `username`, que es lo único
/// del usuario que se enseña: ni nombre real ni correo salen de la consulta.
// Sin `Serialize`, por lo mismo que `OrderSummaryData`: `created_at` es un
// `NaiveDateTime` y chrono no trae la feature `serde` activada en este proyecto.
// La sección se pinta con Askama y nunca sale como JSON.
#[derive(Debug, Clone)]
pub struct ReviewData {
    pub id: Uuid,
    pub author: String,
    pub rating_value: i16,
    pub comment: Option<String>,
    pub created_at: NaiveDateTime,
}

impl ReviewData {
    /// Mismo formato corto que la lista de pedidos.
    pub fn date_label(&self) -> String {
        self.created_at.format("%d/%m/%Y").to_string()
    }

    /// Las cinco estrellas de ESTA opinión, llenas o vacías. Nunca media: la
    /// media sólo aparece en el resumen de la cabecera.
    pub fn stars(&self) -> Vec<&'static str> {
        (1..=5)
            .map(|i| if i <= self.rating_value { "full" } else { "empty" })
            .collect()
    }

    /// Inicial para el círculo del avatar. Los usuarios no tienen foto.
    pub fn initial(&self) -> String {
        self.author
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "?".to_string())
    }

    /// La estrella es obligatoria, el comentario no: hay opiniones que son sólo
    /// una nota. La BD ya impide que llegue un comentario en blanco.
    pub fn comment_text(&self) -> &str {
        self.comment.as_deref().unwrap_or("")
    }

    pub fn has_comment(&self) -> bool {
        self.comment.is_some()
    }
}

/// Una fila del reparto de estrellas ("5 ★ ▇▇▇▇ 12").
#[derive(Debug, Clone)]
pub struct StarBar {
    pub star: i16,
    pub count: i64,
    pub percent: i64,
}

/// La sección entera. `items` trae las opiniones de la página 1 a la `page`
/// (acumuladas), y `total` es cuántas hay en la base de datos: de comparar las
/// dos sale el "ver más".
#[derive(Debug, Clone)]
pub struct ProductReviewsData {
    pub routes: &'static ProductReviewsRoutes,
    pub product_id: Uuid,
    pub items: Vec<ReviewData>,
    pub total: i64,
    pub average: f64,
    /// Contadores de 1 a 5 estrellas, en ese orden.
    pub counts: [i64; 5],
    pub page: i64,
    /// Sin sesión sale el botón que abre el login. Un visitante y un usuario
    /// identificado ven la MISMA lista: leer opiniones no pide sesión.
    pub logged: bool,
    /// Con sesión y con un pedido no cancelado que lleve el producto: sólo
    /// entonces sale el formulario. Con sesión y sin compra, el aviso de que
    /// opinar es para quien lo ha comprado.
    pub can_review: bool,
    /// La opinión que ya dejó el usuario, si la hay: precarga el formulario y
    /// saca el botón de borrar. Puede existir aunque `can_review` sea falso (el
    /// pedido se canceló después): entonces ya no se edita, pero sí se borra.
    pub my_review: Option<ReviewData>,
}

impl ProductReviewsData {
    /// Producto sin opiniones — y también lo que se devuelve si la consulta
    /// falla: la ficha enseña "todavía no hay opiniones" en vez de romperse,
    /// igual que el carrito cuando no puede leer sus líneas.
    pub fn empty(product_id: Uuid, logged: bool, can_review: bool) -> Self {
        Self {
            routes: &ROUTES,
            product_id,
            items: Vec::new(),
            total: 0,
            average: 0.0,
            counts: [0; 5],
            page: 1,
            logged,
            can_review,
            my_review: None,
        }
    }

    pub fn from_db(
        product_id: Uuid,
        summary: RatingSummaryRow,
        items: Vec<ReviewData>,
        page: i64,
        logged: bool,
        can_review: bool,
        my_review: Option<ReviewData>,
    ) -> Self {
        Self {
            routes: &ROUTES,
            product_id,
            items,
            total: summary.total,
            average: summary.average,
            counts: [summary.star_1, summary.star_2, summary.star_3, summary.star_4, summary.star_5],
            page,
            logged,
            can_review,
            my_review,
        }
    }

    /// Tope del comentario para el `maxlength` del textarea.
    pub fn max_comment_len(&self) -> usize {
        MAX_COMMENT_LEN
    }

    pub fn has_my_review(&self) -> bool {
        self.my_review.is_some()
    }

    /// Estrella que sale marcada en el formulario; 0 = ninguna (primera vez).
    pub fn my_rating(&self) -> i16 {
        self.my_review.as_ref().map(|r| r.rating_value).unwrap_or(0)
    }

    /// Texto que precarga el textarea; vacío si no hay opinión o no tenía comentario.
    pub fn my_comment(&self) -> &str {
        self.my_review.as_ref().map(|r| r.comment_text()).unwrap_or("")
    }

    /// Para la etiqueta "Tu opinión" de la lista. Por id y no por autor: el
    /// `username` es lo que se enseña, no lo que identifica.
    pub fn is_mine(&self, review_id: &Uuid) -> bool {
        self.my_review.as_ref().map(|r| &r.id == review_id).unwrap_or(false)
    }

    /// El borrado va por producto, no por id de la opinión: con el `user_id` de
    /// la sesión ya queda determinada, y así no hay id ajeno que se pueda probar.
    pub fn delete_url(&self) -> String {
        format!("{}/{}", self.routes.delete, self.product_id)
    }

    pub fn is_empty(&self) -> bool {
        self.total == 0
    }

    /// "4,3" — con coma, que es como se escribe un decimal en español.
    pub fn average_label(&self) -> String {
        format!("{:.1}", self.average).replace('.', ",")
    }

    /// Las cinco estrellas de la MEDIA. Media estrella a partir del cuarto
    /// (4,3 -> cuatro llenas; 4,6 -> cuatro llenas y media), que es lo que ya
    /// hace a ojo la cabecera de la ficha.
    pub fn stars(&self) -> Vec<&'static str> {
        (1..=5)
            .map(|i| {
                let i = i as f64;
                if self.average >= i - 0.25 {
                    "full"
                } else if self.average >= i - 0.75 {
                    "half"
                } else {
                    "empty"
                }
            })
            .collect()
    }

    /// El reparto, de 5 a 1 estrellas, que es el orden en que se lee.
    pub fn bars(&self) -> Vec<StarBar> {
        (1..=5)
            .rev()
            .map(|star| {
                let count = self.counts[(star - 1) as usize];
                StarBar {
                    star: star as i16,
                    count,
                    // Sin opiniones no hay barra que pintar; y así no se divide entre cero.
                    percent: if self.total > 0 { count * 100 / self.total } else { 0 },
                }
            })
            .collect()
    }

    pub fn shown(&self) -> i64 {
        self.items.len() as i64
    }

    pub fn has_more(&self) -> bool {
        self.shown() < self.total
    }

    /// El "ver más" pide la sección entera con una página más.
    pub fn next_page_url(&self) -> String {
        format!("{}/{}?page={}", self.routes.list, self.product_id, self.page + 1)
    }
}
