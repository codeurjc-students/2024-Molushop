delete from Category;

select * from Category;

INSERT INTO Category (id, name, parent, depth) 
VALUES 
('ELEC', 'Electronics', NULL, 0),
('HOME', 'Home & Kitchen', NULL, 0),
('CLOT', 'Clothing', NULL, 0),
('BEAU', 'Beauty & Personal Care', NULL, 0),
('SPORT', 'Sports & Outdoors', NULL, 0);

INSERT INTO Category (id, name, parent, depth) 
VALUES 
('PHONE', 'Mobile Phones', 'ELEC', 1),
('LAPTOP', 'Laptops', 'ELEC', 1),
('CAMERA', 'Cameras & Photography', 'ELEC', 1),
('TV', 'Televisions', 'ELEC', 1),
('HEADP', 'Headphones', 'ELEC', 1);

INSERT INTO Category (id, name, parent, depth) 
VALUES 
('KITCH', 'Kitchen Appliances', 'HOME', 1),
('FURNI', 'Furniture', 'HOME', 1),
('DECOR', 'Home Décor', 'HOME', 1),
('BEDD', 'Bedding', 'HOME', 1),
('LIGHT', 'Lighting', 'HOME', 1);

INSERT INTO Category (id, name, parent, depth) 
VALUES 
('MENCL', 'Men Clothing', 'CLOT', 1),
('WOMCL', 'Women Clothing', 'CLOT', 1),
('CHILCL', 'Children Clothing', 'CLOT', 1),
('SHOES', 'Shoes', 'CLOT', 1),
('ACC', 'Accessories', 'CLOT', 1);

INSERT INTO Category (id, name, parent, depth) 
VALUES 
('SKIN', 'Skincare', 'BEAU', 1),
('MAKEUP', 'Makeup', 'BEAU', 1),
('HAIR', 'Hair Care', 'BEAU', 1),
('FRAGR', 'Fragrances', 'BEAU', 1),
('BATH', 'Bath & Body', 'BEAU', 1);

INSERT INTO Category (id, name, parent, depth) 
VALUES 
('OUTDO', 'Outdoor Gear', 'SPORT', 1),
('FITNESS', 'Fitness Equipment', 'SPORT', 1),
('CAMP', 'Camping & Hiking', 'SPORT', 1),
('BIKE', 'Cycling', 'SPORT', 1),
('TEAM', 'Team Sports', 'SPORT', 1);