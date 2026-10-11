-- Pedidos por tienda y estado de envío por línea.
--
-- Un pedido puede mezclar productos de varias tiendas, y cada tienda envía lo
-- suyo por su cuenta. Las decisiones del 2026-10-11 que gobiernan este diseño:
--   1. Cada tienda ve en su panel los pedidos que llevan líneas suyas
--                                                 -> `order_items.store_id`.
--   2. El estado se guarda POR LÍNEA: la tienda marca como enviadas o entregadas
--      sus líneas                                 -> `order_items.status`.
--   3. Aparecen dos estados nuevos, 3 SHIPPED y 4 DELIVERED, tanto en la línea
--      como en el pedido. `orders.status` se queda como RESUMEN de sus líneas: lo
--      recalcula el código cada vez que cambia una, nunca se edita a mano.

-- 1. De qué tienda es cada línea. `store_name` sigue siendo el snapshot que se
--    pinta (la tienda puede cambiar de nombre); `store_id` es para filtrar.
--    NULLable y SET NULL, como `shipping_address_id`: borrar una tienda no puede
--    llevarse por delante un pedido, y la línea conserva el nombre copiado.
ALTER TABLE order_items
    ADD COLUMN store_id UUID REFERENCES seller(id) ON DELETE SET NULL;

-- Las líneas que ya existen: la tienda del producto de su variación. Si el
-- producto tuviera varias, se prefiere la que coincide con el nombre copiado
-- (una tienda renombrada no coincide y entra por la otra rama). Las líneas de
-- productos "borrados" del panel (sin fila en `product_seller`) se quedan en NULL.
UPDATE order_items oi
SET store_id = (
    SELECT ps.seller_id
    FROM product_variations pv
    JOIN product_seller ps ON ps.product_id = pv.product_id
    JOIN seller s ON s.id = ps.seller_id
    WHERE pv.id = oi.product_var_id
    ORDER BY (s.store_name = oi.store_name) DESC NULLS LAST
    LIMIT 1
);

-- "Pedidos de mi tienda" se busca por tienda en cada visita al panel.
CREATE INDEX idx_order_items_store ON order_items (store_id);

-- 2. Estado de la línea, con los mismos valores que el pedido:
--      0: PENDING    1: CONFIRMED    2: CANCELLED    3: SHIPPED    4: DELIVERED
--    DEFAULT 1 como en `orders`: hoy no hay pasarela de pago, la línea nace confirmada.
ALTER TABLE order_items
    ADD COLUMN status SMALLINT NOT NULL DEFAULT 1 CHECK (status BETWEEN 0 AND 4);

-- Las líneas que ya existen heredan el estado de su pedido (confirmado o cancelado).
UPDATE order_items oi
SET status = o.status
FROM orders o
WHERE o.id = oi.order_id;

-- 3. El pedido admite los estados nuevos. El CHECK de la migración de pedidos iba
--    en línea con la columna, así que Postgres lo llamó `orders_status_check`.
ALTER TABLE orders DROP CONSTRAINT orders_status_check;
ALTER TABLE orders ADD CONSTRAINT orders_status_check CHECK (status BETWEEN 0 AND 4);
