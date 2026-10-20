DROP TRIGGER IF EXISTS product_variations_fill_store ON product_variations;
DROP FUNCTION IF EXISTS product_variations_fill_store();
DROP INDEX IF EXISTS product_variations_store_sku;
ALTER TABLE product_variations DROP COLUMN IF EXISTS store_id;
