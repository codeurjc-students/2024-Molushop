-- Vuelve al estado único por pedido, con tres valores.
--
-- OJO, pierde datos: SHIPPED (3) y DELIVERED (4) no caben en el CHECK viejo y los
-- pedidos que estén ahí vuelven a CONFIRMED (1). El estado de cada línea y su
-- tienda se pierden con las columnas.

UPDATE orders SET status = 1 WHERE status IN (3, 4);

ALTER TABLE orders DROP CONSTRAINT orders_status_check;
ALTER TABLE orders ADD CONSTRAINT orders_status_check CHECK (status BETWEEN 0 AND 2);

DROP INDEX IF EXISTS idx_order_items_store;

ALTER TABLE order_items
    DROP COLUMN status,
    DROP COLUMN store_id;
