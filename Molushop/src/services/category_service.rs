//! Categorías: las de primer nivel, las hijas de una y la cadena hasta la raíz.

use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_query;
use diesel::sql_types::Text;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::models::models_x::Category;


type DbPool = Pool<AsyncPgConnection>;

pub async fn get_root_categories(pool:&DbPool) -> Result<Vec<Category>,Error> {
    
    use crate::schema::category::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    let results = category.filter(depth.eq(0)).load::<Category>(connection).await;
    results
}

pub async fn get_child_categories(id_category:&String, pool: &DbPool) -> Result<Vec<Category>,Error> {
    use crate::schema::category::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    //let connection = &mut establish_connection();
    let results = category.filter(parent.eq(id_category)).load::<Category>(connection).await;
    results
}

pub async fn get_category_ancestors(id_category:&String, pool: &DbPool) -> Result<Vec<Category>,Error> {
    use crate::models::models_x::Category;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    println!("ID: {}",id_category);
    //el placeholder $1 es para evitar sql injection
    let ancestors:Result<Vec<Category>, Error>  = sql_query("WITH RECURSIVE Ancestors AS (SELECT id, name, parent, depth, base_specs, is_parent FROM Category WHERE id = $1 UNION ALL  SELECT c.id, c.name, c.parent, c.depth, c.base_specs, c.is_parent FROM Category c INNER JOIN Ancestors a ON c.id = a.parent) SELECT * FROM Ancestors order by depth")
    .bind::<Text,_>(id_category.to_string()).get_results(connection).await;
    
    ancestors
}
