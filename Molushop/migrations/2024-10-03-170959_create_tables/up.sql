-- Here goes the code to create the tables
-- Tabla 'User'
CREATE TABLE base_user (
    id UUID PRIMARY KEY,  -- Llave primaria
    name VARCHAR(255) NOT NULL,  -- Nombre del usuario
    lastname VARCHAR(255) NOT NULL,  -- Apellido del usuario
    email VARCHAR(255) NOT NULL UNIQUE,  -- Correo electrónico único
    password VARCHAR(255) NOT NULL,  -- Contraseña
    hash VARCHAR(255) NOT NULL,  -- Hash de la contraseña u otro uso
    birthdate DATE NOT NULL,  -- Fecha de nacimiento
    created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,  -- Fecha de creación
    modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP  -- Fecha de modificación
);



-- Tabla 'Buyer' (relacionada con 'User')
CREATE TABLE Buyer (
    id UUID PRIMARY KEY,  -- Llave primaria y foránea
    CONSTRAINT fk_user_buyer FOREIGN KEY (id) REFERENCES base_user (id) ON DELETE CASCADE  -- Llave foránea referenciando a 'User'
);



-- Tabla 'Seller' (relacionada con 'User')
CREATE TABLE Seller (
    id UUID PRIMARY KEY,  -- Llave primaria y foránea
    rating INT NOT NULL,  -- Calificación del vendedor
    store_name VARCHAR(255) NOT NULL,  -- Nombre de la tienda
    CONSTRAINT fk_user_seller FOREIGN KEY (id) REFERENCES base_user (id) ON DELETE CASCADE  -- Llave foránea referenciando a 'User'
);

CREATE TABLE Admins (
    id UUID PRIMARY KEY,  -- Llave primaria y foránea
    CONSTRAINT fk_user_admin FOREIGN KEY (id) REFERENCES base_user (id) ON DELETE CASCADE  -- Llave foránea referenciando a 'User'
);

-- Tabla 'Customer_address' (relacionada con 'User' a través de customer_id)
CREATE TABLE Customer_address (
    id UUID PRIMARY KEY,  -- Llave primaria
    customer_id UUID NOT NULL,  -- Llave foránea a 'User'
    street1 VARCHAR(255) NOT NULL,  -- Calle principal
    street2 VARCHAR(255),  -- Calle secundaria (opcional)
    postal_code VARCHAR(20) NOT NULL,  -- Código postal
    city VARCHAR(255) NOT NULL,  -- Ciudad
    province VARCHAR(255) NOT NULL,  -- Provincia o estado
    CONSTRAINT fk_user_address FOREIGN KEY (customer_id) REFERENCES base_user (id) ON DELETE CASCADE  -- Llave foránea referenciando a 'User'
);

CREATE TABLE Products (
    id UUID PRIMARY KEY default gen_random_uuid(),
    code VARCHAR(100) unique not null,
    name VARCHAR(100) not null,
    description TEXT not null,
    brand VARCHAR(100) not null,
    status SMALLINT DEFAULT 0 CHECK (status BETWEEN 0 AND 2), -- 0: DRAFT, 1: ACTIVE, 2: INACTIVE
    specs JSONB not null, -- Especificaciones del producto --> tendran una plantilla dependiendo de la categoria
    variations JSONB, --> LAS VARIACIONES DEL PRODUCTO 
    images JSONB, --> thumbnail, otras imagenes
    published BOOLEAN not null default false
);

create table Category(
	id varchar(10) primary key,
	name text,
	parent varchar(10),
	depth integer,
    base_specs jsonb,
    is_parent boolean not null default false,
	foreign key (parent) references Category(id)
);

--------------FUNCION CALCULAR PROFUNDIDAD----------------
CREATE OR REPLACE FUNCTION calculate_depth() 
RETURNS TRIGGER AS $$
DECLARE
    current_parent varchar(10);
    current_depth integer := 0;
BEGIN
    -- Inicializar el valor del parent de la fila que se está insertando o actualizando
    current_parent := NEW.parent;

    -- Subir en la jerarquía contando los niveles hasta que no haya más padres
    WHILE current_parent IS NOT NULL LOOP
        -- Buscar el padre del parent actual
        SELECT parent INTO current_parent
        FROM Category
        WHERE id = current_parent;

        -- Incrementar el contador de profundidad
        current_depth := current_depth + 1;
    END LOOP;

    -- Asignar la profundidad calculada
    NEW.depth := current_depth;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_depth_before_insert_update
BEFORE INSERT OR UPDATE ON Category
FOR EACH ROW
EXECUTE FUNCTION calculate_depth();
--------------FIN FUNCION CALCULAR PROFUNDIDAD----------------
-------------------UPDATE IS_PARENT-------------------
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

    -- Si se elimina una categoría, actualizar el padre
    IF TG_OP = 'DELETE' THEN
        -- Si el padre antiguo ya no tiene hijos, actualizar is_parent a FALSE
        UPDATE Category SET is_parent = FALSE WHERE id = OLD.parent AND NOT EXISTS (
            SELECT 1 FROM Category WHERE parent = OLD.parent
        );
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Crear el trigger para llamar a la función en INSERT, UPDATE y DELETE
CREATE TRIGGER update_is_parent_trigger
AFTER INSERT OR UPDATE OR DELETE ON Category
FOR EACH ROW EXECUTE FUNCTION update_is_parent();
-------------------END UPDATE IS_PARENT-------------------

-- Tabla 'Category_product' (relaciona 'Sub_category' con 'Product')
CREATE TABLE Category_product (
    category_id varchar(10) NOT NULL,  -- Llave foránea a la tabla 'Category'
    product_id UUID NOT NULL,  -- Llave foránea a la tabla 'Product'
    PRIMARY KEY (category_id, product_id),  -- Llave compuesta entre subcategoría y producto
    CONSTRAINT fk_sub_category_product FOREIGN KEY (category_id) REFERENCES Category (id) ON DELETE CASCADE,  -- Llave foránea referenciando a 'Sub_category'
    CONSTRAINT fk_product_category_product FOREIGN KEY (product_id) REFERENCES Products (id) ON DELETE CASCADE  -- Llave foránea referenciando a 'Product'
);

-- Tabla 'Product_attributes'
CREATE TABLE Product_attributes (
    id UUID PRIMARY KEY,  -- Llave primaria del atributo
    name VARCHAR(255) NOT NULL  -- Nombre del atributo (ej: Color, Tamaño)
);

-- Tabla 'Product_sku' (relacionada con 'Product') Son las variaciones de un product

CREATE TABLE Product_variations (
    id UUID PRIMARY KEY,
    product_id UUID NOT NULL,
    identifiers JSONB,
    sku TEXT,
    attributes JSONB,
    stock INT NOT NULL DEFAULT 0,
    FOREIGN KEY (product_id) REFERENCES products(id),
    images JSONB -- imagenes para la variacion
);

CREATE TABLE prices (
    id SERIAL PRIMARY KEY,
    variation_id UUID NOT NULL,
    price DECIMAL(10,2),
    currency CHAR(3),
    start_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    end_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (variation_id) REFERENCES product_variations(id)
);

-- Tabla de descuentos
CREATE TABLE discounts (
    id SERIAL PRIMARY KEY,
    variation_id UUID NOT NULL,
    discount_type VARCHAR(15) CHECK (discount_type IN ('percentage', 'fixed_amount')),
    discount_value DECIMAL(10,2),
    start_date TIMESTAMP,
    end_date TIMESTAMP,
    FOREIGN KEY (variation_id) REFERENCES product_variations(id)
);