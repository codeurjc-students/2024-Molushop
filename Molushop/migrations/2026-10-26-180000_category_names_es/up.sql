-- Nombres de las categorías en español. Los id no cambian (van en las URL).
-- En una base vacía no toca nada: las categorías llegan después con
-- seeds/init.sql, que ya trae estos nombres.
UPDATE category c SET name = v.name
FROM (VALUES
    ('BEAU', 'Belleza y cuidado personal'),
    ('CLOT', 'Ropa'),
    ('ELEC', 'Electrónica'),
    ('HOME', 'Hogar y cocina'),
    ('SPORT', 'Deportes y aire libre'),
    ('BATH', 'Baño y cuerpo'),
    ('FRAGR', 'Perfumes'),
    ('HAIR', 'Cuidado del cabello'),
    ('MAKEUP', 'Maquillaje'),
    ('SKIN', 'Cuidado de la piel'),
    ('ACC', 'Complementos'),
    ('CHILCL', 'Ropa infantil'),
    ('MENCL', 'Ropa de hombre'),
    ('SHOES', 'Calzado'),
    ('WOMCL', 'Ropa de mujer'),
    ('CAMERA', 'Cámaras y fotografía'),
    ('HEADP', 'Auriculares'),
    ('LAPTOP', 'Portátiles'),
    ('PHONE', 'Móviles'),
    ('TV', 'Televisores'),
    ('BEDD', 'Ropa de cama'),
    ('DECOR', 'Decoración'),
    ('FURNI', 'Muebles'),
    ('KITCH', 'Electrodomésticos de cocina'),
    ('LIGHT', 'Iluminación'),
    ('BIKE', 'Ciclismo'),
    ('CAMP', 'Camping y senderismo'),
    ('FITNESS', 'Fitness'),
    ('OUTDO', 'Equipo de exterior'),
    ('PADEL', 'Pádel'),
    ('TEAM', 'Deportes de equipo'),
    ('ACCESS', 'Accesorios de fitness'),
    ('CARDIO', 'Cardio'),
    ('STRENGTH', 'Entrenamiento de fuerza'),
    ('WEIGHTS', 'Pesas y mancuernas'),
    ('YOGA', 'Yoga y pilates')
) AS v(id, name)
WHERE c.id = v.id;
