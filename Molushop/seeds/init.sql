-- Datos base de Molushop: lo que necesita CUALQUIER entorno (desarrollo y
-- producción) después de las migraciones. Hoy solo son las categorías: sin
-- ellas no se puede crear ningún producto.
--
-- Uso (después de `diesel migration run` sobre una base de datos vacía):
--     psql -v ON_ERROR_STOP=1 -f seeds/init.sql "$DATABASE_URL"
--
-- Se puede ejecutar más de una vez: ON CONFLICT no duplica nada.
-- Sacado de la base de datos de desarrollo el 2026-10-18 (36 categorías);
-- nombres en español desde el 2026-10-26 (migración category_names_es). Los
-- id no cambian: van en las URL (/search?category=CLOT). Van ordenadas por
-- profundidad: cada padre antes que sus hijos.
-- `depth` e `is_parent` no se escriben: los calculan los triggers de `category`.

BEGIN;

INSERT INTO category (id, name, parent, base_specs) VALUES
    ('BEAU', 'Belleza y cuidado personal', NULL, '{"specs": ["ingredients", "skin_type"]}'),
    ('CLOT', 'Ropa', NULL, '{"specs": ["material", "size"]}'),
    ('ELEC', 'Electrónica', NULL, '{"specs": ["warranty", "voltage"]}'),
    ('HOME', 'Hogar y cocina', NULL, '{"specs": ["material", "warranty"]}'),
    ('SPORT', 'Deportes y aire libre', NULL, '{"specs": ["material", "recommended_age"]}'),
    ('BATH', 'Baño y cuerpo', 'BEAU', '{"specs": ["use", "skin_type"]}'),
    ('FRAGR', 'Perfumes', 'BEAU', '{"specs": ["scent", "lasting_time"]}'),
    ('HAIR', 'Cuidado del cabello', 'BEAU', '{"specs": ["hair_type", "benefits"]}'),
    ('MAKEUP', 'Maquillaje', 'BEAU', '{"specs": ["ingredients", "skin_type"]}'),
    ('SKIN', 'Cuidado de la piel', 'BEAU', '{"specs": ["use", "suitable_for"]}'),
    ('ACC', 'Complementos', 'CLOT', '{"specs": ["material", "gender"]}'),
    ('CHILCL', 'Ropa infantil', 'CLOT', '{"specs": ["size", "fit"]}'),
    ('MENCL', 'Ropa de hombre', 'CLOT', '{"specs": ["size", "fit"]}'),
    ('SHOES', 'Calzado', 'CLOT', '{"specs": ["size", "material"]}'),
    ('WOMCL', 'Ropa de mujer', 'CLOT', '{"specs": ["size", "fit"]}'),
    ('CAMERA', 'Cámaras y fotografía', 'ELEC', '{"specs": ["megapixels", "battery"]}'),
    ('HEADP', 'Auriculares', 'ELEC', '{"specs": ["type", "wireless"]}'),
    ('LAPTOP', 'Portátiles', 'ELEC', '{"specs": ["os", "storage"]}'),
    ('PHONE', 'Móviles', 'ELEC', '{"specs": ["os", "screen_size"]}'),
    ('TV', 'Televisores', 'ELEC', '{"specs": ["screen_size", "display"]}'),
    ('BEDD', 'Ropa de cama', 'HOME', '{"specs": ["material", "sizes"]}'),
    ('DECOR', 'Decoración', 'HOME', '{"specs": ["style", "material"]}'),
    ('FURNI', 'Muebles', 'HOME', '{"specs": ["material", "weight_capacity"]}'),
    ('KITCH', 'Electrodomésticos de cocina', 'HOME', '{"specs": ["power", "voltage"]}'),
    ('LIGHT', 'Iluminación', 'HOME', '{"specs": ["type", "power"]}'),
    ('BIKE', 'Ciclismo', 'SPORT', '{"specs": ["type", "material"]}'),
    ('CAMP', 'Camping y senderismo', 'SPORT', '{"specs": ["material", "usage"]}'),
    ('FITNESS', 'Fitness', 'SPORT', '{"specs": ["usage", "weight"]}'),
    ('OUTDO', 'Equipo de exterior', 'SPORT', '{"specs": ["material", "warranty"]}'),
    ('PADEL', 'Pádel', 'SPORT', '{"specs": ["use", "recommended_skill"]}'),
    ('TEAM', 'Deportes de equipo', 'SPORT', '{"specs": ["players", "equipment_included"]}'),
    ('ACCESS', 'Accesorios de fitness', 'FITNESS', '{"specs": ["material", "suitable_for"]}'),
    ('CARDIO', 'Cardio', 'FITNESS', '{"specs": ["type", "monitor_included"]}'),
    ('STRENGTH', 'Entrenamiento de fuerza', 'FITNESS', '{"specs": ["material", "weight_capacity"]}'),
    ('WEIGHTS', 'Pesas y mancuernas', 'FITNESS', '{"specs": ["weight_range", "material"]}'),
    ('YOGA', 'Yoga y pilates', 'FITNESS', '{"specs": ["material", "length"]}')
ON CONFLICT (id) DO NOTHING;

COMMIT;
