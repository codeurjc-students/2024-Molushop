-- Pruebas manuales de la transacción de checkout
-- (services/components/checkout_confirm_service.rs, paso 6 parte 3).
--
-- Replica en SQL, en el mismo orden, lo que hace el servicio, para comprobar lo
-- que sólo se puede comprobar contra la BD de verdad: el orden de las sentencias
-- del cierre del carrito frente a uq_active_cart, y el truco de affected = 0.
--
-- Todo dentro de una transacción que termina en ROLLBACK: no deja basura.
-- Usuario de pruebas: MoluxoX, el único con carrito activo y líneas.

BEGIN;

CREATE TEMP TABLE t AS
SELECT
    '703cb39d-358c-4b13-82ee-382f11193136'::uuid              AS user_id,
    (SELECT c.id FROM carts c
     WHERE c.user_id = '703cb39d-358c-4b13-82ee-382f11193136'
       AND c.status = 1)                                      AS cart_id,
    gen_random_uuid()                                         AS order_id,
    gen_random_uuid()                                         AS address_id,
    gen_random_uuid()                                         AS new_cart_id;

-- La línea que se "compra" es la primera del carrito; las otras dos son las que
-- el usuario deja sin seleccionar.
CREATE TEMP TABLE comprada AS
SELECT cp.id, cp.product_var_id, cp.quantity
FROM cart_products cp
WHERE cp.cart_id = (SELECT cart_id FROM t)
ORDER BY cp.added_at ASC
LIMIT 1;

SELECT count(*) AS lineas_en_el_carrito FROM cart_products WHERE cart_id = (SELECT cart_id FROM t);  -- 3

-- ---------------------------------------------------------------------------
-- 1. La dirección y el pedido, como los inserta el servicio.
-- ---------------------------------------------------------------------------
INSERT INTO customer_address (id, customer_id, street1, postal_code, city, province)
SELECT address_id, user_id, 'Calle Falsa 123', '28001', 'Madrid', 'Madrid' FROM t;

INSERT INTO orders (
    id, user_id, cart_id, status, currency,
    subtotal, discount_total, shipping_total, total_amount,
    shipping_address_id, ship_street1, ship_postal_code, ship_city, ship_province
)
SELECT order_id, user_id, cart_id, 1, 'EUR',
       40.00, 0, 0, 40.00,
       address_id, 'Calle Falsa 123', '28001', 'Madrid', 'Madrid'
FROM t;

-- ---------------------------------------------------------------------------
-- 2. El CHECK de la moneda: el símbolo NO puede entrar.
--    (migración 2026-09-06-191958_currency_iso_guard)
-- ---------------------------------------------------------------------------
SAVEPOINT sp;
UPDATE orders SET currency = '€' WHERE id = (SELECT order_id FROM t);  -- ERROR esperado
ROLLBACK TO sp;

-- ---------------------------------------------------------------------------
-- 3. Descuento de stock con el truco de affected = 0.
-- ---------------------------------------------------------------------------
-- 3a. Con stock suficiente afecta 1 fila.
WITH upd AS (
    UPDATE product_variations pv
    SET stock = stock - c.quantity
    FROM comprada c
    WHERE pv.id = c.product_var_id AND pv.stock >= c.quantity
    RETURNING pv.id
)
SELECT count(*) AS filas_afectadas_con_stock FROM upd;  -- 1

-- 3b. Pidiendo más de lo que hay NO afecta ninguna fila: eso es lo que el
--     servicio detecta como NotEnoughStockLine y hace caer el pedido entero.
WITH upd AS (
    UPDATE product_variations pv
    SET stock = stock - 99999
    FROM comprada c
    WHERE pv.id = c.product_var_id AND pv.stock >= 99999
    RETURNING pv.id
)
SELECT count(*) AS filas_afectadas_sin_stock FROM upd;  -- 0

-- ---------------------------------------------------------------------------
-- 4. Las líneas del pedido, con el snapshot.
-- ---------------------------------------------------------------------------
INSERT INTO order_items (
    id, order_id, product_var_id,
    product_name, variation_label, sku, unit_price, currency, quantity, line_total
)
SELECT gen_random_uuid(), (SELECT order_id FROM t), c.product_var_id,
       p.name, NULL, pv.sku, 10.00, 'EUR', c.quantity, 10.00 * c.quantity
FROM comprada c
JOIN product_variations pv ON pv.id = c.product_var_id
JOIN products p ON p.id = pv.product_id;

SELECT count(*) AS lineas_del_pedido FROM order_items WHERE order_id = (SELECT order_id FROM t);  -- 1

-- ---------------------------------------------------------------------------
-- 5. EL ORDEN DEL CIERRE DEL CARRITO. Esto es lo que había que comprobar.
-- ---------------------------------------------------------------------------
-- 5a. Al revés (carrito nuevo ACTIVO antes de degradar el viejo) el índice
--     parcial uq_active_cart lo rechaza: no puede haber dos activos a la vez.
SAVEPOINT sp;
INSERT INTO carts (id, user_id, status)
SELECT new_cart_id, user_id, 1 FROM t;  -- ERROR esperado
ROLLBACK TO sp;

-- 5b. El orden del servicio: primero degradar el viejo...
UPDATE carts SET status = 2 WHERE id = (SELECT cart_id FROM t);

-- ...y sólo entonces crear el nuevo y mudar lo no comprado.
INSERT INTO carts (id, user_id, status)
SELECT new_cart_id, user_id, 1 FROM t;

UPDATE cart_products
SET cart_id = (SELECT new_cart_id FROM t)
WHERE cart_id = (SELECT cart_id FROM t)
  AND id <> (SELECT id FROM comprada);

-- ---------------------------------------------------------------------------
-- 6. Estado final: el pedido apunta a un carrito inactivo que conserva la línea
--    comprada, y lo no seleccionado sigue vivo en un carrito activo nuevo.
-- ---------------------------------------------------------------------------
SELECT
    (SELECT status FROM carts WHERE id = (SELECT cart_id FROM t))          AS carrito_viejo_status,      -- 2
    (SELECT status FROM carts WHERE id = (SELECT new_cart_id FROM t))      AS carrito_nuevo_status,      -- 1
    (SELECT count(*) FROM cart_products WHERE cart_id = (SELECT cart_id FROM t))     AS lineas_compradas, -- 1
    (SELECT count(*) FROM cart_products WHERE cart_id = (SELECT new_cart_id FROM t)) AS lineas_pendientes;-- 2

-- Sigue habiendo un solo carrito activo por usuario.
SELECT count(*) AS carritos_activos
FROM carts WHERE user_id = (SELECT user_id FROM t) AND status = 1;  -- 1

ROLLBACK;

-- Nota: el ROLLBACK no devuelve la secuencia de order_number, que no es
-- transaccional. Es normal que el primer pedido real no empiece en 1.
