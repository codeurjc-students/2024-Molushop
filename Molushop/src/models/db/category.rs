//! Categorías.

use diesel::prelude::*;
use serde_json::Value;
use crate::schema::*;

#[derive(serde::Serialize)]
#[derive(Queryable, Debug, Identifiable,Clone,Selectable)]
#[diesel(table_name = category)]
#[derive(QueryableByName)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub parent: Option<String>,
    pub depth: Option<i32>,
    pub base_specs: Option<Value>,
    pub is_parent: bool,
}

impl Category {
    /// Con los opcionales rellenos (para las plantillas del alta de producto).
    pub fn to_info(&self) -> CategoryInfo {
        CategoryInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            parent: self.parent.clone().unwrap_or_default(),
            depth: self.depth.unwrap_or(0),
            base_specs: self.base_specs.clone().unwrap_or(Value::Null),
            is_parent: self.is_parent,
        }
    }
}

#[derive(serde::Serialize)]
#[derive(Clone,Debug)]
pub struct CategoryInfo {
    pub id: String,
    pub name: String,
    pub parent: String,
    pub depth: i32,
    pub base_specs: Value,
    pub is_parent: bool,
}

/// Solo las specs base de la categoría (el JSONB).
#[derive(Queryable)]
#[diesel(table_name = category)]
#[derive(QueryableByName)]
pub struct BaseSpecs {
    pub base_specs: Option<Value>,
}
