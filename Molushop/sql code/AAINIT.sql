--primero se crean las categorias
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
('PADEL', 'Padel', 'SPORT', 1, '{"specs": ["use", "recommended_skill"]}');

--Lo Siguiente son los usuarios de prueba?
insert into base_user  (id,username, name, lastname, email, password, birthdate) values ('2064d62a-4978-4fe7-bef2-7690ff09bdc8','Moluxo', 'Vik', 'Bernardo', 'moluxxxo@email.com','123456','1990-05-15');
insert into seller (id, rating, store_name) values ('2064d62a-4978-4fe7-bef2-7690ff09bdc8', 5, 'Moluxo Store');

