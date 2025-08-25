select * from product_variations join prices on 

SELECT pv.*, p.price, p.currency, p.start_date
FROM product_variations pv
JOIN prices p ON pv.id = p.variation_id
WHERE p.currency = 'EUR';

select * from user_sessions;

UPDATE user_sessions
SET
    is_revoked = true
    -- ...
WHERE
    jti='41f9b164-809f-4a83-8a92-2ecbc91af812';

delete from base_user;

select * from base_user;

insert into base_user '$argon2id$v=19$m=19456,t=2,p=1$7km5tZ+xWYc3jCHQBHkIhw$oQxRg2zMQOIknmv0hE+FfFJkoIYG7AXhXKibl/qwAVE'

update base_user
set password = '$argon2id$v=19$m=19456,t=2,p=1$7km5tZ+xWYc3jCHQBHkIhw$oQxRg2zMQOIknmv0hE+FfFJkoIYG7AXhXKibl/qwAVE'
where id = '2064d62a-4978-4fe7-bef2-7690ff09bdc8'

UPDATE base_user
SET id = '2064d62a-4978-4fe7-bef2-7690ff09bdc8'
WHERE id = '15ea0b3a-d5bc-43a2-bb9a-5f5142da4692';

select * from seller;

select * from products;

insert into product_seller values ('50fe89df-e469-40bf-9ba2-6a0cf85e102e','2064d62a-4978-4fe7-bef2-7690ff09bdc8');
insert into product_seller values ('d3d0ff87-9480-4a6d-936f-e15a32c92aea','2064d62a-4978-4fe7-bef2-7690ff09bdc8');
insert into product_seller values ('9c468b87-13e7-4779-903e-85e04277a7ea','2064d62a-4978-4fe7-bef2-7690ff09bdc8');
insert into product_seller values ('979a0808-9d31-4b69-8bac-b778a0de2554','2064d62a-4978-4fe7-bef2-7690ff09bdc8');
insert into product_seller values ('d34b3342-8cdd-4e20-9aef-d73dd7913cbb','2064d62a-4978-4fe7-bef2-7690ff09bdc8');

select * from user_sessions;

select * from prices;

select * from products;

select * from product_variations;

select * from product_seller;

/*Hacer un join de products where id = x1 join product_variations join prices */

/* Obtener todos los precios de un prodcuto*/
SELECT 
    pv.id AS variation_id,
    pv.sku,
    pv.attributes,
    pv.stock,
    pv.status AS variation_status,
    p.price,
    p.currency,
    p.start_date AS price_start_date
FROM product_variations pv
INNER JOIN prices p ON pv.id = p.variation_id
WHERE pv.product_id = '50fe89df-e469-40bf-9ba2-6a0cf85e102e'  -- UUID del producto
ORDER BY pv.sku, p.currency;

SELECT 
    pv.id AS variation_id,
    p.price
FROM product_variations pv
INNER JOIN prices p ON pv.id = p.variation_id
WHERE pv.product_id = '50fe89df-e469-40bf-9ba2-6a0cf85e102e'  -- UUID del producto
ORDER BY p.price;