--Pruebas de borrado de precio
 SELECT pv.id AS variation_id, pv.sku, pv.attributes, pr.id AS price_id, pr.price, pr.currency, pr.start_date
  FROM product_variations pv
  LEFT JOIN prices pr ON pr.variation_id = pv.id
  WHERE pv.product_id = '8d5881e0-dcbc-4733-a696-7f33141966c3'
  ORDER BY pv.sku;
 
SELECT *
  FROM product_variations pv
  LEFT JOIN prices pr ON pr.variation_id = pv.id
  WHERE pv.product_id = '8d5881e0-dcbc-4733-a696-7f33141966c3'
  ORDER BY pv.sku;
 --select * from product_variation where product_id = '8d5881e0-dcbc-4733-a696-7f33141966c3'
  --necesito una carpeta con todas las tablas separadas para tenerlas ordenadas
  -- y que luego obvimaente estén 
  --Elige un variation_id de la lista y ponlo en lugar de <VARIATION_ID> en los pasos siguientes.

  --2. Guardar la copia y borrar el precio
select * from prices;

  BEGIN;

  CREATE TABLE IF NOT EXISTS prices_backup_prueba AS
  SELECT * FROM prices WHERE false;

  INSERT INTO prices_backup_prueba
  SELECT * FROM prices WHERE variation_id = '4e1a8d3d-5b53-4366-8bf2-51c4321b12af';

  DELETE FROM prices WHERE variation_id = '4e1a8d3d-5b53-4366-8bf2-51c4321b12af';

  COMMIT;

  --3. Volver a ponerlo
  BEGIN;

  INSERT INTO prices (id, variation_id, price, currency, start_date)
  SELECT id, variation_id, price, currency, start_date
  FROM prices_backup_prueba
  WHERE variation_id = '4e1a8d3d-5b53-4366-8bf2-51c4321b12af';

  DELETE FROM prices_backup_prueba WHERE variation_id = '4e1a8d3d-5b53-4366-8bf2-51c4321b12af';

  COMMIT;

  -- Cuando termines las pruebas:
  DROP TABLE IF EXISTS prices_backup_prueba;