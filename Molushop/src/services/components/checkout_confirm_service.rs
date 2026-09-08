use diesel::prelude::*;
use diesel::sql_query;
use diesel_async::{AsyncConnection, RunQueryDsl};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::scoped_futures::ScopedFutureExt;
use bigdecimal::BigDecimal;
use uuid::Uuid;

use crate::schema::{carts, cart_products, customer_address, orders, order_items};
use crate::models::models_x::{Cart, CartItem1, NewCart, NewCustomerAddress, NewOrder, NewOrderItem};
use crate::models::components::cart_total_v1::{CartTotalData, CartItemData};
use crate::models::components::checkout_v1::ConfirmForm;
use crate::models::error::ServiceError;
use crate::services::components::cart_get_service::get_cart_items_conn;
use crate::services::components::checkout_get_service::parse_line_ids;

type DbPool = Pool<AsyncPgConnection>;

/// Lo que hace falta para llevar al usuario a su pedido recién creado.
pub struct OrderConfirmation {
    pub id: Uuid,
    pub order_number: i64,
}

/// Convierte "" en None. `street2` es el único campo opcional de la dirección y
/// un formulario HTML manda siempre la cadena vacía, nunca la ausencia.
fn none_if_empty(value: &str) -> Option<&str> {
    let v = value.trim();
    if v.is_empty() { None } else { Some(v) }
}

/// Confirma el pedido: dirección, `orders` + `order_items`, descuento de stock y
/// cierre del carrito, TODO dentro de una misma transacción. Si cualquier paso
/// falla no queda nada: ni pedido a medias, ni stock descontado, ni la dirección
/// suelta en `customer_address`.
///
/// Es la primera transacción real del proyecto; el resto de servicios encadenan
/// sentencias sueltas sobre una conexión del pool.
pub async fn confirm_order(
    user_id: &Uuid,
    form: &ConfirmForm,
    pool: &DbPool,
) -> Result<OrderConfirmation, ServiceError> {
    let mut conn = pool.get().await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    conn.transaction::<OrderConfirmation, ServiceError, _>(|conn| async move {
        // 1. El carrito ACTIVO del usuario. Si no hay, no hay nada que comprar.
        let cart: Cart = carts::table
            .filter(carts::user_id.eq(user_id))
            .filter(carts::status.eq(1_i16))
            .first::<Cart>(conn)
            .await
            .map_err(|_| ServiceError::EmptyCart)?;

        // 2. Releer las líneas DENTRO de la transacción. Del cliente sólo nos
        // fiamos de los ids: cantidades, precios y totales salen de aquí.
        let rows = get_cart_items_conn(user_id, conn).await
            .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

        // El filtrado es el mismo que hace el GET del checkout. Como se aplica
        // sobre líneas que YA son del usuario, un id ajeno o inventado
        // simplemente no casa con nada.
        //
        // La diferencia con el GET: aquí el hidden lo escribe SIEMPRE el
        // servidor con la lista explícita, así que un `lines` vacío no es "todo
        // el carrito", es una anomalía y se rechaza. Que hoy sea inalcanzable
        // depende del `is_empty()` de la plantilla, y esa invariante no puede
        // vivir en el HTML cuando lo que hay detrás mueve stock y dinero.
        let wanted = parse_line_ids(Some(&form.lines)).ok_or(ServiceError::EmptyCart)?;
        let rows: Vec<CartItem1> = rows.into_iter().filter(|r| wanted.contains(&r.id)).collect();

        if rows.is_empty() {
            return Err(ServiceError::EmptyCart);
        }

        // 3. Sin precio no se cobra. Antes que insertar un 0, se cae el pedido.
        if rows.iter().any(|r| r.price.is_none() || r.currency.is_none()) {
            return Err(ServiceError::MissingPrice);
        }

        // 4. Totales recalculados en el servidor. El navegador no manda importes
        // y aunque los mandara se ignorarían.
        let items: Vec<CartItemData> = rows.iter().cloned().map(CartItemData::from).collect();
        let totals = CartTotalData::from_items(items);
        let zero = BigDecimal::from(0);

        // 5. La dirección, en el mismo POST para no dejarla huérfana si el
        // pedido se cae.
        let address_id = Uuid::new_v4();
        diesel::insert_into(customer_address::table)
            .values(NewCustomerAddress {
                id: &address_id,
                customer_id: user_id,
                street1: form.street1.trim(),
                street2: none_if_empty(&form.street2),
                postal_code: form.postal_code.trim(),
                city: form.city.trim(),
                province: form.province.trim(),
            })
            .execute(conn)
            .await
            .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

        // 6. El pedido. `order_number` lo pone la BD. Los ship_* son una COPIA:
        // el usuario puede editar o borrar su dirección después de comprar.
        let order_id = Uuid::new_v4();
        let order_number: i64 = diesel::insert_into(orders::table)
            .values(NewOrder {
                id: &order_id,
                user_id,
                cart_id: Some(&cart.id),
                status: 1, // CONFIRMED
                currency: &totals.currency,
                subtotal: &totals.total,
                discount_total: &zero,
                shipping_total: &zero,
                total_amount: &totals.final_total,
                shipping_address_id: Some(&address_id),
                ship_street1: form.street1.trim(),
                ship_street2: none_if_empty(&form.street2),
                ship_postal_code: form.postal_code.trim(),
                ship_city: form.city.trim(),
                ship_province: form.province.trim(),
            })
            .returning(orders::order_number)
            .get_result(conn)
            .await
            .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

        for row in &rows {
            // 7. Descuento de stock con el mismo truco que `add_to_cart`: si la
            // condición no se cumple no se actualiza ninguna fila y `affected`
            // vale 0. Es lo que hace imposible vender por debajo de cero aunque
            // dos personas compren a la vez.
            let affected = sql_query(
                "UPDATE product_variations \
                 SET stock = stock - $2 \
                 WHERE id = $1 AND stock >= $2"
            )
                .bind::<diesel::sql_types::Uuid, _>(row.product_var_id)
                .bind::<diesel::sql_types::Integer, _>(row.quantity)
                .execute(conn)
                .await
                .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

            if affected == 0 {
                // Se cae el pedido ENTERO. El rollback deshace la dirección, el
                // pedido y el stock ya descontado de las líneas anteriores.
                return Err(ServiceError::NotEnoughStockLine {
                    line_id: row.id,
                    product_name: row.name.clone().unwrap_or_else(|| "Producto".to_string()),
                    available: row.stock,
                });
            }

            // 8. Snapshot completo de la línea. Se guarda lo que se vendió, no
            // una referencia al catálogo, que puede cambiar o desaparecer.
            let price = row.price.as_ref().ok_or(ServiceError::MissingPrice)?;
            let currency = row.currency.as_deref().ok_or(ServiceError::MissingPrice)?;
            let line_total = price * BigDecimal::from(row.quantity);
            let item_id = Uuid::new_v4();

            diesel::insert_into(order_items::table)
                .values(NewOrderItem {
                    id: &item_id,
                    order_id: &order_id,
                    product_var_id: &row.product_var_id,
                    product_name: row.name.as_deref().unwrap_or("Producto"),
                    variation_label: row.variation_label.as_deref(),
                    sku: row.sku.as_deref(),
                    image_url: row.image_url.as_deref(),
                    store_name: row.store_name.as_deref(),
                    unit_price: price,
                    currency: currency.trim(),
                    quantity: row.quantity,
                    line_total: &line_total,
                })
                .execute(conn)
                .await
                .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
        }

        // 9. Cierre del carrito. EL ORDEN IMPORTA: `uq_active_cart` es un índice
        // único parcial sobre status = 1 y no es diferible, así que no puede
        // haber dos carritos activos ni por un instante. Por eso se degrada el
        // viejo ANTES de crear el nuevo.
        diesel::update(carts::table.find(&cart.id))
            .set(carts::status.eq(2_i16)) // INACTIVE
            .execute(conn)
            .await
            .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

        // Las líneas compradas se quedan en el carrito ya inactivo: son el
        // registro de qué carrito produjo el pedido (`orders.cart_id`). Lo que
        // el usuario NO seleccionó se muda a un carrito nuevo y activo, para que
        // siga estando en /cart al volver.
        let bought: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
        let leftovers: Vec<Uuid> = cart_products::table
            .filter(cart_products::cart_id.eq(&cart.id))
            .filter(cart_products::id.ne_all(&bought))
            .select(cart_products::id)
            .load(conn)
            .await
            .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

        if !leftovers.is_empty() {
            let new_cart_id = Uuid::new_v4();
            diesel::insert_into(carts::table)
                .values(NewCart { id: &new_cart_id, user_id, status: 1 })
                .execute(conn)
                .await
                .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

            diesel::update(cart_products::table.filter(cart_products::id.eq_any(&leftovers)))
                .set(cart_products::cart_id.eq(&new_cart_id))
                .execute(conn)
                .await
                .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
        }

        Ok(OrderConfirmation { id: order_id, order_number })
    }.scope_boxed()).await
}
