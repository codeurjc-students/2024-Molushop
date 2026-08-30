-- Integridad del carrito.
-- Comprobado antes de escribir esta migración: 0 duplicados, 0 usuarios con
-- más de un carrito activo y 0 NULLs en las FK, así que no hace falta
-- limpieza previa de datos.

-- 1. Las FK del carrito nunca deben ser nulas: una línea sin carrito o sin
--    variación no significa nada, y un UNIQUE con NULLs no cubriría esas filas
--    (Postgres los trata como distintos por defecto).
ALTER TABLE cart_products ALTER COLUMN cart_id SET NOT NULL;
ALTER TABLE cart_products ALTER COLUMN product_var_id SET NOT NULL;
ALTER TABLE carts ALTER COLUMN user_id SET NOT NULL;

-- 2. Una sola línea por producto y carrito. Además de evitar duplicados por
--    clics rápidos, habilita el ON CONFLICT DO UPDATE de add_to_cart.
ALTER TABLE cart_products
    ADD CONSTRAINT uq_cart_product UNIQUE (cart_id, product_var_id);

-- 3. Un único carrito ACTIVO (status = 1) por usuario. Índice parcial: los
--    carritos en DRAFT (0) o INACTIVE (2) pueden ser varios, que es lo que
--    hará falta al convertirlos en pedidos.
CREATE UNIQUE INDEX uq_active_cart ON carts (user_id) WHERE status = 1;
