//aqui estará la los modelos de la página de create produc
use serde::{Deserialize, Serialize};
use crate::models::models_x;
use uuid::Uuid;
use bigdecimal::BigDecimal;
use crate::services::servicesX::get_variation_price;
use crate::models::components::price::Price;
use crate::models::components::variations_explorer::VariationsExplorer;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoutesProductPanelGroup{
    pub delete_product: &'static str,
    pub delete_product_modal: &'static str,
    //images
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Product{
    pub variations: Vec<ProductVariation>,
    pub name: String,
    pub status: i16,
    pub code: String,
    pub variations_titles: Vec<String>,
    pub categories: Vec<String>,
    pub id:Uuid,
    /// Solo se rellena en listados que mezclan tiendas (hoy ninguno).
    pub store_name: Option<String>,
    /// Tienda desde la que se lista: va en la URL de "Editar".
    pub store_id: Uuid,
    /// Imagen principal; `None` si el producto no tiene ninguna.
    pub image_url: Option<String>,
}

impl Product {
    /// Filtros, tabla y matriz de sus variaciones (listado del panel).
    pub fn variations_explorer(&self) -> VariationsExplorer<'_> {
        VariationsExplorer { product: self, bulk_url: None }
    }

    /// Lo mismo con un botón "Editar" por variación y la barra de edición en
    /// bloque, que manda a `bulk_url`/{id del producto} (página de edición).
    pub fn variations_explorer_editable<'b>(&'b self, bulk_url: &'b str) -> VariationsExplorer<'b> {
        VariationsExplorer { product: self, bulk_url: Some(bulk_url) }
    }
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductVariation{
    pub id: Uuid,
    /// `None`: la variación no tiene fila en `prices` para esa moneda.
    pub price: Option<BigDecimal>,
    /// Código ISO ("EUR"), como en `prices`. Ver `price_tag()`.
    pub currency: String,
    pub stock: i32,
    pub status: i16,
    pub variation: Vec<Variation>
    //images
}

impl ProductVariation{
    /// `currency` es la que se ha pedido: la de `otro_p` o, si falta el
    /// precio, la que se le pintará al lado de "No disponible".
    pub fn from(otro_pv:&models_x::ProductVariation,otro_p:Option<&models_x::Price>,currency:&str)->Self{
        let vars: Vec<Variation> = match &otro_pv.attributes {
            Some(vars_xtra) => serde_json::from_value(vars_xtra.clone()).unwrap(),
            None => Vec::new()
        };
        
        //let vars:Vec<Variation> =serde_json::from_value(otro.attributes).unwrap(); 
        ProductVariation{
            id: otro_pv.id,
            price: otro_p.map(|p| p.price.clone()),
            currency: currency.to_string(),
            stock: otro_pv.stock,
            status: otro_pv.status,
            variation: vars
        }
    }

    pub fn price_tag(&self) -> Price {
        Price::maybe(self.price.as_ref(), &self.currency)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variation{
    pub name: String,
    pub value: String,
}
/*
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variations {
    pub variations: Vec<ProductVariation>,
} 

impl IntoIterator for Variations {
    type Item = Variation;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
*/