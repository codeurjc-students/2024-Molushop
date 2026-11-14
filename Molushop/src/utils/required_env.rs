//! Variables sin las que la app no puede funcionar y que no tienen un valor
//! por defecto razonable. `main` las comprueba al arrancar junto con el resto
//! de la configuración: si falta alguna, escribe una línea por cada una y no
//! arranca, en vez de reventar con un `panic` (o, peor, al subir la primera
//! imagen). Las que tienen reglas propias (`PASETO_LOCAL_SECRET`, el correo,
//! `APP_ENV`) se comprueban en su módulo.

use std::env;

/// Base de datos y S3. `AWS_REGION` también: sin ella no se pueden montar las
/// URL de las imágenes ni reconocerlas al borrarlas.
const REQUIRED_VARS: &[&str] = &[
    "DATABASE_URL",
    "AWS_ACCESS_KEY_ID",
    "AWS_SECRET_ACCESS_KEY",
    "AWS_S3_BUCKET_NAME",
    "AWS_REGION",
];

/// Un problema por cada variable obligatoria que falta o está vacía.
pub fn check_required_vars() -> Vec<String> {
    missing_vars(REQUIRED_VARS, |name| env::var(name).ok())
}

fn missing_vars(names: &[&str], get: impl Fn(&str) -> Option<String>) -> Vec<String> {
    names
        .iter()
        .filter(|name| get(name).is_none_or(|value| value.trim().is_empty()))
        .map(|name| format!("{} must be set", name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simula el entorno con una lista fija, sin tocar las variables reales
    /// (los tests corren en paralelo).
    fn env_with(vars: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
        move |name| vars.iter().find(|(n, _)| *n == name).map(|(_, v)| v.to_string())
    }

    #[test]
    fn all_set_has_no_problems() {
        let get = env_with(&[("A", "1"), ("B", "2")]);
        assert!(missing_vars(&["A", "B"], get).is_empty());
    }

    #[test]
    fn reports_every_missing_or_blank_var() {
        let get = env_with(&[("A", "1"), ("B", "  ")]);
        assert_eq!(missing_vars(&["A", "B", "C"], get), vec!["B must be set", "C must be set"]);
    }

    #[test]
    fn required_list_includes_the_region() {
        assert!(REQUIRED_VARS.contains(&"AWS_REGION"));
        assert!(REQUIRED_VARS.contains(&"DATABASE_URL"));
    }
}
