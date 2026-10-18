-- Datos base de Molushop: lo que necesita CUALQUIER entorno (desarrollo y
-- producción) después de las migraciones. Hoy solo son las categorías: sin
-- ellas no se puede crear ningún producto.
--
-- Uso (después de `diesel migration run` sobre una base de datos vacía):
--     psql -v ON_ERROR_STOP=1 -f seeds/init.sql "$DATABASE_URL"
--
-- Se puede ejecutar más de una vez: ON CONFLICT no duplica nada.
-- Sacado de la base de datos de desarrollo el 2026-10-18 (36 categorías, en
-- inglés). Van ordenadas por profundidad: cada padre antes que sus hijos.
-- `depth` e `is_parent` no se escriben: los calculan los triggers de `category`.

BEGIN;

INSERT INTO category (id, name, parent, base_specs) VALUES
    ('BEAU', 'Beauty & Personal Care', NULL, '{"specs": ["ingredients", "skin_type"]}'),
    ('CLOT', 'Clothing', NULL, '{"specs": ["material", "size"]}'),
    ('ELEC', 'Electronics', NULL, '{"specs": ["warranty", "voltage"]}'),
    ('HOME', 'Home & Kitchen', NULL, '{"specs": ["material", "warranty"]}'),
    ('SPORT', 'Sports & Outdoors', NULL, '{"specs": ["material", "recommended_age"]}'),
    ('BATH', 'Bath & Body', 'BEAU', '{"specs": ["use", "skin_type"]}'),
    ('FRAGR', 'Fragrances', 'BEAU', '{"specs": ["scent", "lasting_time"]}'),
    ('HAIR', 'Hair Care', 'BEAU', '{"specs": ["hair_type", "benefits"]}'),
    ('MAKEUP', 'Makeup', 'BEAU', '{"specs": ["ingredients", "skin_type"]}'),
    ('SKIN', 'Skincare', 'BEAU', '{"specs": ["use", "suitable_for"]}'),
    ('ACC', 'Accessories', 'CLOT', '{"specs": ["material", "gender"]}'),
    ('CHILCL', 'Children Clothing', 'CLOT', '{"specs": ["size", "fit"]}'),
    ('MENCL', 'Men Clothing', 'CLOT', '{"specs": ["size", "fit"]}'),
    ('SHOES', 'Shoes', 'CLOT', '{"specs": ["size", "material"]}'),
    ('WOMCL', 'Women Clothing', 'CLOT', '{"specs": ["size", "fit"]}'),
    ('CAMERA', 'Cameras & Photography', 'ELEC', '{"specs": ["megapixels", "battery"]}'),
    ('HEADP', 'Headphones', 'ELEC', '{"specs": ["type", "wireless"]}'),
    ('LAPTOP', 'Laptops', 'ELEC', '{"specs": ["os", "storage"]}'),
    ('PHONE', 'Mobile Phones', 'ELEC', '{"specs": ["os", "screen_size"]}'),
    ('TV', 'Televisions', 'ELEC', '{"specs": ["screen_size", "display"]}'),
    ('BEDD', 'Bedding', 'HOME', '{"specs": ["material", "sizes"]}'),
    ('DECOR', 'Home Décor', 'HOME', '{"specs": ["style", "material"]}'),
    ('FURNI', 'Furniture', 'HOME', '{"specs": ["material", "weight_capacity"]}'),
    ('KITCH', 'Kitchen Appliances', 'HOME', '{"specs": ["power", "voltage"]}'),
    ('LIGHT', 'Lighting', 'HOME', '{"specs": ["type", "power"]}'),
    ('BIKE', 'Cycling', 'SPORT', '{"specs": ["type", "material"]}'),
    ('CAMP', 'Camping & Hiking', 'SPORT', '{"specs": ["material", "usage"]}'),
    ('FITNESS', 'Fitness Equipment', 'SPORT', '{"specs": ["usage", "weight"]}'),
    ('OUTDO', 'Outdoor Gear', 'SPORT', '{"specs": ["material", "warranty"]}'),
    ('PADEL', 'Padel', 'SPORT', '{"specs": ["use", "recommended_skill"]}'),
    ('TEAM', 'Team Sports', 'SPORT', '{"specs": ["players", "equipment_included"]}'),
    ('ACCESS', 'Fitness Accessories', 'FITNESS', '{"specs": ["material", "suitable_for"]}'),
    ('CARDIO', 'Cardio Equipment', 'FITNESS', '{"specs": ["type", "monitor_included"]}'),
    ('STRENGTH', 'Strength Training', 'FITNESS', '{"specs": ["material", "weight_capacity"]}'),
    ('WEIGHTS', 'Weights & Dumbbells', 'FITNESS', '{"specs": ["weight_range", "material"]}'),
    ('YOGA', 'Yoga & Pilates', 'FITNESS', '{"specs": ["material", "length"]}')
ON CONFLICT (id) DO NOTHING;

COMMIT;
