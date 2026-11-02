//! Categorías: las de primer nivel, las hijas de una y la cadena hasta la raíz.

use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_query;
use diesel::sql_types::Text;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::models::models_x::Category;
use crate::schema::category;
use crate::utils::db::get_connection;


type DbPool = Pool<AsyncPgConnection>;

pub async fn get_root_categories(pool: &DbPool) -> Result<Vec<Category>, Error> {
    let connection = &mut get_connection(pool).await?;
    category::table.filter(category::depth.eq(0)).load::<Category>(connection).await
}

pub async fn get_child_categories(category_id: &str, pool: &DbPool) -> Result<Vec<Category>, Error> {
    let connection = &mut get_connection(pool).await?;
    category::table.filter(category::parent.eq(category_id)).load::<Category>(connection).await
}

/// La categoría y todas las de encima, de la raíz hacia abajo.
pub async fn get_category_ancestors(category_id: &str, pool: &DbPool) -> Result<Vec<Category>, Error> {
    let connection = &mut get_connection(pool).await?;
    // $1: el id va como parámetro, nunca dentro del texto de la consulta
    sql_query("WITH RECURSIVE Ancestors AS (SELECT id, name, parent, depth, base_specs, is_parent FROM Category WHERE id = $1 UNION ALL  SELECT c.id, c.name, c.parent, c.depth, c.base_specs, c.is_parent FROM Category c INNER JOIN Ancestors a ON c.id = a.parent) SELECT * FROM Ancestors order by depth")
        .bind::<Text, _>(category_id)
        .get_results(connection)
        .await
}
