-- Roles del panel (/panel y /admin). Los roles son tablas, no columnas:
--   admin    = fila en `admins` (acceso global)
--   vendedor = fila en `seller` (gestiona sus productos, vía product_seller)
-- Un usuario puede ser las dos cosas: el admin crea productos a nombre de su tienda.
-- Nunca se da un rol desde la web: siempre a mano con este SQL.

-- 1. Ver quién tiene qué rol
SELECT bu.id, bu.username, bu.email,
       (a.id IS NOT NULL) AS es_admin,
       s.store_name       AS tienda
FROM base_user bu
LEFT JOIN admins a ON a.id = bu.id
LEFT JOIN seller s ON s.id = bu.id
ORDER BY es_admin DESC, tienda NULLS LAST, bu.username;

-- 2. Hacer admin a un usuario (cambiar el username)
INSERT INTO admins (id)
SELECT id FROM base_user WHERE username = 'Moluxo'
ON CONFLICT (id) DO NOTHING;

-- 3. Dar tienda (vendedor) a un usuario (cambiar username y nombre de la tienda)
INSERT INTO seller (id, rating, store_name)
SELECT id, 0, 'Nombre de la tienda' FROM base_user WHERE username = 'CAMBIAR'
ON CONFLICT (id) DO NOTHING;

-- 4. Quitar roles (surte efecto en la siguiente petición: el rol no va en el token)
-- DELETE FROM admins WHERE id = (SELECT id FROM base_user WHERE username = 'CAMBIAR');
-- OJO: borrar de `seller` borra en cascada su product_seller (sus productos se quedan sin dueño).
-- DELETE FROM seller WHERE id = (SELECT id FROM base_user WHERE username = 'CAMBIAR');
