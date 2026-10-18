-- Datos de DESARROLLO de Molushop. NUNCA en producción.
--
-- Uso, sobre una base de datos recién creada (migraciones + init):
--     diesel database reset                       # OJO: borra la base de datos de DATABASE_URL
--     psql -v ON_ERROR_STOP=1 -f seeds/init.sql "$DATABASE_URL"
--     psql -v ON_ERROR_STOP=1 -f seeds/dev.sql  "$DATABASE_URL"
-- (o `seeds/reset_dev.sh`, que hace las tres cosas). No es repetible: los ids
-- son fijos y la segunda vez choca con las claves primarias; para volver a
-- empezar, reset otra vez.
--
-- Usuarios (todos con la contraseña de desarrollo `Molushop-dev1`, que cumple
-- las reglas de la app; hash argon2id con los parámetros por defecto de la app):
--   admin_dev            admin (fila en `admins`)
--   vendedor_dev         miembro de "Moda Molushop" y "Casa Molushop" (activas:
--                        sirve para el selector de tienda) y de "Tienda Retirada" (inactiva)
--   vendedora_pendiente  solo tiene "Tienda Pendiente" (pendiente de aprobar)
--   comprador_dev        pedidos con líneas en todos los estados, carrito con
--                        líneas que ya no se venden, dirección guardada
--   comprador2_dev       un pedido entregado y una opinión
--
-- Productos (ids 33333333-...-0000000000NN) y lo que prueban:
--   01 Camiseta básica          variaciones Color x Talla, una agotada (Blanco/M)
--   02 Auriculares inalámbricos sin variaciones (variación única sin atributos)
--   03 Lámpara de mesa          una variación SIN precio (Negra): "No disponible"
--   04 Zapatillas running       todas agotadas: botón "Agotado"
--   05 Bolso (borrador)         status 0: no sale en la tienda
--   06 Gorra (eliminado)        status 3: ni tienda ni panel
--   07 Taza de cerámica         sin variaciones, de la otra tienda (Casa Molushop)
--   08 Cojín decorativo         de una tienda INACTIVA: no se vende
--   09 Bufanda (inactivo)       status 2: no sale en la tienda
--
-- Las imágenes apuntan a ficheros que ya están en el bucket S3 de desarrollo.

BEGIN;

-- ---------------------------------------------------------------- Usuarios
INSERT INTO base_user (id, username, name, lastname, email, password, active) VALUES
    ('11111111-0000-4000-8000-000000000001', 'admin_dev',           'Admin',     'Dev',       'admin@dev.local',               '$argon2id$v=19$m=19456,t=2,p=1$FPFBTJ8GSMBbd/ALMKRKgw$Fm6+Zxh3IWC3i8nV4vPYigE9KuwJbIwZBR8Du1G8Ap0', true),
    ('11111111-0000-4000-8000-000000000002', 'vendedor_dev',        'Vendedor',  'Dev',       'vendedor@dev.local',            '$argon2id$v=19$m=19456,t=2,p=1$FPFBTJ8GSMBbd/ALMKRKgw$Fm6+Zxh3IWC3i8nV4vPYigE9KuwJbIwZBR8Du1G8Ap0', true),
    ('11111111-0000-4000-8000-000000000003', 'vendedora_pendiente', 'Vendedora', 'Pendiente', 'vendedora.pendiente@dev.local', '$argon2id$v=19$m=19456,t=2,p=1$FPFBTJ8GSMBbd/ALMKRKgw$Fm6+Zxh3IWC3i8nV4vPYigE9KuwJbIwZBR8Du1G8Ap0', true),
    ('11111111-0000-4000-8000-000000000004', 'comprador_dev',       'Comprador', 'Dev',       'comprador@dev.local',           '$argon2id$v=19$m=19456,t=2,p=1$FPFBTJ8GSMBbd/ALMKRKgw$Fm6+Zxh3IWC3i8nV4vPYigE9KuwJbIwZBR8Du1G8Ap0', true),
    ('11111111-0000-4000-8000-000000000005', 'comprador2_dev',      'Compradora','Dos',       'comprador2@dev.local',          '$argon2id$v=19$m=19456,t=2,p=1$FPFBTJ8GSMBbd/ALMKRKgw$Fm6+Zxh3IWC3i8nV4vPYigE9KuwJbIwZBR8Du1G8Ap0', true);

INSERT INTO admins (id) VALUES ('11111111-0000-4000-8000-000000000001');

-- ---------------------------------------------------------------- Tiendas
-- seller.status: 0 pendiente, 1 activa, 2 inactiva, 3 rechazada.
INSERT INTO seller (id, rating, store_name, status) VALUES
    ('22222222-0000-4000-8000-000000000001', 5, 'Moda Molushop',    1),
    ('22222222-0000-4000-8000-000000000002', 5, 'Casa Molushop',    1),
    ('22222222-0000-4000-8000-000000000003', 5, 'Tienda Retirada',  2),
    ('22222222-0000-4000-8000-000000000004', 5, 'Tienda Pendiente', 0);

INSERT INTO store_members (store_id, user_id) VALUES
    ('22222222-0000-4000-8000-000000000001', '11111111-0000-4000-8000-000000000002'),
    ('22222222-0000-4000-8000-000000000002', '11111111-0000-4000-8000-000000000002'),
    ('22222222-0000-4000-8000-000000000003', '11111111-0000-4000-8000-000000000002'),
    ('22222222-0000-4000-8000-000000000004', '11111111-0000-4000-8000-000000000003');

-- ---------------------------------------------------------------- Productos
-- products.status: 0 borrador, 1 activo, 2 inactivo, 3 eliminado.
INSERT INTO products (id, code, name, description, brand, status, specs, variations, variation_titles) VALUES
    ('33333333-0000-4000-8000-000000000001', 'DEV-CAMISETA', 'Camiseta básica', 'Camiseta de algodón de manga corta.', 'Molushop Basics', 1,
        '{"material": "Algodón"}',
        '[{"name": "Color", "values": [{"value": "Blanco"}, {"value": "Negro"}]}, {"name": "Talla", "values": [{"value": "S"}, {"value": "M"}]}]',
        '{"titulos": ["Color", "Talla"]}'),
    ('33333333-0000-4000-8000-000000000002', 'DEV-AURIS', 'Auriculares inalámbricos', 'Auriculares bluetooth con estuche de carga.', 'SoundDev', 1,
        '{"type": "In-ear", "wireless": "Sí"}', NULL, NULL),
    ('33333333-0000-4000-8000-000000000003', 'DEV-LAMPARA', 'Lámpara de mesa', 'Lámpara LED regulable.', 'Luz Casa', 1,
        '{"type": "LED", "power": "8 W"}',
        '[{"name": "Color", "values": [{"value": "Blanca"}, {"value": "Negra"}]}]',
        '{"titulos": ["Color"]}'),
    ('33333333-0000-4000-8000-000000000004', 'DEV-ZAPAS', 'Zapatillas running', 'Zapatillas ligeras para correr.', 'RunDev', 1,
        '{"material": "Malla"}',
        '[{"name": "Talla", "values": [{"value": "42"}, {"value": "43"}]}]',
        '{"titulos": ["Talla"]}'),
    ('33333333-0000-4000-8000-000000000005', 'DEV-BOLSO', 'Bolso (borrador)', 'Todavía sin publicar.', 'Molushop Basics', 0,
        '{"material": "Piel"}', NULL, NULL),
    ('33333333-0000-4000-8000-000000000006', 'DEV-GORRA', 'Gorra (eliminado)', 'Eliminado desde el panel.', 'Molushop Basics', 3,
        '{"material": "Algodón"}', NULL, NULL),
    ('33333333-0000-4000-8000-000000000007', 'DEV-TAZA', 'Taza de cerámica', 'Taza de 350 ml apta para lavavajillas.', 'Luz Casa', 1,
        '{"style": "Nórdico", "material": "Cerámica"}', NULL, NULL),
    ('33333333-0000-4000-8000-000000000008', 'DEV-COJIN', 'Cojín decorativo', 'De una tienda desactivada.', 'Luz Casa', 1,
        '{"style": "Boho", "material": "Algodón"}', NULL, NULL),
    ('33333333-0000-4000-8000-000000000009', 'DEV-BUFANDA', 'Bufanda (inactivo)', 'Marcado como inactivo en el panel.', 'Molushop Basics', 2,
        '{"material": "Lana"}', NULL, NULL);

INSERT INTO product_seller (product_id, seller_id) VALUES
    ('33333333-0000-4000-8000-000000000001', '22222222-0000-4000-8000-000000000001'),
    ('33333333-0000-4000-8000-000000000002', '22222222-0000-4000-8000-000000000001'),
    ('33333333-0000-4000-8000-000000000003', '22222222-0000-4000-8000-000000000002'),
    ('33333333-0000-4000-8000-000000000004', '22222222-0000-4000-8000-000000000001'),
    ('33333333-0000-4000-8000-000000000005', '22222222-0000-4000-8000-000000000001'),
    ('33333333-0000-4000-8000-000000000006', '22222222-0000-4000-8000-000000000001'),
    ('33333333-0000-4000-8000-000000000007', '22222222-0000-4000-8000-000000000002'),
    ('33333333-0000-4000-8000-000000000008', '22222222-0000-4000-8000-000000000003'),
    ('33333333-0000-4000-8000-000000000009', '22222222-0000-4000-8000-000000000001');

INSERT INTO category_product (category_id, product_id) VALUES
    ('MENCL', '33333333-0000-4000-8000-000000000001'),
    ('HEADP', '33333333-0000-4000-8000-000000000002'),
    ('LIGHT', '33333333-0000-4000-8000-000000000003'),
    ('MENCL', '33333333-0000-4000-8000-000000000004'),
    ('WOMCL', '33333333-0000-4000-8000-000000000005'),
    ('MENCL', '33333333-0000-4000-8000-000000000006'),
    ('DECOR', '33333333-0000-4000-8000-000000000007'),
    ('DECOR', '33333333-0000-4000-8000-000000000008'),
    ('MENCL', '33333333-0000-4000-8000-000000000009');

INSERT INTO images_product (id, product_id, image_url, is_main, display_order) VALUES
    ('aaaaaaaa-0000-4000-8000-000000000001', '33333333-0000-4000-8000-000000000001', 'https://tfg-vik.s3.eu-north-1.amazonaws.com/1765746501azul.jpg', true, 1),
    ('aaaaaaaa-0000-4000-8000-000000000002', '33333333-0000-4000-8000-000000000002', 'https://tfg-vik.s3.eu-north-1.amazonaws.com/1765748966tv1.jpg', true, 1),
    ('aaaaaaaa-0000-4000-8000-000000000003', '33333333-0000-4000-8000-000000000003', 'https://tfg-vik.s3.eu-north-1.amazonaws.com/1766345075amarillo.jpg', true, 1),
    ('aaaaaaaa-0000-4000-8000-000000000004', '33333333-0000-4000-8000-000000000004', 'https://tfg-vik.s3.eu-north-1.amazonaws.com/1767296857zapatos-premium.jpg', true, 1),
    ('aaaaaaaa-0000-4000-8000-000000000007', '33333333-0000-4000-8000-000000000007', 'https://tfg-vik.s3.eu-north-1.amazonaws.com/1767296783Tienda.jpg', true, 1),
    ('aaaaaaaa-0000-4000-8000-000000000008', '33333333-0000-4000-8000-000000000008', 'https://tfg-vik.s3.eu-north-1.amazonaws.com/1769368300Cortinas1.jpg', true, 1);

-- Variaciones. product_variations.status: 1 activa. `attributes` NULL = la
-- variación única de un producto sin variaciones.
INSERT INTO product_variations (id, product_id, sku, attributes, status, stock) VALUES
    ('44444444-0000-4000-8000-000000000101', '33333333-0000-4000-8000-000000000001', 'CAM-BL-S', '[{"name": "Color", "value": "Blanco"}, {"name": "Talla", "value": "S"}]', 1, 20),
    ('44444444-0000-4000-8000-000000000102', '33333333-0000-4000-8000-000000000001', 'CAM-BL-M', '[{"name": "Color", "value": "Blanco"}, {"name": "Talla", "value": "M"}]', 1, 0),
    ('44444444-0000-4000-8000-000000000103', '33333333-0000-4000-8000-000000000001', 'CAM-NE-S', '[{"name": "Color", "value": "Negro"}, {"name": "Talla", "value": "S"}]', 1, 5),
    ('44444444-0000-4000-8000-000000000104', '33333333-0000-4000-8000-000000000001', 'CAM-NE-M', '[{"name": "Color", "value": "Negro"}, {"name": "Talla", "value": "M"}]', 1, 12),
    ('44444444-0000-4000-8000-000000000201', '33333333-0000-4000-8000-000000000002', 'AURI-01',  NULL, 1, 25),
    ('44444444-0000-4000-8000-000000000301', '33333333-0000-4000-8000-000000000003', 'LAMP-BL',  '[{"name": "Color", "value": "Blanca"}]', 1, 8),
    ('44444444-0000-4000-8000-000000000302', '33333333-0000-4000-8000-000000000003', 'LAMP-NE',  '[{"name": "Color", "value": "Negra"}]', 1, 3),
    ('44444444-0000-4000-8000-000000000401', '33333333-0000-4000-8000-000000000004', 'ZAP-42',   '[{"name": "Talla", "value": "42"}]', 1, 0),
    ('44444444-0000-4000-8000-000000000402', '33333333-0000-4000-8000-000000000004', 'ZAP-43',   '[{"name": "Talla", "value": "43"}]', 1, 0),
    ('44444444-0000-4000-8000-000000000501', '33333333-0000-4000-8000-000000000005', 'BOLSO-01', NULL, 1, 4),
    ('44444444-0000-4000-8000-000000000601', '33333333-0000-4000-8000-000000000006', 'GORRA-01', NULL, 1, 10),
    ('44444444-0000-4000-8000-000000000701', '33333333-0000-4000-8000-000000000007', 'TAZA-01',  NULL, 1, 40),
    ('44444444-0000-4000-8000-000000000801', '33333333-0000-4000-8000-000000000008', 'COJIN-01', NULL, 1, 10),
    ('44444444-0000-4000-8000-000000000901', '33333333-0000-4000-8000-000000000009', 'BUF-01',   NULL, 1, 6);

-- Precios (el trigger escribe `price_history`). La Lámpara Negra (302) no tiene
-- precio a propósito.
INSERT INTO prices (variation_id, price, currency) VALUES
    ('44444444-0000-4000-8000-000000000101', 14.95, 'EUR'),
    ('44444444-0000-4000-8000-000000000102', 14.95, 'EUR'),
    ('44444444-0000-4000-8000-000000000103', 14.95, 'EUR'),
    ('44444444-0000-4000-8000-000000000104', 16.95, 'EUR'),
    ('44444444-0000-4000-8000-000000000201', 59.90, 'EUR'),
    ('44444444-0000-4000-8000-000000000301', 39.00, 'EUR'),
    ('44444444-0000-4000-8000-000000000401', 79.99, 'EUR'),
    ('44444444-0000-4000-8000-000000000402', 79.99, 'EUR'),
    ('44444444-0000-4000-8000-000000000501', 45.00, 'EUR'),
    ('44444444-0000-4000-8000-000000000601', 12.00, 'EUR'),
    ('44444444-0000-4000-8000-000000000701',  9.50, 'EUR'),
    ('44444444-0000-4000-8000-000000000801', 19.90, 'EUR'),
    ('44444444-0000-4000-8000-000000000901', 22.00, 'EUR');

-- ---------------------------------------------------------------- Pedidos
INSERT INTO customer_address (id, customer_id, street1, street2, postal_code, city, province) VALUES
    ('99999999-0000-4000-8000-000000000001', '11111111-0000-4000-8000-000000000004', 'Calle Mayor 1', '2º B', '28013', 'Madrid', 'Madrid'),
    ('99999999-0000-4000-8000-000000000002', '11111111-0000-4000-8000-000000000005', 'Avenida del Puerto 20', NULL, '46021', 'Valencia', 'Valencia');

-- orders.status es el resumen de sus líneas (el estado de la más atrasada de
-- las no canceladas): 1 confirmado, 2 cancelado, 3 enviado, 4 entregado.
-- Los totales son lo que se pidió; lo cancelado no se descuenta aquí.
INSERT INTO orders (id, user_id, status, currency, subtotal, total_amount, shipping_address_id,
                    ship_street1, ship_street2, ship_postal_code, ship_city, ship_province, created_at) VALUES
    -- Una línea entregada y otra enviada -> enviado
    ('55555555-0000-4000-8000-000000000001', '11111111-0000-4000-8000-000000000004', 3, 'EUR', 89.80, 89.80,
        '99999999-0000-4000-8000-000000000001', 'Calle Mayor 1', '2º B', '28013', 'Madrid', 'Madrid', now() - interval '10 days'),
    -- Una confirmada y otra cancelada -> confirmado; el comprador ve 28,50 €
    ('55555555-0000-4000-8000-000000000002', '11111111-0000-4000-8000-000000000004', 1, 'EUR', 43.45, 43.45,
        '99999999-0000-4000-8000-000000000001', 'Calle Mayor 1', '2º B', '28013', 'Madrid', 'Madrid', now() - interval '2 days'),
    -- Entregado: comprador2 puede opinar sobre la camiseta
    ('55555555-0000-4000-8000-000000000003', '11111111-0000-4000-8000-000000000005', 4, 'EUR', 16.95, 16.95,
        '99999999-0000-4000-8000-000000000002', 'Avenida del Puerto 20', NULL, '46021', 'Valencia', 'Valencia', now() - interval '20 days');

INSERT INTO order_items (id, order_id, product_var_id, product_name, variation_label, sku, image_url, store_name,
                         unit_price, currency, quantity, line_total, store_id, status) VALUES
    ('66666666-0000-4000-8000-000000000001', '55555555-0000-4000-8000-000000000001', '44444444-0000-4000-8000-000000000101',
        'Camiseta básica', 'Blanco / S', 'CAM-BL-S', 'https://tfg-vik.s3.eu-north-1.amazonaws.com/1765746501azul.jpg', 'Moda Molushop',
        14.95, 'EUR', 2, 29.90, '22222222-0000-4000-8000-000000000001', 4),
    ('66666666-0000-4000-8000-000000000002', '55555555-0000-4000-8000-000000000001', '44444444-0000-4000-8000-000000000201',
        'Auriculares inalámbricos', NULL, 'AURI-01', 'https://tfg-vik.s3.eu-north-1.amazonaws.com/1765748966tv1.jpg', 'Moda Molushop',
        59.90, 'EUR', 1, 59.90, '22222222-0000-4000-8000-000000000001', 3),
    ('66666666-0000-4000-8000-000000000003', '55555555-0000-4000-8000-000000000002', '44444444-0000-4000-8000-000000000701',
        'Taza de cerámica', NULL, 'TAZA-01', 'https://tfg-vik.s3.eu-north-1.amazonaws.com/1767296783Tienda.jpg', 'Casa Molushop',
        9.50, 'EUR', 3, 28.50, '22222222-0000-4000-8000-000000000002', 1),
    ('66666666-0000-4000-8000-000000000004', '55555555-0000-4000-8000-000000000002', '44444444-0000-4000-8000-000000000103',
        'Camiseta básica', 'Negro / S', 'CAM-NE-S', 'https://tfg-vik.s3.eu-north-1.amazonaws.com/1765746501azul.jpg', 'Moda Molushop',
        14.95, 'EUR', 1, 14.95, '22222222-0000-4000-8000-000000000001', 2),
    ('66666666-0000-4000-8000-000000000005', '55555555-0000-4000-8000-000000000003', '44444444-0000-4000-8000-000000000104',
        'Camiseta básica', 'Negro / M', 'CAM-NE-M', 'https://tfg-vik.s3.eu-north-1.amazonaws.com/1765746501azul.jpg', 'Moda Molushop',
        16.95, 'EUR', 1, 16.95, '22222222-0000-4000-8000-000000000001', 4);

-- ---------------------------------------------------------------- Opiniones
INSERT INTO ratings (id, product_id, user_id, rating_value, comment) VALUES
    ('bbbbbbbb-0000-4000-8000-000000000001', '33333333-0000-4000-8000-000000000001', '11111111-0000-4000-8000-000000000004', 5,
        'Muy buena calidad, la talla es la esperada.'),
    ('bbbbbbbb-0000-4000-8000-000000000002', '33333333-0000-4000-8000-000000000001', '11111111-0000-4000-8000-000000000005', 3,
        NULL);

-- ---------------------------------------------------------------- Carrito
-- Carrito activo de comprador_dev: una línea buena y cuatro que ya no se
-- venden (salen marcadas "No disponible" y no cuentan en el total).
INSERT INTO carts (id, user_id, status) VALUES
    ('77777777-0000-4000-8000-000000000001', '11111111-0000-4000-8000-000000000004', 1);

INSERT INTO cart_products (id, cart_id, product_var_id, quantity) VALUES
    ('88888888-0000-4000-8000-000000000001', '77777777-0000-4000-8000-000000000001', '44444444-0000-4000-8000-000000000201', 1), -- auriculares: bien
    ('88888888-0000-4000-8000-000000000002', '77777777-0000-4000-8000-000000000001', '44444444-0000-4000-8000-000000000302', 1), -- lámpara negra: sin precio
    ('88888888-0000-4000-8000-000000000003', '77777777-0000-4000-8000-000000000001', '44444444-0000-4000-8000-000000000801', 1), -- cojín: tienda inactiva
    ('88888888-0000-4000-8000-000000000004', '77777777-0000-4000-8000-000000000001', '44444444-0000-4000-8000-000000000601', 1), -- gorra: eliminada
    ('88888888-0000-4000-8000-000000000005', '77777777-0000-4000-8000-000000000001', '44444444-0000-4000-8000-000000000901', 1); -- bufanda: inactiva

COMMIT;
