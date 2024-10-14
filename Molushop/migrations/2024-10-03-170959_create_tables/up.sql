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



CREATE TABLE Product (
    id UUID PRIMARY KEY,  -- Llave primaria del producto
    name VARCHAR(255) NOT NULL,  -- Nombre del producto
    description TEXT,  -- Descripción detallada del producto
    summary TEXT,  -- Resumen corto del producto
    cover VARCHAR(255)  -- URL o path de la imagen de portada del producto
);



create table Category(
	id varchar(10) primary key,
	name text,
	parent varchar(10),
	depth integer,
	foreign key (parent) references Category(id)
);




-- Tabla 'Category_product' (relaciona 'Sub_category' con 'Product')
CREATE TABLE Category_product (
    category_id varchar(10) NOT NULL,  -- Llave foránea a la tabla 'Category'
    product_id UUID NOT NULL,  -- Llave foránea a la tabla 'Product'
    PRIMARY KEY (category_id, product_id),  -- Llave compuesta entre subcategoría y producto
    CONSTRAINT fk_sub_category_product FOREIGN KEY (category_id) REFERENCES Category (id) ON DELETE CASCADE,  -- Llave foránea referenciando a 'Sub_category'
    CONSTRAINT fk_product_category_product FOREIGN KEY (product_id) REFERENCES Product (id) ON DELETE CASCADE  -- Llave foránea referenciando a 'Product'
);

-- Tabla 'Product_attributes'
CREATE TABLE Product_attributes (
    id UUID PRIMARY KEY,  -- Llave primaria del atributo
    name VARCHAR(255) NOT NULL  -- Nombre del atributo (ej: Color, Tamaño)
);

create table Product_attributes_category (
    id serial primary key,
	product_att_id UUID references Product_attributes(id),
	category_id varchar(10) references Category(id),
	constraint unique_dupla UNIQUE (product_att_id,category_id)
);


-- Tabla 'Product_attributes_options' (relacionada con 'Product_attributes')
CREATE TABLE Product_attributes_options (
    id UUID PRIMARY KEY,  -- Llave primaria de la opción de atributo
    attribute_id UUID NOT NULL,  -- Llave foránea a 'Product_attributes'
    name VARCHAR(255) NOT NULL,  -- Nombre de la opción (ej: Rojo, Azul, Grande)
    CONSTRAINT fk_product_attributes_options FOREIGN KEY (attribute_id) REFERENCES Product_attributes (id) ON DELETE CASCADE  -- Llave foránea referenciando a 'Product_attributes'
);



-- Tabla 'Product_sku' (relacionada con 'Product')
CREATE TABLE Product_sku (
    id UUID PRIMARY KEY,  -- Llave primaria del SKU
    product_id UUID NOT NULL,  -- Llave foránea a la tabla 'Product'
    sku VARCHAR(255) NOT NULL UNIQUE,  -- Código SKU único del producto
    stock INT NOT NULL,  -- Cantidad en inventario
    CONSTRAINT fk_product_sku FOREIGN KEY (product_id) REFERENCES Product (id) ON DELETE CASCADE  -- Llave foránea referenciando a 'Product'
);



-- Tabla 'Attribute_option_sku' (relaciona 'Product_sku' con 'Product_attributes_options')
CREATE TABLE Attribute_option_sku (
    sku_id UUID NOT NULL,  -- Llave foránea a 'Product_sku'
    prod_att_option_id UUID NOT NULL,  -- Llave foránea a 'Product_attributes_options'
    PRIMARY KEY (sku_id, prod_att_option_id),  -- Llave compuesta entre SKU y opción de atributo
    CONSTRAINT fk_sku_attribute_option FOREIGN KEY (sku_id) REFERENCES Product_sku (id) ON DELETE CASCADE,  -- Llave foránea referenciando a 'Product_sku'
    CONSTRAINT fk_attribute_option_sku FOREIGN KEY (prod_att_option_id) REFERENCES Product_attributes_options (id) ON DELETE CASCADE  -- Llave foránea referenciando a 'Product_attributes_options'
);