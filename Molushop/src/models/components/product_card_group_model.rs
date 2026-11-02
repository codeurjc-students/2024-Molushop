use askama::Template;
use serde::{Serialize, Deserialize};
use crate::models::components::product_card_model::*;
use crate::i18n::I18n;

#[derive(Template,Clone,Debug)]
#[template(path="components/product_card_group/templates/product_card_group.html")]
pub struct ProductCardGroup<'a>{
    pub products: Vec<ProductCardData>,
    /// `Some`: hay más productos y el botón "Cargar más" los pide aquí.
    pub more_url: Option<String>,
    pub i18n: &'a I18n,
}

/// Una tanda de "Cargar más": las tarjetas y, si quedan, el botón siguiente,
/// sin la caja del grupo (van dentro de la que ya está en la página).
#[derive(Template,Clone,Debug)]
#[template(path="components/product_card_group/templates/items.html")]
pub struct ProductCardGroupPage<'a>{
    pub products: Vec<ProductCardData>,
    pub more_url: Option<String>,
    pub i18n: &'a I18n,
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

/// Una palabra de hasta estas letras solo cuenta al principio de una palabra
/// del nombre o de la marca ("h": HeadPhonesPro sí, SuperPhone no); con más,
/// en cualquier sitio ("phone": también SuperPhone). Si no, con una o dos
/// letras salía casi todo.
pub const SHORT_WORD_CHARS: usize = 2;

impl CardFilter {
    /// Sin espacios de más; un campo vacío cuenta como si no viniera.
    pub fn new(q: Option<String>, category: Option<String>) -> Self {
        let clean = |value: Option<String>| value
            .map(|v| v.trim().chars().take(MAX_QUERY_CHARS).collect::<String>())
            .filter(|v| !v.is_empty());
        CardFilter { q: clean(q), category: clean(category) }
    }

    /// Un patrón de ILIKE por palabra, con `\`, `%` y `_` escapados para que se
    /// busquen tal cual. Sin texto, ninguno. Las consultas los comparan con el
    /// campo con un espacio delante (`' ' || nombre`): así `% h%` es "una
    /// palabra que empieza por h", también la primera. Las cortas
    /// (`SHORT_WORD_CHARS`), así; las demás, `%palabra%`.
    pub fn like_patterns(&self) -> Vec<String> {
        self.words()
            .map(|word| if word.chars().count() <= SHORT_WORD_CHARS {
                format!("% {}%", escape_like(word))
            } else {
                format!("%{}%", escape_like(word))
            })
            .collect()
    }

    /// El texto para ordenar por parecido (las palabras con un espacio entre
    /// ellas, escapadas para ILIKE): antes lo que empieza por él, luego lo que
    /// tiene una palabra que empieza por él. Sin texto, `None`.
    pub fn rank_text(&self) -> Option<String> {
        let words: Vec<String> = self.words().map(escape_like).collect();
        (!words.is_empty()).then(|| words.join(" "))
    }

    fn words(&self) -> impl Iterator<Item = &str> {
        self.q.as_deref().unwrap_or("").split_whitespace().take(MAX_WORDS)
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

/// `\`, `%` y `_` escapados para que ILIKE los busque tal cual.
fn escape_like(word: &str) -> String {
    word.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
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

    /// Una o dos letras: al principio de una palabra (el espacio del patrón);
    /// tres o más: en cualquier sitio. Cuenta letras, no bytes ("ñu" son dos).
    #[test]
    fn short_words_match_only_at_the_start_of_a_word() {
        let filter = CardFilter::new(Some("h ñu pho".into()), None);
        assert_eq!(filter.like_patterns(), vec!["% h%", "% ñu%", "%pho%"]);
    }

    #[test]
    fn rank_text_joins_the_escaped_words() {
        assert_eq!(CardFilter::new(Some("  super   50% ".into()), None).rank_text().as_deref(), Some("super 50\\%"));
        assert_eq!(CardFilter::new(None, None).rank_text(), None);
    }

    #[test]
    fn url_encodes_the_filter_and_the_page() {
        let filter = CardFilter::new(Some("camiseta roja&".into()), Some("CLOT".into()));
        assert_eq!(filter.url("/search", Some(2)), "/search?q=camiseta+roja%26&category=CLOT&page=2");
        assert_eq!(filter.without_query().url("/search", None), "/search?category=CLOT");
    }
}
