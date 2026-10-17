use askama::Template;

/// Átomo: estrellas de valoración, solo para leer (las de opinar son radios y
/// van en product_reviews_v1). Pinta cinco estrellas para una nota de 0 a 5.
///
/// Se usa desde cualquier plantilla cuyo modelo importe `Stars`:
///
///     {{ Stars::new(reviews.average) }}
///     {{ Stars::sm(product.rating) }}
///
/// El CSS es `ui/styles/atoms/stars.css`.
#[derive(Template, Clone, Debug)]
#[template(path = "components/stars/templates/stars.html")]
pub struct Stars {
    pub value: f64,
    pub small: bool,
}

impl Stars {
    /// 20px de alto.
    pub fn new(value: impl Into<f64>) -> Self {
        Stars { value: value.into(), small: false }
    }

    /// 16px de alto, para la tarjeta de producto.
    pub fn sm(value: impl Into<f64>) -> Self {
        Stars { value: value.into(), small: true }
    }

    /// El símbolo del sprite de cada estrella: "full", "half" o "empty". Se
    /// redondea a la media estrella más cercana: 4,2 -> cuatro; 4,3 -> cuatro
    /// y media; 4,8 -> cinco.
    pub fn icons(&self) -> [&'static str; 5] {
        let mut icons = ["empty"; 5];
        for (i, icon) in icons.iter_mut().enumerate() {
            let star = (i + 1) as f64;
            if self.value >= star - 0.25 {
                *icon = "full";
            } else if self.value >= star - 0.75 {
                *icon = "half";
            }
        }
        icons
    }

    /// Para lectores de pantalla: "4,6 de 5 estrellas", "4 de 5 estrellas".
    pub fn label(&self) -> String {
        let tenths = (self.value * 10.0).round() as i64;
        let number = if tenths % 10 == 0 {
            (tenths / 10).to_string()
        } else {
            format!("{},{}", tenths / 10, tenths % 10)
        };
        format!("{} de 5 estrellas", number)
    }
}

/// Lo que pinta ya es HTML (y solo lleva números y textos fijos): así las
/// plantillas no tienen que escribir `|safe` en cada llamada.
impl askama::filters::HtmlSafe for Stars {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_to_nearest_half_star() {
        assert_eq!(Stars::new(5).icons(), ["full"; 5]);
        assert_eq!(Stars::new(0).icons(), ["empty"; 5]);
        assert_eq!(Stars::new(4.2).icons(), ["full", "full", "full", "full", "empty"]);
        assert_eq!(Stars::new(4.3).icons(), ["full", "full", "full", "full", "half"]);
        assert_eq!(Stars::new(4.8).icons(), ["full"; 5]);
        assert_eq!(Stars::new(2.5).icons(), ["full", "full", "half", "empty", "empty"]);
    }

    #[test]
    fn label_uses_decimal_comma() {
        assert_eq!(Stars::new(4.6).label(), "4,6 de 5 estrellas");
        assert_eq!(Stars::new(4).label(), "4 de 5 estrellas");
        assert_eq!(Stars::new(3.96).label(), "4 de 5 estrellas");
    }
}
