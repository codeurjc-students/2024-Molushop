-- Vuelve al modelo "la tienda es el usuario".
--
-- OJO, pierde datos: las tiendas creadas después de la migración tienen un id que no
-- es el de ningún usuario y no caben en el modelo viejo. Se BORRAN, y con ellas (en
-- cascada) sus filas de `product_seller`: sus productos quedan sin dueño.

DROP TABLE IF EXISTS store_members;

DROP INDEX IF EXISTS uq_seller_store_name;

ALTER TABLE seller
    DROP COLUMN status,
    DROP COLUMN created_at;

DELETE FROM seller WHERE id NOT IN (SELECT id FROM base_user);

ALTER TABLE seller
    ADD CONSTRAINT fk_user_seller FOREIGN KEY (id) REFERENCES base_user (id) ON DELETE CASCADE;
