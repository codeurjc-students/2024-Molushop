-- Índices para las consultas de la tienda y el panel (revisión de consultas,
-- 2026-11-18). Medidos con seeds/demo.sql + seeds/volume.sql (5000 productos):
-- la portada tardaba 4 s porque la consulta de las tarjetas buscaba el precio
-- "desde" de cada producto recorriendo `product_variations` entera.
--
-- Las claves primarias compuestas solo sirven para buscar por su PRIMERA
-- columna: `category_product (category_id, product_id)` no ayuda a buscar por
-- producto, ni `product_seller (product_id, seller_id)` por tienda.

-- Las variaciones de un producto: tarjetas (precio "desde"), ficha, panel.
CREATE INDEX idx_product_variations_product ON product_variations (product_id);

-- Las imágenes de un producto: tarjetas, ficha, editor, imagen principal.
CREATE INDEX idx_images_product_product ON images_product (product_id);

-- Las categorías de un producto: listado del panel, specs sugeridas, filtros.
CREATE INDEX idx_category_product_product ON category_product (product_id);

-- Los productos de una tienda: listado del panel, permisos.
CREATE INDEX idx_product_seller_seller ON product_seller (seller_id);

-- En qué variaciones sale una imagen (la PK empieza por variation_id).
CREATE INDEX idx_images_product_variations_image ON images_product_variations (image_id);

-- El trigger de precios cierra el registro abierto de la variación en cada
-- cambio de precio; sin índice recorre todo el historial, que solo crece.
CREATE INDEX idx_price_history_open ON price_history (variation_id, currency) WHERE end_date IS NULL;

-- Sobran: el UNIQUE de `jti` ya crea su índice, y uno sobre un booleano no lo
-- usa nadie (el middleware busca por `jti`; el borrado, por `expires_at`).
DROP INDEX IF EXISTS idx_user_sessions_jti;
DROP INDEX IF EXISTS idx_user_sessions_is_revoked;
