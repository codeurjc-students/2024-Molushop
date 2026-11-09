DROP INDEX IF EXISTS uq_active_cart;

ALTER TABLE cart_products DROP CONSTRAINT IF EXISTS uq_cart_product;

ALTER TABLE carts ALTER COLUMN user_id DROP NOT NULL;
ALTER TABLE cart_products ALTER COLUMN product_var_id DROP NOT NULL;
ALTER TABLE cart_products ALTER COLUMN cart_id DROP NOT NULL;
