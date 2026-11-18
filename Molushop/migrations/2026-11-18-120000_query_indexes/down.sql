-- Deja los índices como estaban antes de esta migración.
DROP INDEX IF EXISTS idx_product_variations_product;
DROP INDEX IF EXISTS idx_images_product_product;
DROP INDEX IF EXISTS idx_category_product_product;
DROP INDEX IF EXISTS idx_product_seller_seller;
DROP INDEX IF EXISTS idx_images_product_variations_image;
DROP INDEX IF EXISTS idx_price_history_open;

CREATE INDEX idx_user_sessions_jti ON user_sessions (jti);
CREATE INDEX idx_user_sessions_is_revoked ON user_sessions (is_revoked);
