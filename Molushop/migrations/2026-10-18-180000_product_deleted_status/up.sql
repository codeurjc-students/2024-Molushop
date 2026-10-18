-- "Eliminar" del panel deja de borrar `product_seller` (que dejaba el producto
-- huérfano) y marca el producto como borrado: status 3 (DELETED). La fila se
-- queda porque `order_items` y las opiniones apuntan a sus variaciones.
-- Valores: 0 DRAFT, 1 ACTIVE, 2 INACTIVE, 3 DELETED (src/constants/product_status.rs).
ALTER TABLE products DROP CONSTRAINT products_status_check;
ALTER TABLE products ADD CONSTRAINT products_status_check CHECK (status BETWEEN 0 AND 3);

-- Los borrados con el sistema anterior: productos sin ninguna tienda.
UPDATE products p SET status = 3
WHERE NOT EXISTS (SELECT 1 FROM product_seller ps WHERE ps.product_id = p.id);
