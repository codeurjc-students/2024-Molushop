-- SKU único por tienda (no global: dos tiendas pueden repetir el mismo).
-- La tienda de cada variación se guarda en la propia variación porque un
-- índice único no puede llegar a ella por products -> product_seller.
-- Hoy un producto es de una sola tienda; si algún día lo venden varias, esto
-- pasaría a una tabla de ofertas por tienda.
ALTER TABLE product_variations
    ADD COLUMN store_id UUID REFERENCES seller (id) ON DELETE SET NULL;

UPDATE product_variations v
SET store_id = ps.seller_id
FROM product_seller ps
WHERE ps.product_id = v.product_id;

-- Sin SKU es NULL, nunca '' (el índice dejaría un solo '' por tienda).
UPDATE product_variations SET sku = NULLIF(btrim(sku), '') WHERE sku IS NOT NULL;

-- Las de productos ya borrados dejan libre su SKU (los pedidos guardan copia).
UPDATE product_variations v SET sku = NULL
FROM products p
WHERE p.id = v.product_id AND p.status = 3 AND v.sku IS NOT NULL;

CREATE UNIQUE INDEX product_variations_store_sku
    ON product_variations (store_id, sku)
    WHERE sku IS NOT NULL;

-- Toda variación nueva toma la tienda de su producto si no la trae: así no
-- depende de que cada INSERT (alta, seeds...) se acuerde de ponerla.
CREATE FUNCTION product_variations_fill_store() RETURNS trigger AS $$
BEGIN
    IF NEW.store_id IS NULL THEN
        SELECT ps.seller_id INTO NEW.store_id
        FROM product_seller ps
        WHERE ps.product_id = NEW.product_id
        LIMIT 1;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER product_variations_fill_store
    BEFORE INSERT ON product_variations
    FOR EACH ROW EXECUTE FUNCTION product_variations_fill_store();
