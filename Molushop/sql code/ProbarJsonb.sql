CREATE TABLE Products (
    id UUID PRIMARY KEY default gen_random_uuid(),
    code VARCHAR(100) unique,
    name VARCHAR(100),
    description TEXT,
    brand VARCHAR(100),
    specs JSONB, -- Especificaciones del producto --> tendran una plantilla dependiendo de la categoria
    variations JSONB, --> LAS VARIACIONES DEL PRODUCTO 
    images JSONB  --> thumbnail, otras imagenes
);
select * from products;
insert into products (code, name, description, brand, specs, variations, images) values 
    ('LAP001', 'Laptop', 'Laptop Dell', 'Dell', '{"ram": "8GB", "procesador": "Intel i5"}', '[{"sku": "LAP001-1", "stock": 10}, {"sku": "LAP001-2", "stock": 5}]', '["imagen1.jpg", "imagen2.jpg"]');


CREATE TABLE Variations (
    id TEXT PRIMARY KEY,
    product_id TEXT,
    identifiers JSONB,
    sku TEXT,
    variations JSONB,
    stock INT,
    FOREIGN KEY (product_id) REFERENCES products(id),
    images JSONB -- imagenes para la variacion
);

create table Category(
	id varchar(10) primary key,
	name text,
	parent varchar(10),
	depth integer,
    base_specs jsonb,
	foreign key (parent) references Category(id),
    is_parent boolean default false
);

drop table products cascade;
drop table variations cascade;

--Falta ver como inserto imagenes en la base de datos

drop table products cascade;
select * from productosX;

INSERT INTO productosX (id,nombre, detalles) VALUES 
    ('LAP011','Laptop', '{"marca": "Dell", "especificaciones": {"ram": "8GB", "procesador": "Intel i5"}}'),
    ('SMART21','Smartphone', '{"marca": "Samsung", "especificaciones": {"ram": "4GB", "procesador": "Exynos"}}');

SELECT nombre, detalles->'especificaciones'->>'ram' AS ram
FROM productosX
WHERE detalles->>'marca' = 'Dell';

INSERT INTO productosX (id,nombre, detalles) VALUES ('CAM001','Camiseta', '{"tallas": ["XS", "M", "L"], "color": "azul"}');

-- Actualizar el color de la camiseta a rojo
UPDATE productosX SET detalles = jsonb_set(detalles, '{color}', '"rojo"') WHERE nombre = 'Camiseta';
-- Agrergar una talla a la camiseta
UPDATE productosX SET detalles = jsonb_set(detalles, '{tallas}', detalles->'tallas' || '["XL"]') WHERE nombre = 'Camiseta';
-- Agregar un nuevo campo a JSONB
UPDATE productosX SET detalles = detalles || '{"material": "algodón"}' WHERE nombre = 'Camiseta';

