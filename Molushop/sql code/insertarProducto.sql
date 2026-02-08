select * from products;
select * from product_seller;
select * from product_variations;
delete from products;
--Bici Mountain RX
insert into product_seller (product_id, seller_id) values ('90dca2f3-a54e-4839-be67-2c801e6ad05e', '2064d62a-4978-4fe7-bef2-7690ff09bdc8');
--Pala EKC
insert into product_seller (product_id, seller_id) values ('7f91cd4c-a152-4d53-8176-f092732fedab', '2064d62a-4978-4fe7-bef2-7690ff09bdc8');
--BMX Pro
insert into product_seller (product_id, seller_id) values ('347d0773-7430-4344-8333-46c6fa4b6fa1', '2064d62a-4978-4fe7-bef2-7690ff09bdc8');
--Chaqueta de cuero
insert into product_seller (product_id, seller_id) values ('9e95b98c-6314-4fdb-a109-2c888a71bf89', '2064d62a-4978-4fe7-bef2-7690ff09bdc8');

-- el id de la categoría se tiene que insertar con un método
-- 
INSERT INTO Products (id, code, name, description, brand, specs, variations, images, published) VALUES ('3d3abb00-aa89-4aca-9532-e2fe574037a6', '1234PX', 'PhoneXD', 'Es un telefono muy pontente', 'Lenovo', '{"os":"Android","screen_size":"50MPX"}', '[{"name":"Color","values":[{"value":"Azul"},{"value":"Verde"},{"value":"Negro"}]},{"name":"Paquete","values":[{"value":"4GB RAM + 128GB ROM"},{"value":"6GB RAM + 512GB ROM"}]}]', null, false);

INSERT INTO Products (id, code, name, description, brand, specs, variations, images, published) VALUES ('4d4acc11-bb89-4bca-9642-f3fe574037b7', '1235PX', 'UltraPhone', 'Smartphone de última generación', 'Samsung', '{"os":"Android","screen_size":"6.7","camera":"108MPX"}', '[{"name":"Color","values":[{"value":"Negro"},{"value":"Blanco"},{"value":"Dorado"}]},{"name":"Almacenamiento","values":[{"value":"128GB"},{"value":"256GB"}]}]', null, true);

INSERT INTO Products (id, code, name, description, brand, specs, variations, images, published) VALUES ('a71f755b-397f-418b-84c6-9ea45092e4d1', '1236PX', 'ProBook Air', 'Laptop ultraligera para profesionales', 'Apple', '{"os":"MacOS","screen_size":"13.3","processor":"M2"}', '[{"name":"Color","values":[{"value":"Plata"},{"value":"Gris Espacial"}]},{"name":"RAM","values":[{"value":"8GB"},{"value":"16GB"}]}]', null, true);

INSERT INTO Products (id, code, name, description, brand, specs, variations, images, published) VALUES ('29f11934-3b4f-4c17-886a-7054a33ce26e', '1237PX', 'GameStation 5', 'Consola de videojuegos next-gen', 'Sony', '{"storage":"825GB","resolution":"4K","fps":"120"}', '[{"name":"Modelo","values":[{"value":"Digital"},{"value":"Disco"}]},{"name":"Bundle","values":[{"value":"Solo consola"},{"value":"Con juego"}]}]', null, true);

INSERT INTO Products (id, code, name, description, brand, specs, variations, images, published) VALUES ('f2f99856-145e-4da1-a670-b46f98773463', '1238PX', 'SmartWatch Pro', 'Reloj inteligente con monitor cardíaco', 'Xiaomi', '{"screen":"AMOLED","battery":"14 days","water_resistant":"5ATM"}', '[{"name":"Color","values":[{"value":"Negro"},{"value":"Azul"}]},{"name":"Tamaño","values":[{"value":"41mm"},{"value":"45mm"}]}]', null, false);

INSERT INTO Products (id, code, name, description, brand, specs, variations, images, published) VALUES ('57a61e32-49fa-4411-92af-687e8492c605', '1239PX', 'TabletPro X', 'Tablet para diseñadores', 'Huawei', '{"screen":"10.8","pen_support":"Yes","processor":"Kirin 990"}', '[{"name":"Color","values":[{"value":"Gris"},{"value":"Verde"}]},{"name":"Storage","values":[{"value":"64GB"},{"value":"128GB"}]}]', null, true);

INSERT INTO Products (id, code, name, description, brand, specs, variations, images, published) VALUES ('ac740e3e-2df8-4ffb-95ec-f9c502f10deb', '1240PX', 'AudioPods', 'Auriculares inalámbricos premium', 'Apple', '{"type":"TWS","battery":"24h","noise_cancelling":"Yes"}', '[{"name":"Color","values":[{"value":"Blanco"},{"value":"Negro"}]},{"name":"Modelo","values":[{"value":"Normal"},{"value":"Pro"}]}]', null, true);

--INSERT INTO Products (id, code, name, description, brand, specs, variations, images, published) VALUES ('ac740e3e-2df8-4ffb-95ec-f9c502f10deb', '1241PX', 'SmartTV 4K', 'Televisor inteligente con HDR', 'LG', '{"resolution":"4K","size":"55","smart":"webOS"}', '[{"name":"Tamaño","values":[{"value":"55""},{"value":"65""}]},{"name":"Serie","values":[{"value":"Basic"},{"value":"Premium"}]}]', null, true);

INSERT INTO Products (id, code, name, description, brand, specs, variations, images, published) VALUES ('90dca2f3-a54e-4839-be67-2c801e6ad05e', '1242PX', 'PowerBank Pro', 'Batería portátil de alta capacidad', 'Anker', '{"capacity":"20000mAh","fast_charge":"Yes","ports":"USB-C"}', '[{"name":"Color","values":[{"value":"Negro"},{"value":"Blanco"}]},{"name":"Capacidad","values":[{"value":"10000mAh"},{"value":"20000mAh"}]}]', null, false);

INSERT INTO Products (id, code, name, description, brand, specs, variations, images, published) VALUES ('64d29964-088f-4774-af6c-39e0e8945757', '1243PX', 'GamePad Elite', 'Control para gaming profesional', 'Microsoft', '{"wireless":"Yes","battery":"40h","customizable":"Yes"}', '[{"name":"Color","values":[{"value":"Negro"},{"value":"Blanco"},{"value":"Rojo"}]},{"name":"Edición","values":[{"value":"Standard"},{"value":"Pro"}]}]', null, true);

INSERT INTO Products (id, code, name, description, brand, specs, variations, images, published) VALUES ('165361cd-fb4a-42bd-b244-d811b6a09dbb', '1244PX', 'RouterPro X', 'Router WiFi 6 de alto rendimiento', 'TP-Link', '{"wifi":"WiFi 6","speed":"AX6000","coverage":"2500sqft"}', '[{"name":"Modelo","values":[{"value":"Basic"},{"value":"Advanced"}]},{"name":"Antenas","values":[{"value":"4"},{"value":"8"}]}]', null, true);

SELECT p.*
FROM products p
INNER JOIN product_seller sp ON p.id = sp.product_id
WHERE sp.seller_id = '2064d62a-4978-4fe7-bef2-7690ff09bdc8' AND p.id = '9e95b98c-6314-4fdb-a109-2c888a71bf89';

select * from identifiers_base;