-- Nombres de las categorías en otros idiomas. El español sigue en
-- category.name (el idioma por defecto); aquí van los demás, de momento el
-- inglés (src/i18n.rs). Sin fila para un idioma, se usa category.name.
CREATE TABLE category_translation (
    category_id VARCHAR NOT NULL REFERENCES category(id) ON DELETE CASCADE ON UPDATE CASCADE,
    locale VARCHAR(8) NOT NULL,
    name VARCHAR NOT NULL,
    PRIMARY KEY (category_id, locale)
);

-- Las de las bases que ya tienen categorías. En una base vacía no inserta
-- nada: llegan después con seeds/init.sql, que ya trae estas filas.
INSERT INTO category_translation (category_id, locale, name)
SELECT v.id, 'en', v.name
FROM (VALUES
    ('BEAU', 'Beauty & Personal Care'),
    ('CLOT', 'Clothing'),
    ('ELEC', 'Electronics'),
    ('HOME', 'Home & Kitchen'),
    ('SPORT', 'Sports & Outdoors'),
    ('BATH', 'Bath & Body'),
    ('FRAGR', 'Fragrances'),
    ('HAIR', 'Hair Care'),
    ('MAKEUP', 'Makeup'),
    ('SKIN', 'Skincare'),
    ('ACC', 'Accessories'),
    ('CHILCL', 'Children Clothing'),
    ('MENCL', 'Men Clothing'),
    ('SHOES', 'Shoes'),
    ('WOMCL', 'Women Clothing'),
    ('CAMERA', 'Cameras & Photography'),
    ('HEADP', 'Headphones'),
    ('LAPTOP', 'Laptops'),
    ('PHONE', 'Mobile Phones'),
    ('TV', 'Televisions'),
    ('BEDD', 'Bedding'),
    ('DECOR', 'Home Décor'),
    ('FURNI', 'Furniture'),
    ('KITCH', 'Kitchen Appliances'),
    ('LIGHT', 'Lighting'),
    ('BIKE', 'Cycling'),
    ('CAMP', 'Camping & Hiking'),
    ('FITNESS', 'Fitness Equipment'),
    ('OUTDO', 'Outdoor Gear'),
    ('PADEL', 'Padel'),
    ('TEAM', 'Team Sports'),
    ('ACCESS', 'Fitness Accessories'),
    ('CARDIO', 'Cardio Equipment'),
    ('STRENGTH', 'Strength Training'),
    ('WEIGHTS', 'Weights & Dumbbells'),
    ('YOGA', 'Yoga & Pilates')
) AS v(id, name)
JOIN category c ON c.id = v.id;
