use askama::Template;

use crate::constants::product_status::{PRODUCT_ACTIVE, PRODUCT_DRAFT};
use crate::models::data_transfer_objects::product::{Product, ProductVariation};

/// Las variaciones de un producto en el listado del panel: filtros (uno por
/// atributo y por estado), tabla y matriz con los ejes a elegir. El servidor
/// solo pinta la tabla; filtrar y montar la matriz lo hace el JS a partir de
/// sus filas (`ui/components/variations_explorer/static/base/js/script.js`).
///
/// Se pinta como el átomo de precio:
///
///     {{ product.variations_explorer() }}
#[derive(Template)]
#[template(path = "components/variations_explorer/templates/variations_explorer.html")]
pub struct VariationsExplorer<'a> {
    pub product: &'a Product,
}

impl<'a> VariationsExplorer<'a> {
    pub fn titles(&self) -> &[String] {
        &self.product.variations_titles
    }

    /// `Borrow`: la plantilla la llama con `loop.index0`, que Askama pasa por
    /// referencia.
    pub fn title(&self, i: impl std::borrow::Borrow<usize>) -> &str {
        self.titles().get(*i.borrow()).map(|t| t.as_str()).unwrap_or("")
    }

    pub fn total(&self) -> usize {
        self.product.variations.len()
    }

    /// Valor del atributo `i` de una variación. Se busca por nombre (el orden
    /// de `attributes` no tiene por qué ser el de los títulos) y, si no casa,
    /// por posición.
    fn value_of<'v>(&self, pv: &'v ProductVariation, i: usize) -> &'v str {
        let title = self.titles().get(i);
        pv.variation.iter()
            .find(|v| Some(&v.name) == title)
            .or_else(|| pv.variation.get(i))
            .map(|v| v.value.as_str())
            .unwrap_or("—")
    }

    /// Valores distintos del atributo `i`, en el orden en que aparecen.
    fn values_of(&self, i: usize) -> Vec<String> {
        let mut values: Vec<String> = Vec::new();
        for pv in &self.product.variations {
            let value = self.value_of(pv, i);
            if !values.iter().any(|v| v == value) {
                values.push(value.to_string());
            }
        }
        values
    }

    /// Los valores de todos los atributos, en el orden de los títulos.
    pub fn all_values(&self, pv: &ProductVariation) -> Vec<String> {
        (0..self.titles().len()).map(|i| self.value_of(pv, i).to_string()).collect()
    }

    /// (posición, título, valores) de cada atributo, para los selectores.
    pub fn filters(&self) -> Vec<(usize, String, Vec<String>)> {
        self.titles().iter().enumerate()
            .map(|(i, title)| (i, title.clone(), self.values_of(i)))
            .collect()
    }

    /// Ordenadas por sus valores (S, M, L…), atributo a atributo, en vez de
    /// por el orden de la base de datos.
    pub fn sorted_variations(&self) -> Vec<&'a ProductVariation> {
        let orders: Vec<Vec<String>> = (0..self.titles().len()).map(|i| self.values_of(i)).collect();
        let mut list: Vec<&'a ProductVariation> = self.product.variations.iter().collect();
        list.sort_by_key(|pv| {
            orders.iter().enumerate()
                .map(|(i, values)| values.iter().position(|v| v == self.value_of(pv, i)).unwrap_or(usize::MAX))
                .collect::<Vec<usize>>()
        });
        list
    }

    /// Clase de estado de la fila; el JS la usa para filtrar y colorear la matriz.
    pub fn state_class(&self, pv: &ProductVariation) -> &'static str {
        match pv.status {
            PRODUCT_ACTIVE if pv.stock <= 0 => "is-out",
            PRODUCT_ACTIVE => "is-active",
            PRODUCT_DRAFT => "is-draft",
            _ => "is-inactive",
        }
    }

    pub fn status_label(&self, pv: &ProductVariation) -> &'static str {
        match pv.status {
            PRODUCT_ACTIVE => "Activo",
            PRODUCT_DRAFT => "Draft",
            _ => "Inactivo",
        }
    }

    pub fn status_variant(&self, pv: &ProductVariation) -> &'static str {
        match pv.status {
            PRODUCT_ACTIVE => "success",
            PRODUCT_DRAFT => "neutral",
            _ => "danger",
        }
    }
}

/// Lo que pinta ya es HTML: sin esto, `{{ product.variations_explorer() }}`
/// sale escapado en la tarjeta. Los valores del vendedor los escapa la propia
/// plantilla (ver el test `pinta_las_filas_con_sus_datos_y_escapa_los_valores`).
impl askama::filters::HtmlSafe for VariationsExplorer<'_> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::data_transfer_objects::product::Variation;
    use bigdecimal::BigDecimal;
    use uuid::Uuid;

    fn variation(attrs: &[(&str, &str)], status: i16, stock: i32) -> ProductVariation {
        ProductVariation {
            price: BigDecimal::from(10),
            currency: "EUR".to_string(),
            stock,
            status,
            variation: attrs.iter()
                .map(|(n, v)| Variation { name: n.to_string(), value: v.to_string() })
                .collect(),
        }
    }

    fn product(variations: Vec<ProductVariation>) -> Product {
        Product {
            variations,
            name: "Tienda de campaña".to_string(),
            status: PRODUCT_ACTIVE,
            code: "T1".to_string(),
            variations_titles: vec!["Tamaño".to_string(), "Color".to_string()],
            categories: Vec::new(),
            id: Uuid::nil(),
            store_name: None,
            store_id: Uuid::nil(),
            image_url: None,
        }
    }

    #[test]
    fn ordena_por_el_orden_de_aparicion_de_los_valores() {
        // Los atributos llegan en otro orden que los títulos: se casan por nombre.
        let p = product(vec![
            variation(&[("Tamaño", "1"), ("Color", "Rojo")], PRODUCT_DRAFT, 0),
            variation(&[("Color", "Verde"), ("Tamaño", "2")], PRODUCT_DRAFT, 0),
            variation(&[("Tamaño", "2"), ("Color", "Rojo")], PRODUCT_DRAFT, 0),
            variation(&[("Tamaño", "1"), ("Color", "Verde")], PRODUCT_DRAFT, 0),
        ]);
        let explorer = p.variations_explorer();
        let order: Vec<Vec<String>> = explorer.sorted_variations().iter().map(|pv| explorer.all_values(pv)).collect();
        assert_eq!(order, vec![
            vec!["1", "Rojo"], vec!["1", "Verde"], vec!["2", "Rojo"], vec!["2", "Verde"],
        ]);
        assert_eq!(explorer.filters()[1], (1, "Color".to_string(), vec!["Rojo".to_string(), "Verde".to_string()]));
    }

    #[test]
    fn pinta_las_filas_con_sus_datos_y_escapa_los_valores() {
        let p = product(vec![
            variation(&[("Tamaño", "1"), ("Color", "<b>Rojo</b>")], PRODUCT_ACTIVE, 7),
        ]);
        let html = p.variations_explorer().render().unwrap();
        assert!(html.contains(r#"<table class="ve-table">"#));
        assert!(html.contains(r#"data-a0="1""#));
        assert!(html.contains(r#"data-label="Color""#));
        assert!(html.contains(r#"data-state="is-active""#));
        assert!(!html.contains("<b>Rojo</b>"), "un valor del vendedor no puede llegar como HTML");
        // Con dos atributos hay matriz
        assert!(html.contains(r#"data-panel="matrix""#));
    }

    /// Como lo usa la tarjeta del panel: dentro de otra plantilla, sin `|safe`.
    #[derive(Template)]
    #[template(source = "<div>{{ product.variations_explorer() }}</div>", ext = "html")]
    struct Card<'a> {
        product: &'a Product,
    }

    #[test]
    fn dentro_de_otra_plantilla_no_se_escapa() {
        let p = product(vec![variation(&[("Tamaño", "1"), ("Color", "<b>Rojo</b>")], PRODUCT_ACTIVE, 7)]);
        let html = Card { product: &p }.render().unwrap();
        assert!(html.contains(r#"<div class="c-variations-explorer">"#), "el componente sale escapado");
        assert!(!html.contains("<b>Rojo</b>"));
    }

    #[test]
    fn estado_activa_sin_stock_aparte() {
        let p = product(Vec::new());
        let explorer = p.variations_explorer();
        assert_eq!(explorer.state_class(&variation(&[], PRODUCT_ACTIVE, 3)), "is-active");
        assert_eq!(explorer.state_class(&variation(&[], PRODUCT_ACTIVE, 0)), "is-out");
        assert_eq!(explorer.state_class(&variation(&[], PRODUCT_DRAFT, 5)), "is-draft");
    }
}
