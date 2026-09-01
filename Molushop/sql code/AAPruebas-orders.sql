-- Pruebas manuales del esquema de pedidos (migración 2026-09-01-192454_orders).
-- Todo dentro de una transacción que termina en ROLLBACK: no deja basura.
-- Producto de pruebas: "Cortina Personalizable", variación Verde / S / Plástico.

BEGIN;

-- Datos de partida: un usuario cualquiera y la variación de pruebas con stock.
CREATE TEMP TABLE t AS
SELECT
    (SELECT id FROM base_user LIMIT 1)                        AS user_id,
    (SELECT pv.id
     FROM product_variations pv
     WHERE pv.product_id = '01ab75de-8d9f-41d6-acb5-0a42d7d5d6bc'
       AND pv.stock > 0
     ORDER BY pv.stock DESC
     LIMIT 1)                                                 AS var_id,
    gen_random_uuid()                                         AS order_id;

-- 1. Un pedido con su línea: debe pasar.
INSERT INTO orders (
    id, user_id, status, currency,
    subtotal, discount_total, shipping_total, total_amount,
    ship_street1, ship_postal_code, ship_city, ship_province
)
SELECT order_id, user_id, 1, 'EUR',
       20.00, 0, 0, 20.00,
       'Calle Falsa 123', '28001', 'Madrid', 'Madrid'
FROM t;

INSERT INTO order_items (
    id, order_id, product_var_id,
    product_name, variation_label, unit_price, currency, quantity, line_total
)
SELECT gen_random_uuid(), order_id, var_id,
       'Cortina Personalizable', 'Verde / S / Plástico', 10.00, 'EUR', 2, 20.00
FROM t;

SELECT order_number, status, total_amount FROM orders WHERE id = (SELECT order_id FROM t);

-- 2. La misma variación repetida en el mismo pedido: uq_order_item.
SAVEPOINT sp;
INSERT INTO order_items (
    id, order_id, product_var_id,
    product_name, unit_price, currency, quantity, line_total
)
SELECT gen_random_uuid(), order_id, var_id, 'Cortina Personalizable', 10.00, 'EUR', 1, 10.00
FROM t;  -- ERROR esperado: duplicate key uq_order_item
ROLLBACK TO sp;

-- 3. Cantidad 0: CHECK (quantity > 0).
SAVEPOINT sp;
INSERT INTO order_items (
    id, order_id, product_var_id,
    product_name, unit_price, currency, quantity, line_total
)
SELECT gen_random_uuid(), order_id,
       (SELECT id FROM product_variations WHERE id <> (SELECT var_id FROM t) LIMIT 1),
       'Otra', 10.00, 'EUR', 0, 0
FROM t;  -- ERROR esperado: check constraint order_items_quantity_check
ROLLBACK TO sp;

-- 4. Estado fuera de rango: CHECK (status BETWEEN 0 AND 2).
SAVEPOINT sp;
UPDATE orders SET status = 7 WHERE id = (SELECT order_id FROM t);  -- ERROR esperado
ROLLBACK TO sp;

-- 5. Borrar una variación ya vendida: ON DELETE RESTRICT.
SAVEPOINT sp;
DELETE FROM product_variations WHERE id = (SELECT var_id FROM t);  -- ERROR esperado
ROLLBACK TO sp;

-- 6. El trigger trg_orders_updated_at pisa cualquier updated_at que se le pase.
--    Ojo: dentro de una misma transacción NOW() es la hora de INICIO de la
--    transacción, así que updated_at sale igual a created_at, no mayor.
SAVEPOINT sp;
UPDATE orders SET updated_at = TIMESTAMPTZ '2000-01-01' WHERE id = (SELECT order_id FROM t);
SELECT (updated_at = created_at) AS trigger_pisa_el_valor  -- t
FROM orders WHERE id = (SELECT order_id FROM t);
ROLLBACK TO sp;

-- 7. Borrar el pedido se lleva sus líneas: ON DELETE CASCADE.
DELETE FROM orders WHERE id = (SELECT order_id FROM t);
SELECT count(*) AS lineas_huerfanas FROM order_items WHERE order_id = (SELECT order_id FROM t);  -- 0

ROLLBACK;

-- Nota: el ROLLBACK no devuelve la secuencia de order_number, que no es
-- transaccional. Es normal que el primer pedido real no empiece en 1.
