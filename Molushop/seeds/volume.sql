-- VOLUMEN de productos para medir consultas e índices (EXPLAIN ANALYZE). SOLO
-- EN LOCAL: se niega a correr fuera de las bases de datos `Molushop` y
-- `molushop_test`. Nunca en Neon.
--
-- Uso, después de seeds/demo.sql (reparte los productos entre sus tiendas):
--     psql -v ON_ERROR_STOP=1 -v n=5000 -f seeds/volume.sql "$DATABASE_URL"
-- `n` es cuántos productos (5000 si no se pasa). Se puede repetir: cada vez
-- añade `n` más, con códigos VOL-<número> que siguen la cuenta.
--
-- Cada producto: una hoja de categoría (por turnos) en la tienda de demo de
-- su categoría raíz, nombre de palabras sueltas (para que la búsqueda tenga
-- coincidencias), de 1 a 4 variaciones por talla con stock y precio, y un
-- estado como el de una tienda de verdad: casi todos activos, algún borrador,
-- inactivo o eliminado, y alguna variación sin precio.
--
-- Para quitarlo: rehacer la base de datos (seeds/reset_dev.sh) y volver a
-- cargar demo.sql.

SELECT current_database() NOT IN ('Molushop', 'molushop_test') AS not_local \gset
\if :not_local
    \echo 'volume.sql: solo en las bases de datos locales (Molushop, molushop_test). No se hace nada.'
    \quit
\endif

\if :{?n}
\else
    \set n 5000
\endif

BEGIN;

-- Sin las tiendas de demo no hay a dónde ir.
DO $$
BEGIN
    IF (SELECT count(*) FROM seller WHERE id::text LIKE 'd2000000-%') < 5 THEN
        RAISE EXCEPTION 'volume.sql necesita seeds/demo.sql antes';
    END IF;
END $$;

-- Las hojas del árbol de categorías, con la tienda de demo de su raíz.
CREATE TEMP TABLE vol_leaves ON COMMIT DROP AS
WITH RECURSIVE tree AS (
    SELECT id, id AS root FROM category WHERE parent IS NULL
    UNION ALL
    SELECT c.id, t.root FROM category c JOIN tree t ON c.parent = t.id
)
SELECT row_number() OVER (ORDER BY t.id) AS k, t.id AS category,
       ('d2000000-0000-4000-8000-00000000000' ||
           CASE t.root WHEN 'CLOT' THEN 1 WHEN 'ELEC' THEN 2 WHEN 'HOME' THEN 3 WHEN 'SPORT' THEN 4 ELSE 5 END)::uuid AS store
FROM tree t
WHERE NOT EXISTS (SELECT 1 FROM category c WHERE c.parent = t.id);

CREATE TEMP TABLE vol_products ON COMMIT DROP AS
SELECT
    gen_random_uuid() AS id,
    s.i,
    base.offset_n + s.i AS num,
    l.category,
    l.store,
    -- 1 a 4 variaciones; con 1, variación única sin atributos
    1 + (s.i % 4) AS nvars,
    CASE
        WHEN s.i % 50 = 0 THEN 3          -- eliminado
        WHEN s.i % 33 = 0 THEN 2          -- inactivo
        WHEN s.i % 20 = 0 THEN 0          -- borrador
        ELSE 1
    END AS status,
    (ARRAY['Camiseta','Chaqueta','Pantalón','Zapatilla','Mochila','Lámpara','Silla','Mesa','Cojín','Taza',
           'Auriculares','Altavoz','Cargador','Funda','Teclado','Ratón','Monitor','Cámara','Reloj','Batidora',
           'Sartén','Toalla','Manta','Esterilla','Pelota','Raqueta','Casco','Crema','Champú','Perfume'])[1 + s.i % 30]
    || ' ' ||
    (ARRAY['clásico','urbano','ligero','compacto','premium','básico','deportivo','nórdico','vintage','moderno',
           'ecológico','plegable','resistente','suave','elegante','práctico','robusto','minimalista','artesano','esencial'])[1 + (s.i / 30) % 20]
    AS name,
    (ARRAY['Norte','Brisa','Puerto','Cumbre','Aroma','Sonora','Hierro','Prana','Lino & Co','Taller'])[1 + (s.i / 7) % 10] AS brand,
    round((5 + (abs(hashtext(s.i::text)) % 30000) / 100.0)::numeric, 2) AS price
FROM generate_series(1, :n) AS s(i)
CROSS JOIN (SELECT count(*) AS offset_n FROM products WHERE code LIKE 'VOL-%') AS base
JOIN vol_leaves l ON l.k = 1 + s.i % (SELECT count(*) FROM vol_leaves);

INSERT INTO products (id, code, name, description, brand, status, specs, variations, variation_titles)
SELECT id, 'VOL-' || num, name,
       'Producto generado para medir con volumen (' || name || ').',
       brand, status, '{}'::jsonb,
       CASE WHEN nvars = 1 THEN NULL ELSE
           jsonb_build_array(jsonb_build_object('name', 'Talla', 'values',
               (SELECT jsonb_agg(jsonb_build_object('value', t) ORDER BY j)
                FROM unnest((ARRAY['S','M','L','XL'])[1:nvars]) WITH ORDINALITY AS u(t, j)))) END,
       CASE WHEN nvars = 1 THEN NULL ELSE '{"titulos": ["Talla"]}'::jsonb END
FROM vol_products;

INSERT INTO product_seller (product_id, seller_id) SELECT id, store FROM vol_products;
INSERT INTO category_product (category_id, product_id) SELECT category, id FROM vol_products;

CREATE TEMP TABLE vol_variations ON COMMIT DROP AS
SELECT gen_random_uuid() AS id, p.id AS product_id, p.num, p.price, p.nvars, v.j,
       'VOL-' || p.num || '-' || v.j AS sku
FROM vol_products p
CROSS JOIN LATERAL generate_series(1, p.nvars) AS v(j);

-- `store_id` lo pone el trigger.
INSERT INTO product_variations (id, product_id, sku, attributes, status, stock)
SELECT id, product_id, sku,
       CASE WHEN nvars = 1 THEN NULL ELSE
           jsonb_build_array(jsonb_build_object('name', 'Talla', 'value', (ARRAY['S','M','L','XL'])[j])) END,
       1,
       abs(hashtext(sku)) % 60
FROM vol_variations;

-- Alguna variación (1 de cada 37) sin precio: sale "No disponible".
INSERT INTO prices (variation_id, price, currency, start_date)
SELECT id, price + (j - 1) * 2, 'EUR', now() - interval '30 days'
FROM vol_variations
WHERE (num * 4 + j) % 37 <> 0;

COMMIT;

-- Para que el planificador sepa cuántas filas hay ya.
ANALYZE products, product_variations, prices, price_history, product_seller, category_product;
