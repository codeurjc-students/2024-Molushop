-- Datos de DEMO de Molushop: un catálogo con aspecto real para enseñar la tienda.
--
-- Uso, sobre una base de datos con las migraciones y seeds/init.sql (las
-- categorías). No necesita seeds/dev.sql, pero puede ir encima:
--     psql -v ON_ERROR_STOP=1 -f seeds/demo.sql "$DATABASE_URL"
-- Va en una transacción. No es repetible (ids fijos): la segunda vez choca con
-- las claves primarias y no cambia nada.
--
-- Tiendas (activas), una por categoría raíz, cada una con su vendedor:
--   demo_atelier  "Atelier Lino"   Ropa
--   demo_voltio   "Voltio"         Electrónica
--   demo_nomada   "Casa Nómada"    Hogar y cocina
--   demo_cumbre   "Cumbre Sport"   Deportes y aire libre
--   demo_botica   "Botica Bella"   Belleza y cuidado personal
-- Los vendedores entran con la contraseña de desarrollo de seeds/dev.sql
-- (`Molushop-dev1`). OJO antes de cargarlo en una base de datos pública (Neon):
-- con esa contraseña cualquiera entra en sus paneles; cámbiala antes o después.
--
-- 40 productos, 8 por tienda, sin fotos (salen con la imagen por defecto; se
-- suben luego desde el panel). Solo en español: los productos no tienen
-- traducción en la base de datos.
--
-- Las variaciones salen del catálogo de abajo: el producto cruzado de sus dos
-- dimensiones (o una, o ninguna: variación única sin atributos), con SKU, stock
-- (algunas agotadas) y precio en EUR. Con `step`, cada valor de la primera
-- dimensión sube el precio (tallas de colchón, memoria...). Los productos con
-- `sale` tienen una rebaja: precio anterior más alto hace 60 días y el actual
-- desde hace 10 (lo apunta el trigger en `price_history`).
--
-- Ids: usuarios d1000000-..., tiendas d2000000-..., productos d3000000-...-NN;
-- las variaciones, md5 de su SKU.

BEGIN;

-- ---------------------------------------------------------------- Vendedores y tiendas
INSERT INTO base_user (id, username, name, lastname, email, password, active) VALUES
    ('d1000000-0000-4000-8000-000000000001', 'demo_atelier', 'Lucía',  'Ferrer', 'atelier@demo.local', '$argon2id$v=19$m=19456,t=2,p=1$FPFBTJ8GSMBbd/ALMKRKgw$Fm6+Zxh3IWC3i8nV4vPYigE9KuwJbIwZBR8Du1G8Ap0', true),
    ('d1000000-0000-4000-8000-000000000002', 'demo_voltio',  'Marcos', 'Ibáñez', 'voltio@demo.local',  '$argon2id$v=19$m=19456,t=2,p=1$FPFBTJ8GSMBbd/ALMKRKgw$Fm6+Zxh3IWC3i8nV4vPYigE9KuwJbIwZBR8Du1G8Ap0', true),
    ('d1000000-0000-4000-8000-000000000003', 'demo_nomada',  'Elena',  'Ruiz',   'nomada@demo.local',  '$argon2id$v=19$m=19456,t=2,p=1$FPFBTJ8GSMBbd/ALMKRKgw$Fm6+Zxh3IWC3i8nV4vPYigE9KuwJbIwZBR8Du1G8Ap0', true),
    ('d1000000-0000-4000-8000-000000000004', 'demo_cumbre',  'Javier', 'Soto',   'cumbre@demo.local',  '$argon2id$v=19$m=19456,t=2,p=1$FPFBTJ8GSMBbd/ALMKRKgw$Fm6+Zxh3IWC3i8nV4vPYigE9KuwJbIwZBR8Du1G8Ap0', true),
    ('d1000000-0000-4000-8000-000000000005', 'demo_botica',  'Nuria',  'Campos', 'botica@demo.local',  '$argon2id$v=19$m=19456,t=2,p=1$FPFBTJ8GSMBbd/ALMKRKgw$Fm6+Zxh3IWC3i8nV4vPYigE9KuwJbIwZBR8Du1G8Ap0', true);

-- seller.status: 1 activa.
INSERT INTO seller (id, rating, store_name, status) VALUES
    ('d2000000-0000-4000-8000-000000000001', 5, 'Atelier Lino', 1),
    ('d2000000-0000-4000-8000-000000000002', 4, 'Voltio',       1),
    ('d2000000-0000-4000-8000-000000000003', 5, 'Casa Nómada',  1),
    ('d2000000-0000-4000-8000-000000000004', 4, 'Cumbre Sport', 1),
    ('d2000000-0000-4000-8000-000000000005', 5, 'Botica Bella', 1);

INSERT INTO store_members (store_id, user_id)
SELECT ('d2000000-0000-4000-8000-00000000000' || n)::uuid, ('d1000000-0000-4000-8000-00000000000' || n)::uuid
FROM generate_series(1, 5) AS n;

-- ---------------------------------------------------------------- Catálogo
CREATE TEMP TABLE demo_catalog (
    n          int PRIMARY KEY,   -- el producto d3000000-...-NN
    store      int NOT NULL,      -- la tienda d2000000-...-N
    category   text NOT NULL,     -- una hoja de seeds/init.sql
    code       text NOT NULL,     -- también el prefijo de los SKU
    name       text NOT NULL,
    descr      text NOT NULL,
    brand      text NOT NULL,
    specs      jsonb NOT NULL,    -- con las claves que propone la categoría
    d1_name    text,              -- primera dimensión (NULL: sin variaciones)
    d1_values  text[],
    d2_name    text,              -- segunda dimensión (opcional)
    d2_values  text[],
    price      numeric(10,2) NOT NULL,
    step       numeric(10,2) NOT NULL DEFAULT 0,
    sale       boolean NOT NULL DEFAULT false
) ON COMMIT DROP;

INSERT INTO demo_catalog (n, store, category, code, name, descr, brand, specs, d1_name, d1_values, d2_name, d2_values, price, step, sale) VALUES
    -- Atelier Lino: ropa
    (1, 1, 'MENCL', 'DEMO-CAMISA-OXF', 'Camisa Oxford', 'Camisa de algodón Oxford con cuello abotonado y corte regular. Lavable a máquina.', 'Lino & Co',
        '{"size": "S a XL", "fit": "Regular", "material": "Algodón 100 %"}', 'Color', '{Azul claro,Blanco}', 'Talla', '{S,M,L,XL}', 39.90, 0, false),
    (2, 1, 'WOMCL', 'DEMO-VESTIDO-LINO', 'Vestido midi de lino', 'Vestido midi de lino lavado, con tirantes anchos y bolsillos laterales.', 'Lino & Co',
        '{"size": "XS a L", "fit": "Holgado", "material": "Lino"}', 'Color', '{Arena,Verde salvia}', 'Talla', '{XS,S,M,L}', 59.90, 0, false),
    (3, 1, 'MENCL', 'DEMO-JERSEY-MERINO', 'Jersey de lana merino', 'Jersey de punto fino en lana merino extrafina. Abriga sin abultar.', 'Norte Textil',
        '{"size": "S a L", "fit": "Slim", "material": "Lana merino"}', 'Color', '{Gris,Azul marino,Burdeos}', 'Talla', '{S,M,L}', 69.00, 0, true),
    (4, 1, 'WOMCL', 'DEMO-VAQUERO-RECTO', 'Vaquero recto de mujer', 'Vaquero de tiro alto y pierna recta en denim rígido.', 'Denim Sur',
        '{"size": "36 a 42", "fit": "Recto"}', 'Talla', '{36,38,40,42}', NULL, NULL, 49.95, 0, false),
    (5, 1, 'SHOES', 'DEMO-ZAPATILLA-PIEL', 'Zapatillas de piel blancas', 'Zapatillas minimalistas de piel con suela de goma cosida.', 'Paso Firme',
        '{"size": "38 a 44", "material": "Piel"}', 'Talla', '{38,39,40,41,42,43,44}', NULL, NULL, 79.00, 0, true),
    (6, 1, 'ACC', 'DEMO-CINTURON', 'Cinturón de piel', 'Cinturón de piel curtida al vegetal con hebilla de latón.', 'Paso Firme',
        '{"material": "Piel", "gender": "Unisex"}', 'Color', '{Marrón,Negro}', 'Talla', '{90,100,110}', 29.90, 0, false),
    (7, 1, 'CHILCL', 'DEMO-SUDADERA-NINO', 'Sudadera infantil con capucha', 'Sudadera de felpa suave con capucha y bolsillo canguro.', 'Pequeño Norte',
        '{"size": "4 a 8 años", "fit": "Regular"}', 'Color', '{Mostaza,Azul}', 'Talla', '{4 años,6 años,8 años}', 24.95, 0, false),
    (8, 1, 'ACC', 'DEMO-BOLSO-TOTE', 'Bolso tote de lona', 'Bolso grande de lona gruesa con asas largas y bolsillo interior.', 'Lino & Co',
        '{"material": "Lona de algodón", "gender": "Unisex"}', NULL, NULL, NULL, NULL, 19.90, 0, false),

    -- Voltio: electrónica
    (9, 2, 'HEADP', 'DEMO-AURIS-ANC', 'Auriculares con cancelación de ruido', 'Auriculares de diadema con cancelación activa de ruido y 30 horas de batería.', 'Sonora',
        '{"type": "Over-ear", "wireless": "Sí", "battery": "30 h"}', 'Color', '{Negro,Plata}', NULL, NULL, 199.00, 0, true),
    (10, 2, 'HEADP', 'DEMO-AURIS-SPORT', 'Auriculares deportivos', 'Auriculares inalámbricos resistentes al sudor, con ganchos para la oreja.', 'Sonora',
        '{"type": "In-ear", "wireless": "Sí"}', NULL, NULL, NULL, NULL, 49.90, 0, false),
    (11, 2, 'PHONE', 'DEMO-MOVIL-NIMBUS', 'Móvil Nimbus 6', 'Pantalla OLED de 6,4", triple cámara y carga rápida de 45 W.', 'Nimbus',
        '{"os": "Android", "screen_size": "6,4\""}', 'Almacenamiento', '{128 GB,256 GB}', 'Color', '{Grafito,Lavanda}', 399.00, 80, false),
    (12, 2, 'LAPTOP', 'DEMO-PORTATIL-14', 'Portátil ultraligero 14"', 'Portátil de 1,2 kg con pantalla 2,8K y hasta 14 horas de autonomía.', 'Áureo',
        '{"os": "Windows 11", "storage": "SSD NVMe"}', 'Memoria', '{16 GB / 512 GB,32 GB / 1 TB}', NULL, NULL, 1099.00, 400, false),
    (13, 2, 'TV', 'DEMO-TV-4K', 'Televisor 4K QLED', 'Televisor QLED 4K con HDR10+ y sistema operativo inteligente.', 'Visión',
        '{"display": "QLED 4K", "screen_size": "50\" a 65\""}', 'Tamaño', ARRAY['50"', '55"', '65"'], NULL, NULL, 449.00, 100, true),
    (14, 2, 'CAMERA', 'DEMO-CAMARA-MIRRORLESS', 'Cámara sin espejo 24 MP', 'Cámara sin espejo con sensor APS-C, vídeo 4K y estabilizador.', 'Óptica Prima',
        '{"megapixels": "24", "battery": "420 fotos"}', 'Kit', '{Solo cuerpo,Con objetivo 18-55 mm}', NULL, NULL, 849.00, 150, false),
    (15, 2, 'PHONE', 'DEMO-CARGADOR-65W', 'Cargador USB-C 65 W', 'Cargador GaN compacto con dos puertos USB-C, para móvil y portátil.', 'Voltio',
        '{"os": "Universal"}', NULL, NULL, NULL, NULL, 34.90, 0, false),
    (16, 2, 'CAMERA', 'DEMO-TRIPODE', 'Trípode de viaje', 'Trípode de aluminio plegable a 40 cm, con rótula de bola.', 'Óptica Prima',
        '{"battery": "No necesita"}', NULL, NULL, NULL, NULL, 59.00, 0, false),

    -- Casa Nómada: hogar y cocina
    (17, 3, 'KITCH', 'DEMO-CAFETERA-ESPRESSO', 'Cafetera espresso', 'Cafetera de 15 bares con vaporizador para espumar leche.', 'Aroma Casa',
        '{"power": "1350 W", "voltage": "230 V"}', 'Color', '{Negro,Crema}', NULL, NULL, 189.00, 0, true),
    (18, 3, 'KITCH', 'DEMO-BATIDORA-VASO', 'Batidora de vaso 1200 W', 'Batidora de vaso de vidrio de 1,5 litros con picahielo.', 'Aroma Casa',
        '{"power": "1200 W", "voltage": "230 V"}', NULL, NULL, NULL, NULL, 79.90, 0, false),
    (19, 3, 'BEDD', 'DEMO-SABANAS-PERCAL', 'Juego de sábanas de percal', 'Sábanas de percal de algodón de 200 hilos: bajera, encimera y funda.', 'Sueño Claro',
        '{"material": "Percal de algodón", "sizes": "90, 135 y 150 cm"}', 'Tamaño', '{90 cm,135 cm,150 cm}', 'Color', '{Blanco,Gris perla}', 45.00, 10, false),
    (20, 3, 'BEDD', 'DEMO-EDREDON', 'Edredón nórdico', 'Relleno nórdico de fibra hueca, cálido y ligero.', 'Sueño Claro',
        '{"material": "Microfibra", "sizes": "90, 135 y 150 cm"}', 'Tamaño', '{90 cm,135 cm,150 cm}', NULL, NULL, 69.00, 15, false),
    (21, 3, 'DECOR', 'DEMO-JARRON', 'Jarrón de cerámica', 'Jarrón hecho a mano en cerámica esmaltada, 30 cm de alto.', 'Barro Vivo',
        '{"style": "Mediterráneo", "material": "Cerámica"}', 'Color', '{Terracota,Blanco roto}', NULL, NULL, 24.90, 0, false),
    (22, 3, 'LIGHT', 'DEMO-LAMPARA-RATAN', 'Lámpara de pie de ratán', 'Lámpara de pie con pantalla de ratán natural trenzado.', 'Barro Vivo',
        '{"type": "Pie", "power": "E27, máx. 40 W"}', NULL, NULL, NULL, NULL, 119.00, 0, false),
    (23, 3, 'FURNI', 'DEMO-SILLA-NORDICA', 'Silla nórdica de madera', 'Silla de comedor de haya maciza con asiento curvo.', 'Taller Nómada',
        '{"material": "Haya maciza", "weight_capacity": "120 kg"}', 'Color', '{Natural,Negro}', NULL, NULL, 64.90, 0, false),
    (24, 3, 'FURNI', 'DEMO-ESTANTERIA-ROBLE', 'Estantería de roble', 'Estantería de cinco baldas en roble macizo aceitado.', 'Taller Nómada',
        '{"material": "Roble macizo", "weight_capacity": "25 kg por balda"}', 'Ancho', '{60 cm,90 cm}', NULL, NULL, 149.00, 40, false),

    -- Cumbre Sport: deportes y aire libre
    (25, 4, 'PADEL', 'DEMO-PALA-CONTROL', 'Pala de pádel de control', 'Pala redonda de fibra de vidrio, pensada para el control y la salida de bola.', 'Red Pádel',
        '{"use": "Control", "recommended_skill": "Iniciación e intermedio"}', NULL, NULL, NULL, NULL, 129.00, 0, true),
    (26, 4, 'PADEL', 'DEMO-PELOTAS-PADEL', 'Pelotas de pádel (3 uds.)', 'Bote de tres pelotas presurizadas para pista.', 'Red Pádel',
        '{"use": "Pista", "recommended_skill": "Todos los niveles"}', NULL, NULL, NULL, NULL, 6.95, 0, false),
    (27, 4, 'BIKE', 'DEMO-CASCO-BICI', 'Casco de ciclismo', 'Casco de carretera ligero con 18 ventilaciones y ajuste trasero.', 'Puerto',
        '{"type": "Carretera", "material": "EPS y policarbonato"}', 'Talla', '{S,M,L}', 'Color', '{Negro,Blanco}', 59.90, 0, false),
    (28, 4, 'CAMP', 'DEMO-TIENDA-2P', 'Tienda de campaña 2 personas', 'Tienda iglú de montaje rápido, 2,4 kg, columna de agua de 3000 mm.', 'Cumbre',
        '{"material": "Poliéster ripstop", "usage": "Tres estaciones"}', NULL, NULL, NULL, NULL, 139.00, 0, false),
    (29, 4, 'CAMP', 'DEMO-MOCHILA-30L', 'Mochila de trekking 30 L', 'Mochila con espalda ventilada, funda de lluvia y portabastones.', 'Cumbre',
        '{"material": "Nailon", "usage": "Trekking de día"}', 'Color', '{Verde,Gris}', NULL, NULL, 74.90, 0, true),
    (30, 4, 'YOGA', 'DEMO-ESTERILLA-YOGA', 'Esterilla de yoga 6 mm', 'Esterilla antideslizante de TPE con correa de transporte.', 'Prana',
        '{"material": "TPE", "length": "183 cm"}', 'Color', '{Morado,Azul,Negro}', NULL, NULL, 29.90, 0, false),
    (31, 4, 'WEIGHTS', 'DEMO-MANCUERNAS', 'Mancuernas ajustables', 'Par de mancuernas con selector de peso rápido.', 'Hierro',
        '{"weight_range": "2 a 24 kg", "material": "Acero"}', 'Peso', '{2-10 kg,2-24 kg}', NULL, NULL, 119.00, 120, false),
    (32, 4, 'TEAM', 'DEMO-BALON-FUTBOL', 'Balón de fútbol', 'Balón termosellado para césped natural y artificial.', 'Puerto',
        '{"players": "Fútbol 7 y 11", "equipment_included": "No"}', 'Talla', '{4,5}', NULL, NULL, 24.90, 0, false),

    -- Botica Bella: belleza y cuidado personal
    (33, 5, 'SKIN', 'DEMO-CREMA-HIDRATANTE', 'Crema hidratante facial', 'Crema ligera con ácido hialurónico para uso diario.', 'Botica Bella',
        '{"use": "Día y noche", "suitable_for": "Todo tipo de piel"}', 'Tamaño', '{50 ml,100 ml}', NULL, NULL, 18.90, 9, false),
    (34, 5, 'SKIN', 'DEMO-SERUM-VITC', 'Sérum de vitamina C', 'Sérum iluminador con un 15 % de vitamina C estabilizada.', 'Botica Bella',
        '{"use": "Mañana", "suitable_for": "Pieles apagadas"}', NULL, NULL, NULL, NULL, 24.50, 0, false),
    (35, 5, 'HAIR', 'DEMO-CHAMPU-SOLIDO', 'Champú sólido', 'Champú en pastilla sin envase, dura como dos botes.', 'Jabonería Clara',
        '{"hair_type": "Normal", "benefits": "Sin sulfatos"}', 'Aroma', '{Romero,Lavanda,Coco}', NULL, NULL, 9.90, 0, false),
    (36, 5, 'FRAGR', 'DEMO-PERFUME-CITRICO', 'Eau de parfum cítrica', 'Perfume fresco de bergamota, neroli y cedro.', 'Brisa',
        '{"scent": "Cítrico", "lasting_time": "6-8 h"}', 'Tamaño', '{30 ml,50 ml,100 ml}', NULL, NULL, 39.00, 15, true),
    (37, 5, 'MAKEUP', 'DEMO-BARRA-LABIOS', 'Barra de labios mate', 'Barra de labios de acabado mate y larga duración.', 'Brisa',
        '{"ingredients": "Vitamina E", "skin_type": "Todo tipo"}', 'Tono', '{Rojo cereza,Nude,Rosa palo,Coral}', NULL, NULL, 14.90, 0, false),
    (38, 5, 'BATH', 'DEMO-GEL-AVENA', 'Gel de baño de avena', 'Gel de ducha suave con avena coloidal, 500 ml.', 'Jabonería Clara',
        '{"use": "Diario", "skin_type": "Sensible"}', NULL, NULL, NULL, NULL, 6.50, 0, false),
    (39, 5, 'MAKEUP', 'DEMO-MASCARA-PESTANAS', 'Máscara de pestañas', 'Máscara de volumen resistente al agua.', 'Brisa',
        '{"ingredients": "Cera de abejas", "skin_type": "Ojos sensibles"}', NULL, NULL, NULL, NULL, 12.90, 0, false),
    (40, 5, 'BATH', 'DEMO-SALES-BANO', 'Sales de baño relajantes', 'Sales del Mar Muerto con aceites esenciales, 400 g.', 'Jabonería Clara',
        '{"use": "Baño relajante", "skin_type": "Todo tipo"}', 'Aroma', '{Eucalipto,Lavanda}', NULL, NULL, 11.90, 0, false);

-- ---------------------------------------------------------------- Productos
-- products.status: 1 activo. `variations` y `variation_titles`, como los
-- guarda el alta de producto.
INSERT INTO products (id, code, name, description, brand, status, specs, variations, variation_titles)
SELECT
    ('d3000000-0000-4000-8000-' || lpad(c.n::text, 12, '0'))::uuid,
    c.code, c.name, c.descr, c.brand, 1, c.specs,
    CASE WHEN c.d1_name IS NULL THEN NULL ELSE (
        SELECT jsonb_agg(jsonb_build_object('name', d.name, 'values',
                   (SELECT jsonb_agg(jsonb_build_object('value', v) ORDER BY i) FROM unnest(d.vals) WITH ORDINALITY AS u(v, i)))
               ORDER BY d.ord)
        FROM (VALUES (1, c.d1_name, c.d1_values), (2, c.d2_name, c.d2_values)) AS d(ord, name, vals)
        WHERE d.name IS NOT NULL) END,
    CASE WHEN c.d1_name IS NULL THEN NULL ELSE
        jsonb_build_object('titulos', to_jsonb(array_remove(ARRAY[c.d1_name, c.d2_name], NULL))) END
FROM demo_catalog c;

INSERT INTO product_seller (product_id, seller_id)
SELECT ('d3000000-0000-4000-8000-' || lpad(n::text, 12, '0'))::uuid,
       ('d2000000-0000-4000-8000-00000000000' || store)::uuid
FROM demo_catalog;

INSERT INTO category_product (category_id, product_id)
SELECT category, ('d3000000-0000-4000-8000-' || lpad(n::text, 12, '0'))::uuid
FROM demo_catalog;

-- ---------------------------------------------------------------- Variaciones
-- Una fila por combinación de valores (o una sola, sin atributos, si el
-- producto no tiene variaciones). `store_id` lo pone el trigger.
CREATE TEMP TABLE demo_variations ON COMMIT DROP AS
SELECT
    c.n,
    c.code || CASE WHEN c.d1_name IS NULL THEN '' ELSE '-' || lpad(v1.i::text, 2, '0') END
           || CASE WHEN c.d2_name IS NULL THEN '' ELSE '-' || lpad(v2.i::text, 2, '0') END AS sku,
    CASE WHEN c.d1_name IS NULL THEN NULL ELSE
        jsonb_build_array(jsonb_build_object('name', c.d1_name, 'value', v1.v))
        || CASE WHEN c.d2_name IS NULL THEN '[]'::jsonb ELSE jsonb_build_array(jsonb_build_object('name', c.d2_name, 'value', v2.v)) END
    END AS attributes,
    c.price + c.step * (coalesce(v1.i, 1) - 1) AS price,
    c.sale
FROM demo_catalog c
LEFT JOIN LATERAL unnest(c.d1_values) WITH ORDINALITY AS v1(v, i) ON true
LEFT JOIN LATERAL unnest(c.d2_values) WITH ORDINALITY AS v2(v, i) ON true;

-- Stock entre 0 y 39 según el SKU: alguna combinación sale agotada.
INSERT INTO product_variations (id, product_id, sku, attributes, status, stock)
SELECT md5(sku)::uuid,
       ('d3000000-0000-4000-8000-' || lpad(n::text, 12, '0'))::uuid,
       sku, attributes, 1,
       CASE WHEN abs(hashtext(sku)) % 40 < 3 THEN 0 ELSE abs(hashtext(sku)) % 40 END
FROM demo_variations;

-- ---------------------------------------------------------------- Precios
-- Los de rebaja entran un 20 % más caros hace 60 días y bajan hace 10 (el
-- trigger cierra el precio anterior en `price_history`).
INSERT INTO prices (variation_id, price, currency, start_date)
SELECT md5(sku)::uuid,
       CASE WHEN sale THEN round(price * 1.2, 0) - 0.10 ELSE price END,
       'EUR',
       CASE WHEN sale THEN now() - interval '60 days' ELSE now() - interval '30 days' END
FROM demo_variations;

UPDATE prices p
SET price = v.price, start_date = now() - interval '10 days'
FROM demo_variations v
WHERE v.sale AND p.variation_id = md5(v.sku)::uuid AND p.currency = 'EUR';

COMMIT;
