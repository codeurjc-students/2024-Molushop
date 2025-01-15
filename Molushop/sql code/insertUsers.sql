select * from base_user;
select * from seller;

delete from base_user;

insert into base_user  (id, name, lastname, email, password, hash, birthdate) values ('2064d62a-4978-4fe7-bef2-7690ff09bdc8', 'Super', 'Moluxo', 'moluxxxo@email.com','123456', 'hash123abc','1990-05-15');
insert into seller (id, rating, store_name) values ('2064d62a-4978-4fe7-bef2-7690ff09bdc8', 5, 'Moluxo Store');

INSERT INTO base_user (id, name, lastname, email, password, hash, birthdate)
VALUES 
(
    '2064d62a-4978-4fe7-bef2-7690ff09bdc8', 
    'Super', 
    'Moluxo', 
    'moluxxxo@email.com',
    '123456', 
    'hash123abc',
    '1990-05-15'
),
(
    gen_random_uuid(), 
    'María', 
    'González', 
    'maria.gonzalez@email.com',
    'abc123', 
    'hash456def',
    '1988-10-20'
),
(
    gen_random_uuid(), 
    'Carlos', 
    'Rodríguez', 
    'carlos.rodriguez@email.com',
    'pass789', 
    'hash789ghi',
    '1995-03-25'
),
(
    gen_random_uuid(), 
    'Ana', 
    'Martínez', 
    'ana.martinez@email.com',
    'qwerty123', 
    'hashjklmno',
    '1992-12-08'
),
(
    gen_random_uuid(), 
    'Luis', 
    'Sánchez', 
    'luis.sanchez@email.com',
    'xyz789', 
    'hashpqrstu',
    '1985-07-30'
);

