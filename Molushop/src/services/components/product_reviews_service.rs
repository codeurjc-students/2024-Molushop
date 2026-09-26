use askama::Template;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::sql_query;
use diesel_async::RunQueryDsl;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use uuid::Uuid;

use crate::models::components::product_reviews_v1::{
    ProductReviewsData, ProductReviewsV1Body, ReviewData, MAX_COMMENT_LEN, PAGE_SIZE,
};
use crate::models::error::ServiceError;
use crate::models::models_x::RatingSummaryRow;
use crate::schema::{base_user, order_items, orders, product_variations, ratings};

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
    user_id: Option<&Uuid>,
    pool: &DbPool,
) -> ProductReviewsData {
    // La página llega de la URL, así que puede venir con cualquier cosa.
    let page = page.max(1);
    let logged = user_id.is_some();

    let connection = &mut match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            println!("Error obteniendo conexión para las opiniones: {:?}", e);
            return ProductReviewsData::empty(*product_id, logged, false);
        }
    };

    // Si la comprobación falla, no se enseña el formulario: el POST lo iba a
    // rechazar igual, y es mejor no ofrecer algo que no va a funcionar.
    let can_review = match user_id {
        Some(uid) => match has_purchased(uid, product_id, connection).await {
            Ok(b) => b,
            Err(e) => {
                println!("Error comprobando la compra para opinar: {:?}", e);
                false
            }
        },
        None => false,
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
            return ProductReviewsData::empty(*product_id, logged, can_review);
        }
    };

    // Sin opiniones no hace falta la segunda consulta.
    if summary.total == 0 {
        return ProductReviewsData::empty(*product_id, logged, can_review);
    }

    // El "ver más" repinta la sección entera, así que se piden TODAS las
    // opiniones hasta la página actual, no sólo las de esa página. Es un LIMIT
    // más grande, no una consulta más cara: el orden sale tal cual del índice
    // idx_ratings_product_created (product_id, created_at DESC) y por eso no se
    // añade ningún criterio de desempate, que obligaría a ordenar de verdad.
    let rows: Vec<ReviewRow> = match ratings::table
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
            return ProductReviewsData::empty(*product_id, logged, can_review);
        }
    };

    let items: Vec<ReviewData> = rows.into_iter().map(review_from_row).collect();

    // La propia se busca aparte y no entre `items`: puede estar en una página
    // que todavía no se ha pedido, y el formulario la necesita igual. Si falla,
    // el formulario sale en blanco y el UPSERT la corregiría de todas formas.
    let my_review = match user_id {
        Some(uid) => match ratings::table
            .inner_join(base_user::table)
            .filter(ratings::product_id.eq(product_id))
            .filter(ratings::user_id.eq(uid))
            .select((
                ratings::id,
                base_user::username,
                ratings::rating_value,
                ratings::comment,
                ratings::created_at,
            ))
            .first::<ReviewRow>(connection)
            .await
            .optional()
        {
            Ok(row) => row.map(review_from_row),
            Err(e) => {
                println!("Error obteniendo la opinión del usuario: {:?}", e);
                None
            }
        },
        None => None,
    };

    ProductReviewsData::from_db(*product_id, summary, items, page, logged, can_review, my_review)
}

/// Lo que devuelven las dos consultas de opiniones (la lista y la propia).
type ReviewRow = (Uuid, String, i16, Option<String>, NaiveDateTime);

fn review_from_row((id, author, rating_value, comment, created_at): ReviewRow) -> ReviewData {
    ReviewData {
        id,
        author,
        rating_value,
        comment,
        created_at,
    }
}

/// Render del cuerpo de la sección: lo que devuelven el "ver más" y el POST de
/// una opinión nueva.
pub async fn get_product_reviews_body_render(
    product_id: &Uuid,
    page: i64,
    user_id: Option<&Uuid>,
    pool: &DbPool,
) -> String {
    let objeto = get_product_reviews_object(product_id, page, user_id, pool).await;
    ProductReviewsV1Body { reviews: objeto }.render().unwrap()
}

/// `orders.status` = 2 es CANCELLED (ver la migración de `orders`).
const ORDER_STATUS_CANCELLED: i16 = 2;

/// ¿Tiene el usuario algún pedido NO cancelado con alguna variación de este
/// producto? Es la regla para poder opinar. La opinión es por producto y el
/// pedido va por variación, de ahí el paso por `product_variations`.
///
/// Cuenta también los pedidos pendientes. Exigir que esté entregado queda para
/// cuando `orders` tenga ese estado.
async fn has_purchased(
    user_id: &Uuid,
    product_id: &Uuid,
    connection: &mut AsyncPgConnection,
) -> QueryResult<bool> {
    diesel::select(diesel::dsl::exists(
        order_items::table
            .inner_join(orders::table)
            .inner_join(product_variations::table)
            .filter(orders::user_id.eq(user_id))
            .filter(orders::status.ne(ORDER_STATUS_CANCELLED))
            .filter(product_variations::product_id.eq(product_id)),
    ))
    .get_result::<bool>(connection)
    .await
}

/// Guarda la opinión del usuario sobre un producto.
///
/// Es un UPSERT, no un INSERT: la decisión es "una opinión por usuario y
/// producto, editable", y el `uq_rating_user_product` de la tabla la impone. Sin
/// el ON CONFLICT, volver a opinar reventaría contra el UNIQUE en vez de
/// corregir lo que ya se dijo. El `updated_at` lo pone el trigger, que solo
/// salta en el UPDATE: una opinión nueva conserva `created_at = updated_at`.
pub async fn save_review(
    user_id: &Uuid,
    product_id: &Uuid,
    rating_value: i16,
    comment: Option<String>,
    pool: &DbPool,
) -> Result<(), ServiceError> {
    if !(1..=5).contains(&rating_value) {
        return Err(ServiceError::InvalidRating);
    }

    // Un comentario en blanco es "no he escrito nada", no una cadena vacía: la
    // tabla rechaza el texto vacío y la plantilla se salta el párrafo si es NULL.
    let comment = comment
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty());

    // Se cuenta en caracteres, no en bytes, igual que el char_length del CHECK.
    if let Some(text) = &comment {
        if text.chars().count() > MAX_COMMENT_LEN {
            return Err(ServiceError::CommentTooLong);
        }
    }

    let connection = &mut pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    // Que no salga el formulario no basta: el POST se puede hacer a mano.
    // Esto también cubre un producto inexistente, que nadie puede haber comprado.
    if !has_purchased(user_id, product_id, connection).await? {
        return Err(ServiceError::NotPurchased);
    }

    let result = diesel::insert_into(ratings::table)
        .values((
            ratings::id.eq(Uuid::new_v4()),
            ratings::product_id.eq(product_id),
            ratings::user_id.eq(user_id),
            ratings::rating_value.eq(rating_value),
            ratings::comment.eq(&comment),
        ))
        .on_conflict((ratings::product_id, ratings::user_id))
        .do_update()
        .set((
            ratings::rating_value.eq(rating_value),
            ratings::comment.eq(&comment),
        ))
        .execute(connection)
        .await;

    match result {
        Ok(_) => Ok(()),
        // Opinar sobre un producto que no existe. No se comprueba antes a
        // propósito: la FK ya lo sabe y así no hay una consulta de más en el
        // caso normal, que es que el producto exista.
        Err(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::ForeignKeyViolation,
            _,
        )) => Err(ServiceError::ProductNotFound),
        Err(e) => {
            println!("Error guardando la opinión: {:?}", e);
            Err(ServiceError::InternalServerError(e.to_string()))
        }
    }
}

/// Borra la opinión del usuario sobre un producto. Borrado duro: no hay columna
/// de estado ni moderación.
///
/// El `user_id` va en el propio WHERE, y es eso lo que impide borrar la de
/// otro: no hay un id de opinión que se pueda cambiar a mano. Mismo truco que
/// `get_order_object`.
///
/// No borrar nada (ya estaba borrada: doble clic, otra pestaña) no es un error;
/// el resultado que el usuario quería ya se cumple.
pub async fn delete_review(
    user_id: &Uuid,
    product_id: &Uuid,
    pool: &DbPool,
) -> Result<(), ServiceError> {
    let connection = &mut pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    diesel::delete(
        ratings::table
            .filter(ratings::product_id.eq(product_id))
            .filter(ratings::user_id.eq(user_id)),
    )
    .execute(connection)
    .await?;

    Ok(())
}
