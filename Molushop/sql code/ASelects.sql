select * from base_user;
select * from products;
select * from admins;
select * from Product_variations;
SELECT * FROM Product_variations
WHERE attributes @> '[{"name": "Color", "value": "Negro"}]';

select * from category;
delete from category;
insert into category values ('1','categoria 1',null);
insert into category values ('1.1','categoria 1.1','1');
insert into category values ('1.1.1','categoria 3','1.1');

insert into category values ('2','categoria 2',null);
insert into category values ('3','categoria 3',null,34);

update category set parent='1.1' where id='3';

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


DROP TRIGGER IF EXISTS set_depth_before_insert_update ON Category;
DROP FUNCTION IF EXISTS calculate_depth();

CREATE TABLE images_product(
    id UUID PRIMARY KEY,
    product_id UUID NOT NULL,
    image_url TEXT NOT NULL,
    is_main BOOLEAN DEFAULT FALSE,
    display_order INT NOT NULL DEFAULT 1,
    CONSTRAINT fk_image_product FOREIGN KEY (product_id) REFERENCES Products (id) ON DELETE CASCADE
);

CREATE TABLE images_product_variations(
    image_id UUID NOT NULL,
    variation_id UUID NOT NULL,
    PRIMARY KEY (variation_id, image_id),
    CONSTRAINT fk_ipv_image FOREIGN KEY (image_id) REFERENCES images_product (id) ON DELETE CASCADE,
    CONSTRAINT fk_ipv_variation FOREIGN KEY (variation_id) REFERENCES Product_variations (id) ON DELETE CASCADE
);

select * from images_product;
select * from images_product_variations;

select * from prices;

select * from images_product;
--consulta principal lo malo es que obtiene varias imágenes 
select p.id,
    p.name,
    p.brand,
    ip.image_url,
    MIN(pr.price) price,
    pr.currency,
    s.store_name,
    s.id store_id
from products p
left join images_product ip on p.id=ip.product_id
left join product_variations pv on p.id=pv.product_id
left join prices pr on pv.id=pr.variation_id
left join product_seller ps on p.id=ps.product_id
left join seller s on ps.seller_id=s.id
where p.status = 1
group by p.id, p.name, p.brand, ip.image_url, pr.currency, s.store_name, s.id
order by p.name;

select p.id,
    p.name,
    p.brand,
    (SELECT img.image_url 
     FROM images_product img 
     WHERE img.product_id = p.id 
     ORDER BY img.is_main DESC, img.id ASC 
     LIMIT 1) as image_url,
    MIN(pr.price) price,
    pr.currency,
    s.store_name,
    s.id store_id
from products p
left join product_variations pv on p.id=pv.product_id
left join prices pr on pv.id=pr.variation_id
left join product_seller ps on p.id=ps.product_id
left join seller s on ps.seller_id=s.id
where p.status = 1
group by p.id, p.name, p.brand, pr.currency, s.store_name, s.id
order by p.name;

select * from prices;
select * from product_seller;

-- Seleccion de productos con imágenes --->
select p.*,
    json_agg(json_build_object(
        'image_url', i.image_url,
        'is_main', i.is_main,
        'display_order', i.display_order
    )) FILTER (WHERE i.id IS NOT NULL) as images_product
FROM products p
LEFT JOIN images_product i on p.id = i.product_id
LEFT JOIN product_seller ps ON p.id = ps.product_id
WHERE ps.seller_id = '2064d62a-4978-4fe7-bef2-7690ff09bdc8'
GROUP BY p.id

select * from products;
-- ver las variaciones 


WITH stock_por_atributo AS (
    -- 1. "Aplanamos" los atributos de todas las variaciones de un producto
    SELECT 
        v.product_id,
        attr->>'name' as attr_name,
        attr->>'value' as attr_value,
        SUM(v.stock) as total_stock
    FROM Product_variations v,
    LATERAL jsonb_array_elements(v.attributes) AS attr
    WHERE v.product_id = '3a70c5ad-5e65-45bb-a2e0-e8cda4efcbef' -- Filtrar por el producto deseado
    GROUP BY v.product_id, attr_name, attr_value
)
-- 2. Cruzamos con la definición global de la tabla Products
SELECT 
    p.id,
    p.name,
    (
        SELECT jsonb_agg(
            jsonb_build_object(
                'name', v_def->>'name',
                'values', (
                    SELECT jsonb_agg(
                        jsonb_build_object(
                            'value', val->>'value',
                            -- Si el stock sumado para este valor es 0, inStock es false
                            'inStock', COALESCE(s.total_stock, 0) > 0
                        )
                    )
                    FROM jsonb_array_elements(v_def->'values') AS val
                    LEFT JOIN stock_por_atributo s 
                        ON s.attr_name = v_def->>'name' 
                        AND s.attr_value = val->>'value'
                )
            )
        )
        FROM jsonb_array_elements(p.variations) AS v_def
    ) AS variations_with_stock_status
FROM Products p
WHERE p.id = '3a70c5ad-5e65-45bb-a2e0-e8cda4efcbef';

----- Otro pero ahora con imágenes, faltaría los precios también
WITH stock_por_atributo AS (
    -- 1. "Aplanamos" los atributos de todas las variaciones de un producto
    SELECT 
        v.product_id,
        attr->>'name' as attr_name,
        attr->>'value' as attr_value,
        SUM(v.stock) as total_stock
    FROM Product_variations v,
    LATERAL jsonb_array_elements(v.attributes) AS attr
    WHERE v.product_id = '3a70c5ad-5e65-45bb-a2e0-e8cda4efcbef' -- Filtrar por el producto deseado
    GROUP BY v.product_id, attr_name, attr_value
)
-- 2. Cruzamos con la definición global de la tabla Products
SELECT 
    p.id,
    p.name,
    p.brand,
    (SELECT img.image_url 
    FROM images_product img 
    WHERE img.product_id = p.id 
    ORDER BY img.is_main DESC, img.id ASC 
    LIMIT 1) as image_url,
    MIN(pr.price) price,
    pr.currency,
    s.store_name,
    s.id store_id,
    json_agg(DISTINCT jsonb_build_object(
        'image_url', i.image_url,
        'is_main', i.is_main,
        'display_order', i.display_order
    )) FILTER (WHERE i.id IS NOT NULL) as images_product,
    (
        SELECT jsonb_agg(
            jsonb_build_object(
                'name', v_def->>'name',
                'values', (
                    SELECT jsonb_agg(
                        jsonb_build_object(
                            'value', val->>'value',
                            -- Si el stock sumado para este valor es 0, inStock es false
                            'inStock', COALESCE(s.total_stock, 0) > 0
                        )
                    )
                    FROM jsonb_array_elements(v_def->'values') AS val
                    LEFT JOIN stock_por_atributo s 
                        ON s.attr_name = v_def->>'name' 
                        AND s.attr_value = val->>'value'
                )
            )
        )
        FROM jsonb_array_elements(p.variations) AS v_def
    ) AS variations_with_stock_status
FROM Products p
LEFT JOIN images_product i on p.id = i.product_id
left join product_variations pv on p.id=pv.product_id
left join prices pr on pv.id=pr.variation_id
left join product_seller ps on p.id=ps.product_id
left join seller s on ps.seller_id=s.id
WHERE p.id = '3a70c5ad-5e65-45bb-a2e0-e8cda4efcbef' and p.status = 1
group by p.id, p.name, p.brand, pr.currency, s.store_name, s.id;

---- Lo mismo que lo de arriba pero más eficiente:
WITH stock_por_atributo AS (
    -- 1. "Aplanamos" los atributos de todas las variaciones de un producto
    SELECT 
        v.product_id,
        attr->>'name' as attr_name,
        attr->>'value' as attr_value,
        SUM(v.stock) as total_stock
    FROM Product_variations v,
    LATERAL jsonb_array_elements(v.attributes) AS attr
    WHERE v.product_id = '3a70c5ad-5e65-45bb-a2e0-e8cda4efcbef' -- Filtrar por el producto deseado
    GROUP BY v.product_id, attr_name, attr_value
)
-- 2. Cruzamos con la definición global de la tabla Products
SELECT 
    p.id,
    p.name,
    p.brand,
    p.description,
    MIN(pr.price) price,
    pr.currency,
    s.store_name,
    s.id store_id,
    img_agg.lista_imagenes as images_product,
    (
        SELECT jsonb_agg(
            jsonb_build_object(
                'name', v_def->>'name',
                'values', (
                    SELECT jsonb_agg(
                        jsonb_build_object(
                            'value', val->>'value',
                            -- Si el stock sumado para este valor es 0, inStock es false
                            'in_stock', COALESCE(s.total_stock, 0) > 0
                        )
                    )
                    FROM jsonb_array_elements(v_def->'values') AS val
                    LEFT JOIN stock_por_atributo s 
                        ON s.attr_name = v_def->>'name' 
                        AND s.attr_value = val->>'value'
                )
            )
        )
        FROM jsonb_array_elements(p.variations) AS v_def
    ) AS variations_with_stock_status
FROM Products p
LEFT JOIN LATERAL (
    SELECT jsonb_agg(jsonb_build_object(
        'image_url', img.image_url,
        'is_main', img.is_main,
        'display_order', img.display_order
    )) as lista_imagenes
    FROM images_product img
    WHERE img.product_id = p.id
) img_agg ON true
left join product_variations pv on p.id=pv.product_id
left join prices pr on pv.id=pr.variation_id
left join product_seller ps on p.id=ps.product_id
left join seller s on ps.seller_id=s.id
WHERE p.id = '3a70c5ad-5e65-45bb-a2e0-e8cda4efcbef' and p.status = 1
GROUP BY p.id, p.name, p.brand, pr.currency, s.store_name, s.id, img_agg.lista_imagenes;

WITH stock_por_atributo AS (
    SELECT 
        v.product_id,
        attr->>'name' as attr_name,
        attr->>'value' as attr_value,
        SUM(v.stock) as total_stock
    FROM Product_variations v,
    LATERAL jsonb_array_elements(v.attributes) AS attr
    WHERE v.product_id = '03ead311-27d2-4b00-8e40-b52b433b3a51' AND v.status = 1 -- Solo variaciones activas
    GROUP BY v.product_id, attr_name, attr_value
),
variacion_prioritaria AS (
    -- Buscamos la mejor variación para ser la "por defecto"
    SELECT attributes
    FROM Product_variations
    WHERE product_id = '03ead311-27d2-4b00-8e40-b52b433b3a51' AND status = 1
    ORDER BY (stock > 0) DESC, stock DESC -- Prioriza que tenga stock
    LIMIT 1
)
SELECT 
    p.id,
    p.name,
    p.brand,
    p.description,
    MIN(pr.price) as price,
    pr.currency,
    s.store_name,
    s.id as store_id,
    img_agg.lista_imagenes as images_product,
    -- Aquí inyectamos cuál es la combinación por defecto
    (SELECT attributes FROM variacion_prioritaria) as default_selection,
    (
        SELECT jsonb_agg(
            jsonb_build_object(
                'name', v_def->>'name',
                'values', (
                    SELECT jsonb_agg(
                        jsonb_build_object(
                            'value', val->>'value',
                            'in_stock', COALESCE(sa.total_stock, 0) > 0,
                            -- Marcamos si este valor específico es parte de la selección por defecto
                            'is_default', EXISTS (
                                SELECT 1 FROM variacion_prioritaria vp, 
                                LATERAL jsonb_array_elements(vp.attributes) as def_attr
                                WHERE def_attr->>'name' = v_def->>'name' 
                                AND def_attr->>'value' = val->>'value'
                            )
                        )
                    )
                    FROM jsonb_array_elements(v_def->'values') AS val
                    LEFT JOIN stock_por_atributo sa 
                        ON sa.attr_name = v_def->>'name' 
                        AND sa.attr_value = val->>'value'
                )
            )
        )
        FROM jsonb_array_elements(p.variations) AS v_def
    ) AS variations_with_stock_status
FROM Products p
LEFT JOIN LATERAL (
    SELECT jsonb_agg(jsonb_build_object(
        'image_url', img.image_url,
        'is_main', img.is_main,
        'display_order', img.display_order
    )) as lista_imagenes
    FROM images_product img
    WHERE img.product_id = p.id
) img_agg ON true
LEFT JOIN Product_variations pv ON p.id = pv.product_id
LEFT JOIN prices pr ON pv.id = pr.variation_id
LEFT JOIN product_seller ps ON p.id = ps.product_id
LEFT JOIN seller s ON ps.seller_id = s.id
WHERE p.id = '03ead311-27d2-4b00-8e40-b52b433b3a51' AND p.status = 1
GROUP BY p.id, p.name, p.brand, pr.currency, s.store_name, s.id, img_agg.lista_imagenes;


WITH variacion_prioritaria AS (
    SELECT v.id, v.attributes, pr.price, pr.currency
    FROM Product_variations v
    LEFT JOIN prices pr ON v.id = pr.variation_id
    WHERE v.product_id = '01ab75de-8d9f-41d6-acb5-0a42d7d5d6bc' AND v.status = 1
    ORDER BY (v.stock > 0) DESC, v.stock DESC 
    LIMIT 1
),
stock_por_atributo AS (
    SELECT 
        v.product_id,
        attr->>'name' as attr_name,
        attr->>'value' as attr_value,
        SUM(v.stock) as total_stock
    FROM Product_variations v,
    LATERAL jsonb_array_elements(v.attributes) AS attr
    WHERE v.product_id = '01ab75de-8d9f-41d6-acb5-0a42d7d5d6bc' AND v.status = 1
    GROUP BY v.product_id, attr_name, attr_value
),
todas_las_combinaciones AS (
    -- Esta es la tabla de búsqueda para tu JavaScript
    SELECT jsonb_agg(
        jsonb_build_object(
            'attributes', v.attributes,
            'price', pr.price,
            'id', v.id,
            'stock', v.stock
        )
    ) as mapa
    FROM Product_variations v
    LEFT JOIN prices pr ON v.id = pr.variation_id
    WHERE v.product_id = '01ab75de-8d9f-41d6-acb5-0a42d7d5d6bc' AND v.status = 1
)
SELECT 
    p.id,
    p.name,
    p.brand,
    p.description,
    pr.currency,
    s.store_name,
    s.id store_id,
    img_agg.lista_imagenes as images_product,
    -- TABLA DE BÚSQUEDA PARA JS
    (SELECT mapa FROM todas_las_combinaciones) as variant_map,
    (
        SELECT jsonb_agg(
            jsonb_build_object(
                'name', v_def->>'name',
                'values', (
                    SELECT jsonb_agg(
                        jsonb_build_object(
                            'value', val->>'value',
                            'in_stock', COALESCE(sa.total_stock, 0) > 0,
                            'is_default', EXISTS (
                                SELECT 1 FROM variacion_prioritaria vp, 
                                LATERAL jsonb_array_elements(vp.attributes) as def_attr
                                WHERE def_attr->>'name' = v_def->>'name' 
                                AND def_attr->>'value' = val->>'value'
                            )
                        )
                    )
                    FROM jsonb_array_elements(v_def->'values') AS val
                    LEFT JOIN stock_por_atributo sa 
                        ON sa.attr_name = v_def->>'name' 
                        AND sa.attr_value = val->>'value'
                )
            )
        )
        FROM jsonb_array_elements(p.variations) AS v_def
    ) AS variations_with_stock_status
FROM Products p
LEFT JOIN LATERAL (
    SELECT jsonb_agg(jsonb_build_object(
        'image_url', img.image_url,
        'is_main', img.is_main,
        'display_order', img.display_order
    )) as lista_imagenes
    FROM images_product img
    WHERE img.product_id = p.id
) img_agg ON true
LEFT JOIN Product_variations pv ON p.id = pv.product_id
LEFT JOIN prices pr ON pv.id = pr.variation_id
LEFT JOIN product_seller ps ON p.id = ps.product_id
LEFT JOIN seller s ON ps.seller_id = s.id
WHERE p.id = '01ab75de-8d9f-41d6-acb5-0a42d7d5d6bc' AND p.status = 1
GROUP BY p.id, p.name, p.brand, pr.currency, s.store_name, s.id, img_agg.lista_imagenes;
--lo mismo pero con
WITH variacion_prioritaria AS (
    SELECT v.id, v.attributes, pr.price, pr.currency
    FROM Product_variations v
    LEFT JOIN prices pr ON v.id = pr.variation_id
    WHERE v.product_id = '01ab75de-8d9f-41d6-acb5-0a42d7d5d6bc' AND v.status = 1
    ORDER BY (v.stock > 0) DESC, v.stock DESC 
    LIMIT 1
),
stock_por_atributo AS (
    SELECT 
        v.product_id,
        attr->>'name' as attr_name,
        attr->>'value' as attr_value,
        SUM(v.stock) as total_stock
    FROM Product_variations v,
    LATERAL jsonb_array_elements(v.attributes) AS attr
    WHERE v.product_id = '01ab75de-8d9f-41d6-acb5-0a42d7d5d6bc' AND v.status = 1
    GROUP BY v.product_id, attr_name, attr_value
),
todas_las_combinaciones AS (
    -- Esta es la tabla de búsqueda para tu JavaScript
    SELECT jsonb_agg(
        jsonb_build_object(
            'attributes', v.attributes,
            'price', pr.price,
            'id', v.id,
            'stock', v.stock
        )
        ORDER BY pr.price ASC
    ) as mapa
    FROM Product_variations v
    LEFT JOIN prices pr ON v.id = pr.variation_id
    WHERE v.product_id = '01ab75de-8d9f-41d6-acb5-0a42d7d5d6bc' AND v.status = 1
)
SELECT 
    p.id,
    p.name,
    p.brand,
    p.description,
    pr.currency,
    s.store_name,
    s.id store_id,
    img_agg.lista_imagenes as images_product,
    -- TABLA DE BÚSQUEDA PARA JS
    (SELECT mapa FROM todas_las_combinaciones) as variant_map,
    (
        SELECT jsonb_agg(
            jsonb_build_object(
                'name', v_def->>'name',
                'values', (
                    SELECT jsonb_agg(
                        jsonb_build_object(
                            'value', val->>'value',
                            'in_stock', COALESCE(sa.total_stock, 0) > 0,
                            'is_default', EXISTS (
                                SELECT 1 FROM variacion_prioritaria vp, 
                                LATERAL jsonb_array_elements(vp.attributes) as def_attr
                                WHERE def_attr->>'name' = v_def->>'name' 
                                AND def_attr->>'value' = val->>'value'
                            )
                        )
                    )
                    FROM jsonb_array_elements(v_def->'values') AS val
                    LEFT JOIN stock_por_atributo sa 
                        ON sa.attr_name = v_def->>'name' 
                        AND sa.attr_value = val->>'value'
                )
            )
        )
        FROM jsonb_array_elements(p.variations) AS v_def
    ) AS variations_with_stock_status
FROM Products p
LEFT JOIN LATERAL (
    SELECT jsonb_agg(jsonb_build_object(
        'image_url', img.image_url,
        'is_main', img.is_main,
        'display_order', img.display_order
    )) as lista_imagenes
    FROM images_product img
    WHERE img.product_id = p.id
) img_agg ON true
LEFT JOIN Product_variations pv ON p.id = pv.product_id
LEFT JOIN prices pr ON pv.id = pr.variation_id
LEFT JOIN product_seller ps ON p.id = ps.product_id
LEFT JOIN seller s ON ps.seller_id = s.id
WHERE p.id = '01ab75de-8d9f-41d6-acb5-0a42d7d5d6bc' AND p.status = 1
GROUP BY p.id, p.name, p.brand, pr.currency, s.store_name, s.id, img_agg.lista_imagenes;