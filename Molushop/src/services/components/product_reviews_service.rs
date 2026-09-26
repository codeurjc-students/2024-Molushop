use askama::Template;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::sql_query;
use diesel_async::RunQueryDsl;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use uuid::Uuid;

use crate::models::components::product_reviews_v1::{
    ProductReviewsData, ProductReviewsV1Body, ReviewData, PAGE_SIZE,
};
use crate::models::models_x::RatingSummaryRow;
use crate::schema::{base_user, ratings};

type DbPool = Pool<AsyncPgConnection>;

/// Las opiniones de un producto: el resumen (media y reparto de estrellas) y la
/// lista, de la más reciente a la más antigua.
///
/// Son DOS consultas y no una: el resumen cuenta TODAS las opiniones y la lista
/// trae sólo las que se pintan. Meterlas en la misma repetiría la media en cada
/// fila, y es el número que más se lee de toda la sección.
///
/// Cuando algo falla se devuelve la sección vacía, no un error: la ficha del
/// producto tiene que seguir saliendo aunque las opiniones no se puedan leer.
pub async fn get_product_reviews_object(
    product_id: &Uuid,
    page: i64,
    pool: &DbPool,
) -> ProductReviewsData {
    // La página llega de la URL, así que puede venir con cualquier cosa.
    let page = page.max(1);

    let connection = &mut match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            println!("Error obteniendo conexión para las opiniones: {:?}", e);
            return ProductReviewsData::empty(*product_id);
        }
    };

    // COUNT ... FILTER hace el reparto de estrellas en una pasada. AVG devuelve
    // NULL sin filas y numeric con ellas: el COALESCE y el ::float8 dejan
    // siempre un f64, que es lo que espera RatingSummaryRow.
    let summary: RatingSummaryRow = match sql_query(
        r#"
        SELECT
            COUNT(*)                                        AS total,
            COALESCE(AVG(rating_value), 0)::float8          AS average,
            COUNT(*) FILTER (WHERE rating_value = 1)        AS star_1,
            COUNT(*) FILTER (WHERE rating_value = 2)        AS star_2,
            COUNT(*) FILTER (WHERE rating_value = 3)        AS star_3,
            COUNT(*) FILTER (WHERE rating_value = 4)        AS star_4,
            COUNT(*) FILTER (WHERE rating_value = 5)        AS star_5
        FROM ratings
        WHERE product_id = $1;
    "#,
    )
    .bind::<diesel::sql_types::Uuid, _>(product_id)
    .get_result::<RatingSummaryRow>(connection)
    .await
    {
        Ok(s) => s,
        Err(e) => {
            println!("Error obteniendo el resumen de opiniones: {:?}", e);
            return ProductReviewsData::empty(*product_id);
        }
    };

    // Sin opiniones no hace falta la segunda consulta.
    if summary.total == 0 {
        return ProductReviewsData::empty(*product_id);
    }

    // El "ver más" repinta la sección entera, así que se piden TODAS las
    // opiniones hasta la página actual, no sólo las de esa página. Es un LIMIT
    // más grande, no una consulta más cara: el orden sale tal cual del índice
    // idx_ratings_product_created (product_id, created_at DESC) y por eso no se
    // añade ningún criterio de desempate, que obligaría a ordenar de verdad.
    let rows: Vec<(Uuid, String, i16, Option<String>, NaiveDateTime)> = match ratings::table
        .inner_join(base_user::table)
        .filter(ratings::product_id.eq(product_id))
        .select((
            ratings::id,
            base_user::username,
            ratings::rating_value,
            ratings::comment,
            ratings::created_at,
        ))
        .order(ratings::created_at.desc())
        .limit(page * PAGE_SIZE)
        .load(connection)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            println!("Error obteniendo las opiniones del producto: {:?}", e);
            return ProductReviewsData::empty(*product_id);
        }
    };

    let items: Vec<ReviewData> = rows
        .into_iter()
        .map(|(id, author, rating_value, comment, created_at)| ReviewData {
            id,
            author,
            rating_value,
            comment,
            created_at,
        })
        .collect();

    ProductReviewsData::from_db(*product_id, summary, items, page)
}

/// Render del cuerpo de la sección, que es lo que devuelve el "ver más".
pub async fn get_product_reviews_body_render(product_id: &Uuid, page: i64, pool: &DbPool) -> String {
    let objeto = get_product_reviews_object(product_id, page, pool).await;
    ProductReviewsV1Body { reviews: objeto }.render().unwrap()
}
