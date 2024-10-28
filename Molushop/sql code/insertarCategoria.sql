delete from Category;

drop table Category cascade;

select * from Category order by depth;

create table Category(
	id varchar(10) primary key,
	name text,
	parent varchar(10),
	depth integer,
    base_specs jsonb,
	foreign key (parent) references Category(id),
    is_parent boolean default false
);
INSERT INTO Category (id, name, parent, depth, base_specs) 
VALUES 
('ELEC', 'Electronics', NULL, 0, '{"specs": ["warranty", "voltage"]}'),
('HOME', 'Home & Kitchen', NULL, 0, '{"specs": ["material", "warranty"]}'),
('CLOT', 'Clothing', NULL, 0, '{"specs": ["material", "size"]}'),
('BEAU', 'Beauty & Personal Care', NULL, 0, '{"specs": ["ingredients", "skin_type"]}'),
('SPORT', 'Sports & Outdoors', NULL, 0, '{"specs": ["material", "recommended_age"]}');

INSERT INTO Category (id, name, parent, depth, base_specs) 
VALUES 
('PHONE', 'Mobile Phones', 'ELEC', 1, '{"specs": ["os", "screen_size"]}'),
('LAPTOP', 'Laptops', 'ELEC', 1, '{"specs": ["os", "storage"]}'),
('CAMERA', 'Cameras & Photography', 'ELEC', 1, '{"specs": ["megapixels", "battery"]}'),
('TV', 'Televisions', 'ELEC', 1, '{"specs": ["screen_size", "display"]}'),
('HEADP', 'Headphones', 'ELEC', 1, '{"specs": ["type", "wireless"]}');

INSERT INTO Category (id, name, parent, depth, base_specs) 
VALUES 
('KITCH', 'Kitchen Appliances', 'HOME', 1, '{"specs": ["power", "voltage"]}'),
('FURNI', 'Furniture', 'HOME', 1, '{"specs": ["material", "weight_capacity"]}'),
('DECOR', 'Home Décor', 'HOME', 1, '{"specs": ["style", "material"]}'),
('BEDD', 'Bedding', 'HOME', 1, '{"specs": ["material", "sizes"]}'),
('LIGHT', 'Lighting', 'HOME', 1, '{"specs": ["type", "power"]}');

INSERT INTO Category (id, name, parent, depth, base_specs) 
VALUES 
('MENCL', 'Men Clothing', 'CLOT', 1, '{"specs": ["size", "fit"]}'),
('WOMCL', 'Women Clothing', 'CLOT', 1, '{"specs": ["size", "fit"]}'),
('CHILCL', 'Children Clothing', 'CLOT', 1, '{"specs": ["size", "fit"]}'),
('SHOES', 'Shoes', 'CLOT', 1, '{"specs": ["size", "material"]}'),
('ACC', 'Accessories', 'CLOT', 1, '{"specs": ["material", "gender"]}');

INSERT INTO Category (id, name, parent, depth, base_specs) 
VALUES 
('SKIN', 'Skincare', 'BEAU', 1, '{"specs": ["use", "suitable_for"]}'),
('MAKEUP', 'Makeup', 'BEAU', 1, '{"specs": ["ingredients", "skin_type"]}'),
('HAIR', 'Hair Care', 'BEAU', 1, '{"specs": ["hair_type", "benefits"]}'),
('FRAGR', 'Fragrances', 'BEAU', 1, '{"specs": ["scent", "lasting_time"]}'),
('BATH', 'Bath & Body', 'BEAU', 1, '{"specs": ["use", "skin_type"]}');

INSERT INTO Category (id, name, parent, depth, base_specs) 
VALUES 
('OUTDO', 'Outdoor Gear', 'SPORT', 1, '{"specs": ["material", "warranty"]}'),
('FITNESS', 'Fitness Equipment', 'SPORT', 1, '{"specs": ["usage", "weight"]}'),
('CAMP', 'Camping & Hiking', 'SPORT', 1, '{"specs": ["material", "usage"]}'),
('BIKE', 'Cycling', 'SPORT', 1, '{"specs": ["type", "material"]}'),
('TEAM', 'Team Sports', 'SPORT', 1, '{"specs": ["players", "equipment_included"]}');

INSERT INTO Category (id, name, parent, depth, base_specs) 
VALUES 
('WEIGHTS', 'Weights & Dumbbells', 'FITNESS', 2, '{"specs": ["weight_range", "material"]}'),
('CARDIO', 'Cardio Equipment', 'FITNESS', 2, '{"specs": ["type", "monitor_included"]}'),
('YOGA', 'Yoga & Pilates', 'FITNESS', 2, '{"specs": ["material", "length"]}'),
('STRENGTH', 'Strength Training', 'FITNESS', 2, '{"specs": ["material", "weight_capacity"]}'),
('ACCESS', 'Fitness Accessories', 'FITNESS', 2, '{"specs": ["material", "suitable_for"]}');

INSERT INTO Category (id, name, parent, depth, base_specs) 
VALUES 
('PADEL', 'Padel', 'SPORT', 1, '{"specs": ["use", "recommended_skill"]}'),
('BALLPAD', 'Padel Balls', 'PADEL', 2, '{"specs": ["material", "size"]}');



WITH RECURSIVE Ancestors AS (
    -- Selecciona la categoría inicial
    SELECT id, name, parent, depth
    FROM Category
    WHERE id = 'ACCESS'

    UNION ALL
    
    -- Selecciona los ancestros recursivamente
    SELECT c.id, c.name, c.parent, c.depth
    FROM Category c
    INNER JOIN Ancestors a ON c.id = a.parent
)
SELECT * FROM Ancestors order by depth;

SELECT * FROM Category;

-- Podriamos poner un flag para saber si es hoja o no
------------------------------------------
-- Añadir la columna is_parent
ALTER TABLE Category ADD COLUMN is_parent BOOLEAN DEFAULT FALSE;

-- Crear la función para actualizar is_parent
CREATE OR REPLACE FUNCTION update_is_parent() RETURNS TRIGGER AS $$
BEGIN
    -- Si se inserta una nueva categoría, actualizar el padre
    IF TG_OP = 'INSERT' THEN
        UPDATE Category SET is_parent = TRUE WHERE id = NEW.parent;
    END IF;

    -- Si se actualiza una categoría, actualizar el padre antiguo y el nuevo
    IF TG_OP = 'UPDATE' THEN
        IF OLD.parent IS DISTINCT FROM NEW.parent THEN
            -- Si el padre antiguo ya no tiene hijos, actualizar is_parent a FALSE
            UPDATE Category SET is_parent = FALSE WHERE id = OLD.parent AND NOT EXISTS (
                SELECT 1 FROM Category WHERE parent = OLD.parent
            );
            -- Actualizar el nuevo padre a TRUE
            UPDATE Category SET is_parent = TRUE WHERE id = NEW.parent;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Crear el trigger para llamar a la función en INSERT y UPDATE
CREATE TRIGGER update_is_parent_trigger
AFTER INSERT OR UPDATE ON Category
FOR EACH ROW EXECUTE FUNCTION update_is_parent();