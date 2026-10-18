use askama::Template;
use bigdecimal::BigDecimal;
use std::collections::HashMap;

use crate::constants::product_status::{PRODUCT_ACTIVE, PRODUCT_DRAFT};
use crate::models::components::price::Price;
use crate::models::data_transfer_objects::product::{Product, ProductVariation};

/// `/sandbox/variaciones`: tres formas de enseñar las variaciones de un
/// producto en el listado del panel, con los productos reales de una tienda.
/// Solo existe en desarrollo (ver `routes/sandbox.rs`).
#[derive(Template)]
#[template(path = "pages/sandbox_variations/sandbox_variations.html")]
pub struct SandboxVariationsPage {
    pub user_logged: bool,
    pub page_name: String,
    pub store_name: String,
    pub products: Vec<VariationsDemo>,
}

/// Una tabla de la matriz. Con tres atributos hay una por cada valor del
/// tercero (`title` = "Material: Plástico"); con dos, una sola sin título.
pub struct MatrixTable<'a> {
    pub title: Option<String>,
    pub cols: Vec<String>,
    /// (valor del primer atributo, celda por columna)
    pub rows: Vec<(String, Vec<Option<&'a ProductVariation>>)>,
}

pub struct VariationsDemo {
    pub product: Product,
}

impl VariationsDemo {
    pub fn titles(&self) -> &[String] {
        &self.product.variations_titles
    }

    /// `Borrow`: la plantilla la llama con un literal (`title(0)`) y con
    /// `loop.index0`, que Askama pasa por referencia.
    pub fn title(&self, i: impl std::borrow::Borrow<usize>) -> &str {
        self.titles().get(*i.borrow()).map(|t| t.as_str()).unwrap_or("")
    }

    /// Valor del atributo `i` de una variación. Se busca por nombre (el orden
    /// de `attributes` no tiene por qué ser el de los títulos) y, si no casa,
    /// por posición.
    pub fn value_of<'a>(&self, pv: &'a ProductVariation, i: usize) -> &'a str {
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

    // ---- Opción 1: resumen ----

    pub fn total(&self) -> usize {
        self.product.variations.len()
    }

    pub fn active(&self) -> usize {
        self.product.variations.iter().filter(|pv| pv.status == PRODUCT_ACTIVE).count()
    }

    /// Activas con stock 0: lo primero que querrá arreglar el vendedor.
    pub fn active_out_of_stock(&self) -> usize {
        self.product.variations.iter()
            .filter(|pv| pv.status == PRODUCT_ACTIVE && pv.stock <= 0)
            .count()
    }

    pub fn stock(&self) -> i64 {
        self.product.variations.iter().map(|pv| pv.stock as i64).sum()
    }

    pub fn price_min(&self) -> Option<Price> {
        self.product.variations.iter().min_by(|a, b| a.price.cmp(&b.price)).map(|pv| pv.price_tag())
    }

    /// Solo si es distinto del mínimo: con un precio único se pinta uno.
    pub fn price_max(&self) -> Option<Price> {
        let min = self.product.variations.iter().map(|pv| &pv.price).min()?;
        self.product.variations.iter()
            .max_by(|a, b| a.price.cmp(&b.price))
            .filter(|pv| &pv.price != min)
            .map(|pv| pv.price_tag())
    }

    /// (título, valores) de cada atributo, para las etiquetas del resumen.
    pub fn attribute_values(&self) -> Vec<(String, Vec<String>)> {
        self.titles().iter().enumerate()
            .map(|(i, title)| (title.clone(), self.values_of(i)))
            .collect()
    }

    // ---- Opción 2: tabla agrupada por el primer atributo ----

    pub fn groups(&self) -> Vec<(String, Vec<&ProductVariation>)> {
        let mut groups: Vec<(String, Vec<&ProductVariation>)> = Vec::new();
        for pv in &self.product.variations {
            let key = if self.titles().is_empty() { String::new() } else { self.value_of(pv, 0).to_string() };
            match groups.iter_mut().find(|(k, _)| *k == key) {
                Some((_, list)) => list.push(pv),
                None => groups.push((key, vec![pv])),
            }
        }
        // Dentro del grupo, por el orden de los valores (S, M, L…), no por el
        // de la base de datos.
        let orders: Vec<Vec<String>> = (1..self.titles().len()).map(|i| self.values_of(i)).collect();
        let rank = |pv: &ProductVariation| -> Vec<usize> {
            orders.iter().enumerate()
                .map(|(k, values)| values.iter().position(|v| v == self.value_of(pv, k + 1)).unwrap_or(usize::MAX))
                .collect()
        };
        for (_, list) in groups.iter_mut() {
            list.sort_by_key(|pv| rank(pv));
        }
        groups
    }

    /// Títulos de las columnas de la tabla agrupada: todos menos el primero.
    pub fn rest_titles(&self) -> &[String] {
        self.titles().get(1..).unwrap_or(&[])
    }

    pub fn rest_values(&self, pv: &ProductVariation) -> Vec<String> {
        (1..self.titles().len()).map(|i| self.value_of(pv, i).to_string()).collect()
    }

    // ---- Opción 4: tabla con filtros (la matriz la monta el JS desde la tabla) ----

    /// Los valores de todos los atributos, en el orden de los títulos.
    pub fn all_values(&self, pv: &ProductVariation) -> Vec<String> {
        (0..self.titles().len()).map(|i| self.value_of(pv, i).to_string()).collect()
    }

    /// (título, valores) con su posición, para los selectores de filtro.
    pub fn filters(&self) -> Vec<(usize, String, Vec<String>)> {
        self.attribute_values().into_iter().enumerate()
            .map(|(i, (title, values))| (i, title, values))
            .collect()
    }

    /// Todas las variaciones ordenadas por sus valores (S, M, L…), atributo a
    /// atributo, en vez de por el orden de la base de datos.
    pub fn sorted_variations(&self) -> Vec<&ProductVariation> {
        let orders: Vec<Vec<String>> = (0..self.titles().len()).map(|i| self.values_of(i)).collect();
        let mut list: Vec<&ProductVariation> = self.product.variations.iter().collect();
        list.sort_by_key(|pv| {
            orders.iter().enumerate()
                .map(|(i, values)| values.iter().position(|v| v == self.value_of(pv, i)).unwrap_or(usize::MAX))
                .collect::<Vec<usize>>()
        });
        list
    }

    // ---- Opción 3: matriz ----

    /// `None` con menos de dos atributos (no hay matriz que hacer) o con más
    /// de tres (habría que elegir qué dos van en filas y columnas).
    pub fn matrix(&self) -> Option<Vec<MatrixTable<'_>>> {
        let n = self.titles().len();
        if !(2..=3).contains(&n) {
            return None;
        }
        let rows = self.values_of(0);
        let cols = self.values_of(1);
        let pages: Vec<Option<String>> = if n == 3 {
            self.values_of(2).into_iter().map(Some).collect()
        } else {
            vec![None]
        };

        Some(pages.into_iter().map(|page| {
            let find = |row: &str, col: &str| self.product.variations.iter().find(|pv| {
                self.value_of(pv, 0) == row
                    && self.value_of(pv, 1) == col
                    && page.as_deref().map_or(true, |p| self.value_of(pv, 2) == p)
            });
            MatrixTable {
                title: page.as_ref().map(|p| format!("{}: {}", self.titles()[2], p)),
                cols: cols.clone(),
                rows: rows.iter()
                    .map(|row| (row.clone(), cols.iter().map(|col| find(row, col)).collect()))
                    .collect(),
            }
        }).collect())
    }

    /// El precio más repetido. En la matriz solo se escribe el de las celdas
    /// que se salen de él, para que la cuadrícula no sea una pared de precios.
    fn usual_price(&self) -> Option<&BigDecimal> {
        let mut counts: HashMap<&BigDecimal, usize> = HashMap::new();
        for pv in &self.product.variations {
            *counts.entry(&pv.price).or_default() += 1;
        }
        counts.into_iter().max_by_key(|(_, n)| *n).map(|(price, _)| price)
    }

    pub fn has_unusual_price(&self, pv: &ProductVariation) -> bool {
        self.usual_price().map_or(false, |usual| &pv.price != usual)
    }

    pub fn usual_price_tag(&self) -> Option<Price> {
        let pv = self.product.variations.first()?;
        self.usual_price().map(|price| Price::new(price, &pv.currency))
    }

    // ---- Estado de una variación (las tres opciones) ----

    /// Clase para atenuar o marcar la fila / celda.
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
