use askama::Template;
use crate::models::{
    data_transfer_objects::product::{Product,RoutesProductPanelGroup},
    models_x::ProductForPage1
};
use serde_json::{Value, from_value};
use crate::controllers::components::cart_add_controller::ROUTES as CART_ROUTES;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use bigdecimal::BigDecimal;
use crate::models::components::stars::Stars;
use crate::models::components::price::Price;
use crate::models::components::quantity::Quantity;
#[derive(Template,Clone,Debug)]
#[template(path = "components/product_detail_principal/product_detail_principal.html")]
pub struct ProductDetailPrincipal{
    product_data : ProductDetailPrincipalData
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductDetailPrincipalData{
    pub id: Uuid,
    pub name: String,
    pub brand: String,
    pub description: String,
    /// Código ISO ("EUR"), no el símbolo: lo pone `Price` al pintar.
    pub currency: String,
    pub store_name: String,
    pub store_id: Uuid,
    pub images_product: Vec<ImageProduct>,
    pub variant_map: Vec<VariantMapItem>,
    pub variations_with_stock_status: Vec<Variation>,
    pub rating_average: f64,
    pub rating_count: i64
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageProduct{
    pub is_main : bool,
    pub image_url : String,
    /// Las variaciones en las que sale (`images_product_variations`); vacío:
    /// es del producto entero y sale en todas.
    #[serde(default)]
    pub variation_ids : Vec<Uuid>,
}

impl ImageProduct {
    /// Para `data-variations` de la galería: el JS la recoloca al elegir variación.
    pub fn variation_ids_csv(&self) -> String {
        self.variation_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",")
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variation{
    pub name : String,
    pub values : Vec<VariationValue>
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariationValue{
    pub value : String,
    pub in_stock : bool,
    pub is_default : bool
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariantAttribute {
    pub name: String,
    pub value: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariantMapItem {
    pub id: Uuid,
    /// `None` = la variación no tiene fila en `prices`. Es un `Option` para que
    /// UNA variación sin precio no tumbe el mapa entero: sin él, `from_value`
    /// fallaba para toda la lista y la ficha se quedaba sin variaciones.
    pub price: Option<BigDecimal>,
    pub stock: i32,
    pub attributes: Vec<VariantAttribute>,
}

// Implementación de Default para ImageProduct
impl Default for ImageProduct {
    fn default() -> Self {
        Self {
            is_main: true,
            image_url: "https://via.placeholder.com/500".to_string(),
            variation_ids: Vec::new(),
        }
    }
}

// Implementación de Default para Variation
impl Default for VariationValue {
    fn default() -> Self {
        Self {
            value: "Talla".to_string(),
            in_stock: false,
            is_default: false,
        }
    }
}

// Implementación de Default para Variation
impl Default for Variation {
    fn default() -> Self {
        Self {
            name: "Talla".to_string(),
            values : Vec::new()
        }
    }
}

// Implementación de Default para el modelo principal
impl Default for ProductDetailPrincipalData {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Producto genérico".to_string(),
            brand: "Marca blanca".to_string(),
            description: "Sin descripción disponible".to_string(),
            currency: "EUR".to_string(),
            store_name: "Tienda Principal".to_string(),
            store_id: Uuid::nil(),
            images_product: Vec::new(),
            variant_map: Vec::new(),
            variations_with_stock_status: Vec::new(),
            rating_average: 0.0,
            rating_count: 0,
        }
    }
}

impl ProductDetailPrincipalData {
    /// Las estrellas de la media, para la cabecera.
    pub fn stars(&self) -> Stars {
        Stars::new(self.rating_average)
    }

    /// "9 opiniones", "1 opinión".
    pub fn rating_count_label(&self) -> String {
        if self.rating_count == 1 {
            "1 opinión".to_string()
        } else {
            format!("{} opiniones", self.rating_count)
        }
    }

    /// La variación que sale seleccionada al entrar: la de los valores marcados
    /// `is_default` (la de más stock, ver `variacion_prioritaria`). Si no casa
    /// ninguna, la primera del mapa. Un producto sin variaciones tiene una sola,
    /// sin atributos, y casa siempre.
    pub fn initial_variant(&self) -> Option<&VariantMapItem> {
        let is_default = |attr: &VariantAttribute| {
            self.variations_with_stock_status.iter().any(|v| {
                v.name == attr.name && v.values.iter().any(|val| val.is_default && val.value == attr.value)
            })
        };
        self.variant_map
            .iter()
            .find(|item| item.attributes.iter().all(is_default))
            .or_else(|| self.variant_map.first())
    }

    /// Precio de la variación que sale seleccionada al entrar. Antes era el de
    /// la primera del mapa (la más barata), que no tenía por qué ser la elegida.
    /// `None` si no hay variaciones o esa no tiene precio.
    pub fn initial_price(&self) -> Option<&BigDecimal> {
        self.initial_variant().and_then(|v| v.price.as_ref())
    }

    /// El precio grande de la cabecera: el de la variación que sale
    /// seleccionada, o "No disponible". El JS lo cambia al elegir otra.
    pub fn price_tag(&self) -> Price {
        Price::maybe(self.initial_price(), &self.currency).lg()
    }

    /// Cantidad a añadir, de 1 a 99. El tope real es el stock de la variación
    /// elegida, y ese lo comprueba el servidor al añadir.
    pub fn quantity_tag(&self) -> Quantity {
        Quantity::new(1, 1, Some(99))
    }

    /// La galería para una variación, en el orden en que se enseña y si se ve:
    /// primero las imágenes de esa variación, después las del producto entero
    /// (sin enlaces) y al final, escondidas, las de otras variaciones. Si no
    /// tiene propias ni hay generales se ven todas: mejor eso que una galería
    /// vacía. El JS (`refreshGallery`) repite la regla al elegir otra.
    pub fn gallery_for(&self, variant_id: Option<&Uuid>) -> Vec<(&ImageProduct, bool)> {
        let is_own = |i: &ImageProduct| variant_id.is_some_and(|id| i.variation_ids.contains(id));
        let own: Vec<&ImageProduct> = self.images_product.iter().filter(|i| is_own(i)).collect();
        let general: Vec<&ImageProduct> = self.images_product.iter()
            .filter(|i| i.variation_ids.is_empty())
            .collect();
        if own.is_empty() && general.is_empty() {
            return self.images_product.iter().map(|i| (i, true)).collect();
        }
        let others = self.images_product.iter()
            .filter(|i| !i.variation_ids.is_empty() && !is_own(i));
        own.into_iter().chain(general).map(|i| (i, true))
            .chain(others.map(|i| (i, false)))
            .collect()
    }

    /// La galería al entrar: la de la variación que sale seleccionada.
    pub fn gallery(&self) -> Vec<(&ImageProduct, bool)> {
        self.gallery_for(self.initial_variant().map(|v| &v.id))
    }

    /// La imagen grande al entrar: la primera que se ve de la galería.
    pub fn main_image_url(&self) -> Option<&str> {
        self.gallery().into_iter().find(|(_, visible)| *visible).map(|(i, _)| i.image_url.as_str())
    }

    pub fn variant_map_json(&self) -> String {
        serde_json::to_string(&self.variant_map).unwrap_or_else(|_| "[]".to_string())
    }

    /// URL del endpoint de "añadir al carrito". No es un dato del producto,
    /// por eso es un método y no un campo: se lee de las rutas del carrito.
    pub fn cart_add_url(&self) -> &'static str {
        &CART_ROUTES.add
    }
}

impl From<ProductForPage1> for ProductDetailPrincipalData {
    fn from(db: ProductForPage1) -> Self {
        let default = ProductDetailPrincipalData::default();

        Self {
            id: db.id,
            name: db.name.unwrap_or(default.name),
            brand: db.brand.unwrap_or(default.brand),
            description: db.description.unwrap_or(default.description),
            currency: db.currency.unwrap_or(default.currency),
            store_name: db.store_name.unwrap_or(default.store_name),
            store_id: db.store_id.unwrap_or(default.store_id),

            images_product: parse_json(db.images_product, "images_product", &db.id)
                .unwrap_or(default.images_product),

            variant_map: parse_json(db.variant_map, "variant_map", &db.id)
                .unwrap_or(default.variant_map),

            variations_with_stock_status: parse_json(db.variations_with_stock_status, "variations_with_stock_status", &db.id)
                .unwrap_or(default.variations_with_stock_status),

            rating_average: db.rating_average,
            rating_count: db.rating_count,
        }
    }
}

/// Lee uno de los JSON que monta la consulta de la ficha. Si no encaja se usa
/// el valor por defecto, como antes, pero ahora queda en el log: con el `.ok()`
/// a secas, una variación sin precio dejaba la ficha sin variaciones y nadie se
/// enteraba de por qué.
fn parse_json<T: serde::de::DeserializeOwned>(value: Option<Value>, field: &str, product_id: &Uuid) -> Option<T> {
    match from_value::<T>(value?) {
        Ok(parsed) => Some(parsed),
        Err(e) => {
            println!("Ficha {}: no se pudo leer `{}`: {}", product_id, field, e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(url: &str, variation_ids: &[Uuid]) -> ImageProduct {
        ImageProduct { is_main: false, image_url: url.to_string(), variation_ids: variation_ids.to_vec() }
    }

    fn urls(gallery: Vec<(&ImageProduct, bool)>) -> Vec<(String, bool)> {
        gallery.into_iter().map(|(i, visible)| (i.image_url.clone(), visible)).collect()
    }

    #[test]
    fn galeria_primero_las_de_la_variacion() {
        let (red, blue) = (Uuid::new_v4(), Uuid::new_v4());
        let data = ProductDetailPrincipalData {
            images_product: vec![image("general", &[]), image("azul", &[blue]), image("roja", &[red])],
            ..Default::default()
        };
        assert_eq!(urls(data.gallery_for(Some(&red))),
            vec![("roja".into(), true), ("general".into(), true), ("azul".into(), false)]);
        // Sin propias: las generales, y las de otras escondidas
        assert_eq!(urls(data.gallery_for(Some(&Uuid::new_v4()))),
            vec![("general".into(), true), ("azul".into(), false), ("roja".into(), false)]);
    }

    #[test]
    fn galeria_sin_propias_ni_generales_las_ensena_todas() {
        let (red, blue) = (Uuid::new_v4(), Uuid::new_v4());
        let data = ProductDetailPrincipalData {
            images_product: vec![image("azul", &[blue]), image("roja", &[red])],
            ..Default::default()
        };
        assert_eq!(urls(data.gallery_for(Some(&Uuid::new_v4()))),
            vec![("azul".into(), true), ("roja".into(), true)]);
        assert_eq!(data.main_image_url(), Some("azul"));
    }
}
