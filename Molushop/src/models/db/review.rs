//! Opiniones de los productos.

use diesel::prelude::*;

// Resumen de las estrellas de un producto: media, total y cuántas hay de cada
// estrella. Se agrega al vuelo con AVG/COUNT y no se guarda en ningún sitio
// (`product_summary_ratings` se borró en 2026-09-21-181355_ratings_fix), así que
// no hay dos fuentes de verdad que puedan desincronizarse.
#[derive(QueryableByName, Debug, Clone)]
pub struct RatingSummaryRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub total: i64,
    /// Ya viene como float8 desde la consulta: la media solo se pinta, no se
    /// opera con ella, así que no compensa arrastrar un BigDecimal.
    #[diesel(sql_type = diesel::sql_types::Double)]
    pub average: f64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub star_1: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub star_2: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub star_3: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub star_4: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub star_5: i64,
}
