use askama::Template;
use crate::models::data_transfer_objects::product::{Product,RoutesProductPanelGroup};

use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Template,Clone,Debug)]
#[template(path = "components/edit_product/templates/edit_product.html")]
pub struct EditProduct{
    pub product: ProductEdit2,
    pub routes_edit_product: &'static Routes,
    pub store_id: Uuid,
    /// El producto como en el listado: la pestaña Variaciones usa su explorador.
    pub panel_product: Product,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Routes{
    pub edit_general: &'static str,
    pub edit_images: &'static str,
    pub bulk_variations: &'static str,
    pub image_variations: &'static str,
    //pub delete_product_modal: &'static str,
    //images
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductEdit{
    pub id: Uuid,
    pub general: General,
    pub images: Images,
    pub variations: Variations
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductEdit2{
    pub id: Uuid,
    pub general: General,
    pub images: Vec<ImageCategories2>,
    pub variations: Variations
}

impl ProductEdit2 {
    /// La imagen de la cabecera, como en la tarjeta del listado: la marcada
    /// como principal y, si no hay, la primera.
    pub fn main_image(&self) -> Option<&str> {
        self.images.iter()
            .find(|i| i.is_main)
            .or_else(|| self.images.first())
            .map(|i| i.image_url.as_str())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct General{
    pub name: String,
    pub description: String,
    pub brand: String,
    pub status: i16,
    //images
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Images{
    pub images: Vec<ImageCategories>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageCategories{
    pub url: String,
    pub tipo: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageCategories2{
    pub id: Uuid,
    pub image_url: String,
    pub is_main: bool,
    pub display_order: i32,
    /// Las variaciones en las que sale; vacío: en todas.
    #[serde(default)]
    pub variation_ids: Vec<Uuid>,
}

impl ImageCategories2 {
    /// Para `data-variations` de la galería (image-variations.js).
    pub fn variation_ids_csv(&self) -> String {
        self.variation_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",")
    }
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Categories{

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Prices{
    pub prices: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stock{
    pub stock: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Advanced{
    pub purchase_note: String,

}
/////////VARIATIONS//////////
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variations{
    //
    pub variations: Vec<Variation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variation{
    pub id: Uuid,
    pub attributes: Vec<Attribute>, 
    pub status: i16
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attribute{
    pub name: String,
    pub value: String
}

