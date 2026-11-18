use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_query;
use diesel::sql_types::{Json, Jsonb, Nullable, Text, Uuid as SqlUuid};
use diesel_async::RunQueryDsl;
use serde_json::Value;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;
use uuid::Uuid;
use crate::models::components::edit_product::{ProductEdit2,ImageCategories2,General,Variations,Variation,Attribute};
use crate::models::data_transfer_objects::product::Product;
use crate::models::db::Products;
use crate::services::product_specs_service::split_specs;
use crate::services::products_panel::{panel_product_on, MAIN_IMAGE_ORDER};
use crate::utils::db::get_connection;

/// Lo del editor que no está en la fila de `products`.
#[derive(QueryableByName)]
struct EditorExtras {
    /// Todas sus imágenes, en el orden del editor (`ImageCategories2`).
    #[diesel(sql_type = Nullable<Json>)]
    images_product: Option<Value>,
    /// La de la tarjeta del panel (`MAIN_IMAGE_ORDER`).
    #[diesel(sql_type = Nullable<Text>)]
    main_image: Option<String>,
    /// Las specs que propone su categoría.
    #[diesel(sql_type = Nullable<Jsonb>)]
    base_specs: Option<Value>,
}

/// Todo lo de la página "Editar producto": el editor y el producto como lo
/// enseña el panel. Una conexión y 5 consultas (producto, imágenes con specs
/// sugeridas, y categorías, variaciones y precios); antes eran 9 consultas en
/// 6 conexiones, leyendo el producto tres veces y las variaciones dos.
/// No comprueba propiedad: el handler llama antes a `product_in_store`.
pub async fn get_editor(store_id: &Uuid, product_id: &Uuid, pool: &DbPool) -> Result<(ProductEdit2, Product), Error> {
    use crate::schema::products;
    let connection = &mut get_connection(pool).await?;
    let product = products::table.find(product_id).first::<Products>(connection).await?;

    let extras = sql_query(format!(r#"
        SELECT
            (SELECT json_agg(json_build_object(
                    'id', i.id,
                    'image_url', i.image_url,
                    'is_main', i.is_main,
                    'display_order', i.display_order,
                    -- Las variaciones en las que sale (ninguna: en todas)
                    'variation_ids', COALESCE(
                        (SELECT json_agg(ipv.variation_id) FROM images_product_variations ipv WHERE ipv.image_id = i.id),
                        '[]'::json)
                ) ORDER BY i.is_main IS TRUE DESC, i.display_order)
             FROM images_product i WHERE i.product_id = $1) AS images_product,
            (SELECT image_url FROM images_product WHERE product_id = $1
             ORDER BY {MAIN_IMAGE_ORDER} LIMIT 1) AS main_image,
            (SELECT c.base_specs FROM category_product cp
             JOIN category c ON c.id = cp.category_id
             WHERE cp.product_id = $1 LIMIT 1) AS base_specs
    "#))
        .bind::<SqlUuid, _>(product_id)
        .get_result::<EditorExtras>(connection)
        .await?;

    let general = General {
        name: product.name.clone(),
        description: product.description.clone(),
        brand: product.brand.clone(),
        status: product.status,
    };
    let specs = split_specs(Some(&product.specs), extras.base_specs.as_ref());

    let (panel_product, variation_rows) = panel_product_on(connection, store_id, product, extras.main_image).await?;

    let variations: Vec<Variation> = variation_rows
        .iter()
        .map(|var| Variation {
            id: var.id,
            attributes: crate::utils::db_json::parse_db_json::<Vec<Attribute>>(var.attributes.clone(), "attributes", &var.id)
                .unwrap_or_default(),
            status: var.status,
        })
        .collect();

    let images: Vec<ImageCategories2> =
        crate::utils::db_json::parse_db_json(extras.images_product, "images_product", product_id).unwrap_or_default();

    let product_edit = ProductEdit2 {
        id: *product_id,
        general,
        images,
        variations: Variations { variations },
        specs,
    };
    Ok((product_edit, panel_product))
}
