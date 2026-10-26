use askama::Template;
use serde::{Serialize, Deserialize};
use crate::models::components::product_card_model::*;

#[derive(Template,Clone,Debug)]
#[template(path="components/product_card_group/templates/product_card_group.html")]
pub struct ProductCardGroup{
    pub products: Vec<ProductCardData>,
    /// `Some`: hay más productos y el botón "Cargar más" los pide aquí.
    pub more_url: Option<String>,
}

/// Una tanda de "Cargar más": las tarjetas y, si quedan, el botón siguiente,
/// sin la caja del grupo (van dentro de la que ya está en la página).
#[derive(Template,Clone,Debug)]
#[template(path="components/product_card_group/templates/items.html")]
pub struct ProductCardGroupPage{
    pub products: Vec<ProductCardData>,
    pub more_url: Option<String>,
}

/// Lo que filtra la rejilla de tarjetas: la portada va sin nada y la búsqueda
/// (`/search`) con el texto y la categoría. Viaja en la URL tal cual (`?q=`,
/// `?category=`), también en la de "Cargar más".
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct CardFilter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// Como mucho, palabras y caracteres de la búsqueda que se usan.
const MAX_WORDS: usize = 6;
const MAX_QUERY_CHARS: usize = 100;

impl CardFilter {
    /// Sin espacios de más; un campo vacío cuenta como si no viniera.
    pub fn new(q: Option<String>, category: Option<String>) -> Self {
        let clean = |value: Option<String>| value
            .map(|v| v.trim().chars().take(MAX_QUERY_CHARS).collect::<String>())
            .filter(|v| !v.is_empty());
        CardFilter { q: clean(q), category: clean(category) }
    }

    /// Un patrón de ILIKE por palabra (`%palabra%`), con `\`, `%` y `_`
    /// escapados para que se busquen tal cual. Sin texto, ninguno.
    pub fn like_patterns(&self) -> Vec<String> {
        self.q.as_deref().unwrap_or("")
            .split_whitespace()
            .take(MAX_WORDS)
            .map(|word| format!("%{}%", word.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")))
            .collect()
    }

    /// El mismo filtro con otra categoría (`None`: todas).
    pub fn with_category(&self, category: Option<&str>) -> Self {
        CardFilter { q: self.q.clone(), category: category.map(str::to_string) }
    }

    /// El mismo filtro sin el texto.
    pub fn without_query(&self) -> Self {
        CardFilter { q: None, category: self.category.clone() }
    }

    /// `base` con el filtro y, si se da, la página (`/search?q=...&page=2`).
    pub fn url(&self, base: &str, page: Option<i64>) -> String {
        #[derive(Serialize)]
        struct Params<'a> {
            #[serde(flatten)]
            filter: &'a CardFilter,
            #[serde(skip_serializing_if = "Option::is_none")]
            page: Option<i64>,
        }
        let query = serde_urlencoded::to_string(Params { filter: self, page }).unwrap_or_default();
        if query.is_empty() { base.to_string() } else { format!("{}?{}", base, query) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_fields_are_dropped() {
        let filter = CardFilter::new(Some("   ".into()), Some("".into()));
        assert!(filter.q.is_none() && filter.category.is_none());
        assert_eq!(filter.url("/search", None), "/search");
    }

    #[test]
    fn like_patterns_escape_wildcards() {
        let filter = CardFilter::new(Some(" 50%  off_x ".into()), None);
        assert_eq!(filter.like_patterns(), vec!["%50\\%%", "%off\\_x%"]);
    }

    #[test]
    fn url_encodes_the_filter_and_the_page() {
        let filter = CardFilter::new(Some("camiseta roja&".into()), Some("CLOT".into()));
        assert_eq!(filter.url("/search", Some(2)), "/search?q=camiseta+roja%26&category=CLOT&page=2");
        assert_eq!(filter.without_query().url("/search", None), "/search?category=CLOT");
    }
}
