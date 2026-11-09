//! Productos: alta completa, lectura (panel, tarjetas, ficha) y edición de los datos generales.

use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_query;
use diesel::sql_types::Text;
use diesel::{insert_into, update};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::controllers::components::edit_product_controller::FormGeneral;
use crate::models::error::ServiceError;
use crate::models::db::{Category, NewProduct, ProductCardRow, ProductForPageRow, ProductForm, ProductWithImagesRow, Products};
use crate::services::create_product_variations::create_product_variations;
use crate::services::product_specs_service::clean_specs;
use crate::utils::db::get_connection;


type DbPool = Pool<AsyncPgConnection>;

/// Crea el producto ENTERO en una sola transacción: el producto, su vendedor, su
/// categoría, las variaciones y sus precios. Si falla cualquier paso no queda
/// nada escrito.
///
/// Antes eran cinco escrituras sueltas, cada una con su conexión: un fallo a
/// mitad dejaba productos sin categoría o, peor, variaciones sin fila en
/// `prices`, que dejaban la ficha sin selector de variaciones.
pub async fn create_full_product(seller_id: Uuid, form: &ProductForm, category_id: &str, pool: &DbPool) -> Result<Uuid, ServiceError> {
    let mut conn = pool.get().await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    conn.transaction::<Uuid, ServiceError, _>(async move |conn| {
        let product_id = insert_product(form, conn).await?;
        insert_product_seller(&seller_id, &product_id, conn).await?;
        insert_product_category(&product_id, category_id, conn).await?;
        create_product_variations(form, product_id, conn).await?;
        Ok(product_id)
    }).await
}

pub async fn insert_product_category(product_id: &Uuid, category_id: &str, connection: &mut AsyncPgConnection) -> Result<usize, Error> {
    use crate::schema::category_product as cp;
    insert_into(cp::table)
        .values((cp::category_id.eq(category_id), cp::product_id.eq(product_id)))
        .execute(connection)
        .await
}

pub async fn insert_product_seller(seller_id: &Uuid, product_id: &Uuid, connection: &mut AsyncPgConnection) -> Result<usize, Error> {
    use crate::schema::product_seller as ps;
    insert_into(ps::table)
        .values((ps::product_id.eq(product_id), ps::seller_id.eq(seller_id)))
        .execute(connection)
        .await
}

pub async fn insert_product(form:&ProductForm,connection:&mut AsyncPgConnection) -> Result<Uuid, ServiceError> {
    use crate::schema::products::dsl::*;
    let new_id = Uuid::new_v4();
    let cleaned_specs = clean_specs(&form.specs)?;
    //metodo para obtener los nombres de los titulos de las variaciones
    //obtenern ese jsson b y obtener los nombres de las keys
    // El JSON de variaciones lo escribe el navegador: si no es una lista de
    // objetos con `name`, se rechaza la petición en vez de tumbar el hilo.
    let titles:Option<Value> = match form.variations.as_ref(){
        Some(a)=>{
            let names = a.as_array()
                .ok_or(ServiceError::InvalidVariations)?
                .iter()
                .map(|item| item["name"].as_str().ok_or(ServiceError::InvalidVariations))
                .collect::<Result<Vec<&str>, ServiceError>>()?;
            Some(json!({ "titulos": names }))
        },
        None=>None

    };
    

    let new_product = NewProduct{
        id: &new_id,
        code: &form.code,
        name: &form.name,
        description: &form.description,
        brand: &form.brand,
        specs: &cleaned_specs,
        variations: form.variations.as_ref(),
        variation_titles: titles.as_ref(), //de momento, cambiar
        images: form.images.as_ref(),
    };
    insert_into(products).values(new_product).execute(connection).await?;
    Ok(new_id)
}

pub async fn get_product(product_id: &Uuid, pool: &DbPool) -> Result<Products, Error> {
    use crate::schema::products;
    let connection = &mut get_connection(pool).await?;
    products::table.find(product_id).first::<Products>(connection).await
}

pub async fn get_product_categories(product_id: &Uuid, pool: &DbPool) -> Result<Vec<Category>, Error> {
    use crate::schema::{category, category_product};
    let connection = &mut get_connection(pool).await?;
    category::table
        .inner_join(category_product::table.on(category::id.eq(category_product::category_id)))
        .filter(category_product::product_id.eq(product_id))
        .select(Category::as_select())
        .load::<Category>(connection)
        .await
}

pub async fn update_product_general(product_id: &Uuid, form: FormGeneral, pool: &DbPool) -> Result<usize, Error> {
    use crate::schema::products;
    let connection = &mut get_connection(pool).await?;
    update(products::table.find(product_id))
        .set((
            products::name.eq(form.name),
            products::description.eq(form.description),
            products::brand.eq(form.brand),
            products::status.eq(form.status),
        ))
        .execute(connection)
        .await
}

/// Las tarjetas de la portada, de `limit` en `limit` ("Cargar más").
/// Tarjetas de la portada y de la búsqueda. `patterns`: un ILIKE por palabra
/// (`CardFilter::like_patterns`; vacío, sin filtrar); `rank`: el texto para
/// ordenar por parecido (`CardFilter::rank_text`; `None`, por nombre);
/// `category`: esa categoría o cualquiera por debajo de ella. Lo que cuenta
/// como "a la venta" y la búsqueda los repite `search_service::category_counts`:
/// si cambian aquí, también allí.
pub async fn get_product_cards(limit: i64, offset: i64, patterns: &[String], rank: Option<&str>, category: Option<&str>, pool:&DbPool)->Result<Vec<ProductCardRow>,Error>{
    let connection = &mut get_connection(pool).await?;
    let results = sql_query(r#"
        select p.id,
            p.name,
            p.brand,
            (SELECT img.image_url 
            FROM images_product img 
            WHERE img.product_id = p.id 
            ORDER BY img.is_main DESC, img.id ASC 
            LIMIT 1) as image_url,
            pr.price,
            pr.currency,
            s.store_name,
            s.id store_id,
            -- Estrellas de la tarjeta (AVG da NULL sin filas).
            (SELECT COALESCE(AVG(r.rating_value), 0)::float8 FROM ratings r WHERE r.product_id = p.id) AS rating_average,
            (SELECT COUNT(*) FROM ratings r WHERE r.product_id = p.id) AS rating_count
        from products p
        -- El precio "desde": el más barato de las variaciones ACTIVAS que tienen
        -- precio. Una subconsulta y no un GROUP BY por moneda: así una
        -- variación sin precio no forma su propio grupo (salía el producto
        -- dos veces, uno a 0 €). Sin ninguno, la tarjeta dice "No disponible".
        left join lateral (
            select pr2.price, pr2.currency
            from product_variations pv
            join prices pr2 on pr2.variation_id = pv.id
            where pv.product_id = p.id and pv.status = 1
            order by pr2.price asc
            limit 1
        ) pr on true
        -- JOIN y no LEFT JOIN: solo se enseña lo que vende una tienda ACTIVA
        -- (seller.status = 1). Fuera quedan las tiendas pendientes, inactivas o
        -- rechazadas y los productos sin tienda (los "borrados" del panel).
        join product_seller ps on p.id=ps.product_id
        join seller s on ps.seller_id=s.id and s.status = 1
        where p.status = 1
        -- Búsqueda: cada palabra tiene que salir en el nombre o en la marca,
        -- sin contar acentos (f_unaccent, migración unaccent_search). Con un
        -- espacio delante del campo: "% h%" es una palabra que empieza por h
        -- (CardFilter::like_patterns).
        and not exists (
            select 1 from unnest($3::text[]) w
            where not (' ' || f_unaccent(coalesce(p.name, '')) ilike f_unaccent(w) or ' ' || f_unaccent(coalesce(p.brand, '')) ilike f_unaccent(w))
        )
        and ($4::text is null or exists (
            with recursive sub as (
                select id from category where id = $4
                union all
                select c.id from category c join sub on c.parent = sub.id
            )
            select 1 from category_product cp
            where cp.product_id = p.id and cp.category_id in (select id from sub)
        ))
        -- Con texto, por parecido: antes los nombres que empiezan por él, luego
        -- los que tienen una palabra que empieza por él, luego el resto.
        -- p.id desempata: sin él, dos productos con el mismo nombre podrían
        -- salir en dos tandas o en ninguna.
        order by case
                when $5::text is null then 0
                when f_unaccent(p.name) ilike (f_unaccent($5) || '%') then 0
                when ' ' || f_unaccent(p.name) ilike ('% ' || f_unaccent($5) || '%') then 1
                else 2
            end,
            p.name, p.id
        limit $1 offset $2;
    "#)
        .bind::<diesel::sql_types::BigInt, _>(limit)
        .bind::<diesel::sql_types::BigInt, _>(offset)
        .bind::<diesel::sql_types::Array<Text>, _>(patterns)
        .bind::<diesel::sql_types::Nullable<Text>, _>(category)
        .bind::<diesel::sql_types::Nullable<Text>, _>(rank)
        .load::<ProductCardRow>(connection).await?;
    Ok(results)
}

/// No filtra por vendedor: quien llame tiene que haber comprobado antes la
/// propiedad (`panel_role_service::can_edit_product`).
pub async fn get_product_with_images(product_id: &Uuid, pool:&DbPool)->Result<ProductWithImagesRow,Error>{
    let connection = &mut get_connection(pool).await?;
    let results = sql_query(r#"
        select p.*,
            json_agg(json_build_object(
                'id', i.id,
                'image_url', i.image_url,
                'is_main', i.is_main,
                'display_order', i.display_order,
                -- Las variaciones en las que sale (ninguna: en todas)
                'variation_ids', COALESCE(
                    (SELECT json_agg(ipv.variation_id) FROM images_product_variations ipv WHERE ipv.image_id = i.id),
                    '[]'::json)
            ) ORDER BY i.is_main IS TRUE DESC, i.display_order) FILTER (WHERE i.id IS NOT NULL) as images_product
        FROM products p
        LEFT JOIN images_product i on p.id = i.product_id
        WHERE p.id = $1
        GROUP BY p.id
    "#).bind::<diesel::sql_types::Uuid,_>(product_id)
        .get_result::<ProductWithImagesRow>(connection).await?;
    Ok(results)
}

pub async fn get_product_for_page(product_id: &Uuid, pool:&DbPool)->Result<ProductForPageRow,Error>{
    let connection = &mut get_connection(pool).await?;
    let results = sql_query(r#"
        WITH variacion_prioritaria AS (
            SELECT v.id, v.attributes, pr.price, pr.currency
            FROM Product_variations v
            LEFT JOIN prices pr ON v.id = pr.variation_id
            WHERE v.product_id = $1 AND v.status = 1
            ORDER BY (v.stock > 0) DESC, v.stock DESC
            LIMIT 1
        ),
        stock_por_atributo AS (
            SELECT
                v.product_id,
                attr->>'name' as attr_name,
                attr->>'value' as attr_value,
                SUM(v.stock) as total_stock
            FROM Product_variations v,
            LATERAL jsonb_array_elements(v.attributes) AS attr
            WHERE v.product_id = $1 AND v.status = 1
            GROUP BY v.product_id, attr_name, attr_value
        ),
        todas_las_combinaciones AS (
            SELECT jsonb_agg(
                jsonb_build_object(
                    -- NULL en la variación única de un producto sin variaciones:
                    -- como `[]`, que si no el mapa entero no se puede leer.
                    'attributes', COALESCE(v.attributes, '[]'::jsonb),
                    -- Como texto, no como número: serde_json lee los números
                    -- JSON como f64 y 150.99 llegaba a BigDecimal como
                    -- 150.990000000000009094947017729282379150390625.
                    -- Como cadena, BigDecimal la lee exacta.
                    'price', pr.price::text,
                    'id', v.id,
                    'stock', v.stock
                )
                ORDER BY pr.price ASC NULLS LAST
            ) as mapa
            FROM Product_variations v
            LEFT JOIN prices pr ON v.id = pr.variation_id
            WHERE v.product_id = $1 AND v.status = 1
        )
        SELECT
            p.id,
            p.name,
            p.brand,
            p.description,
            pr.currency,
            s.store_name,
            s.id store_id,
            img_agg.lista_imagenes as images_product,
            (SELECT mapa FROM todas_las_combinaciones) as variant_map,
            (
                SELECT jsonb_agg(
                    jsonb_build_object(
                        'name', v_def->>'name',
                        'values', (
                            SELECT jsonb_agg(
                                jsonb_build_object(
                                    'value', val->>'value',
                                    -- Lo lleva alguna variación activa (aunque esté
                                    -- agotada): si no, no hay nada que enseñar.
                                    'exists', sa.attr_value IS NOT NULL,
                                    'in_stock', COALESCE(sa.total_stock, 0) > 0,
                                    'is_default', EXISTS (
                                        SELECT 1 FROM variacion_prioritaria vp,
                                        LATERAL jsonb_array_elements(vp.attributes) as def_attr
                                        WHERE def_attr->>'name' = v_def->>'name'
                                        AND def_attr->>'value' = val->>'value'
                                    )
                                )
                            )
                            FROM jsonb_array_elements(v_def->'values') AS val
                            LEFT JOIN stock_por_atributo sa
                                ON sa.attr_name = v_def->>'name'
                                AND sa.attr_value = val->>'value'
                        )
                    )
                )
                FROM jsonb_array_elements(p.variations) AS v_def
            ) AS variations_with_stock_status,
            -- Media y número de opiniones para la cabecera, igual que el
            -- resumen de product_reviews_service (AVG da NULL sin filas).
            (SELECT COALESCE(AVG(r.rating_value), 0)::float8 FROM ratings r WHERE r.product_id = p.id) AS rating_average,
            (SELECT COUNT(*) FROM ratings r WHERE r.product_id = p.id) AS rating_count
        FROM Products p
        LEFT JOIN LATERAL (
            -- En el orden del carrito: la principal y después display_order.
            -- variation_ids: en qué variaciones sale (vacío: en todas).
            SELECT jsonb_agg(jsonb_build_object(
                'image_url', img.image_url,
                'is_main', img.is_main,
                'display_order', img.display_order,
                'variation_ids', COALESCE(
                    (SELECT jsonb_agg(ipv.variation_id) FROM images_product_variations ipv WHERE ipv.image_id = img.id),
                    '[]'::jsonb)
            ) ORDER BY img.is_main IS TRUE DESC, img.display_order ASC) as lista_imagenes
            FROM images_product img
            WHERE img.product_id = p.id
        ) img_agg ON true
        LEFT JOIN Product_variations pv ON p.id = pv.product_id
        LEFT JOIN prices pr ON pv.id = pr.variation_id
        -- Igual que en la portada: sin tienda ACTIVA el producto no existe
        -- para el público (la consulta no devuelve fila).
        JOIN product_seller ps ON p.id = ps.product_id
        JOIN seller s ON ps.seller_id = s.id AND s.status = 1
        WHERE p.id = $1 AND p.status = 1
        GROUP BY p.id, p.name, p.brand, pr.currency, s.store_name, s.id, img_agg.lista_imagenes;
    "#).bind::<diesel::sql_types::Uuid,_>(product_id)
        .get_result::<ProductForPageRow>(connection).await?;
    Ok(results)
}
