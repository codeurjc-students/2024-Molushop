-- Roles del panel (/panel y /admin). Los roles son tablas, no columnas:
--   admin    = fila en `admins` (acceso global)
--   vendedor = fila en `seller` (gestiona sus productos, vía product_seller)
-- Un usuario puede ser las dos cosas: el admin crea productos a nombre de su tienda.
-- Nunca se da un rol desde la web: siempre a mano con este SQL.

-- 1. Ver quién tiene qué rol
-- (desde la migración stores_multi: una fila por tienda del usuario;
--  estado 0 pendiente, 1 activa, 2 inactiva, 3 rechazada)
SELECT bu.id, bu.username, bu.email,
       (a.id IS NOT NULL) AS es_admin,
       s.store_name       AS tienda,
       s.status           AS estado_tienda,
       s.id               AS store_id
FROM base_user bu
LEFT JOIN admins a ON a.id = bu.id
LEFT JOIN store_members sm ON sm.user_id = bu.id
LEFT JOIN seller s ON s.id = sm.store_id
ORDER BY es_admin DESC, tienda NULLS LAST, bu.username;

-- 2. Hacer admin a un usuario (cambiar el username)
INSERT INTO admins (id)
SELECT id FROM base_user WHERE username = 'Mox'
ON CONFLICT (id) DO NOTHING;

-- 3. Dar tienda (vendedor) a un usuario (cambiar username y nombre de la tienda)
-- Desde la migración stores_multi son dos pasos: crear la tienda (ya activa) y
-- hacer miembro al usuario. La tienda tiene id propio, no el del usuario.
WITH nueva AS (
    INSERT INTO seller (id, rating, store_name, status)
    VALUES (gen_random_uuid(), 0, 'Nombre de la tienda', 1)
    RETURNING id
)
INSERT INTO store_members (store_id, user_id)
SELECT nueva.id, bu.id FROM nueva, base_user bu WHERE bu.username = 'CAMBIAR';

-- 3b. Hacer miembro de una tienda que ya existe a otro usuario
-- INSERT INTO store_members (store_id, user_id)
-- SELECT s.id, bu.id FROM seller s, base_user bu
-- WHERE s.store_name = 'Nombre de la tienda' AND bu.username = 'CAMBIAR';

-- 4. Quitar roles (surte efecto en la siguiente petición: el rol no va en el token)
-- DELETE FROM admins WHERE id = (SELECT id FROM base_user WHERE username = 'CAMBIAR');
-- OJO: borrar de `seller` borra en cascada su product_seller (sus productos se quedan sin dueño).
-- DELETE FROM seller WHERE id = (SELECT id FROM base_user WHERE username = 'CAMBIAR');


--password copy -- only test
UPDATE base_user
  SET password = (SELECT password FROM base_user WHERE username = 'Mox')
  WHERE username = 'Moluxo';


DELETE FROM admins WHERE id = (SELECT id FROM base_user WHERE username = 'Mox');   -- pasa a vendedor sin admin
  INSERT INTO admins (id) SELECT id FROM base_user WHERE username = 'Mox';           -- vuelve a ser admin

  SELECT id, store_name FROM seller;
  UPDATE seller SET store_name = 'Moluxo Store Two' WHERE id = 'd440feaf-f847-418a-a2e3-c90b650e1b38'