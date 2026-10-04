-- Tiendas separadas de usuarios, con estado de aprobación.
--
-- Hasta ahora la tienda ERA el usuario: `seller.id` = `base_user.id` (PK y FK). Eso
-- obliga a una tienda por usuario y un usuario por tienda. Las decisiones del
-- 2026-10-04 que gobiernan este diseño:
--   1. Un usuario puede gestionar varias tiendas y una tienda puede tener varios
--      usuarios                                   -> tabla de pertenencia `store_members`.
--   2. Una tienda nace PENDIENTE y la aprueba un admin; solo un admin la desactiva
--                                                 -> columna `seller.status`.
--   3. Los productos de una tienda que no está ACTIVA no se ven en la tienda pública
--      (lo filtra el código al leer; la BD no guarda nada más).
--
-- La tabla sigue llamándose `seller` a propósito: `product_seller.seller_id` y todo
-- el código la referencian con ese nombre, y renombrarla no aporta nada hoy. A partir
-- de aquí una fila de `seller` es UNA TIENDA, no un usuario.

-- 1. La tienda deja de ser un usuario. Los ids actuales se conservan (siguen
--    coincidiendo con el id de su dueño, pero ya es casualidad): `product_seller` y
--    los pedidos no se tocan. Las tiendas nuevas tendrán un UUID propio.
ALTER TABLE seller DROP CONSTRAINT fk_user_seller;

-- 2. Estado de la tienda. Un solo campo en vez de "aprobada" + "activa" para que no
--    existan combinaciones sin sentido (rechazada pero activa).
--      0: PENDING   recién solicitada; el vendedor la ve con el aviso, no gestiona productos
--      1: ACTIVE    aprobada; sus productos se ven en la tienda pública
--      2: INACTIVE  desactivada por un admin; el vendedor entra y edita, el público no la ve
--      3: REJECTED  el admin no la aprueba
--    DEFAULT 0: cualquier alta que se olvide del estado queda pendiente, nunca visible.
ALTER TABLE seller
    ADD COLUMN status SMALLINT NOT NULL DEFAULT 0 CHECK (status BETWEEN 0 AND 3),
    ADD COLUMN created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW();

-- Las tiendas que ya existen estaban funcionando: quedan activas.
UPDATE seller SET status = 1;

-- Dos tiendas con el mismo nombre (sin distinguir mayúsculas ni espacios de los
-- lados) confunden al comprador y al admin que las aprueba.
CREATE UNIQUE INDEX uq_seller_store_name ON seller (lower(btrim(store_name)));

-- 3. Quién gestiona qué tienda. Sin roles dentro de la tienda (dueño/empleado):
--    todos los miembros pueden lo mismo. Si hace falta, se añade una columna.
CREATE TABLE store_members (
    store_id UUID NOT NULL REFERENCES seller(id) ON DELETE CASCADE,
    user_id  UUID NOT NULL REFERENCES base_user(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (store_id, user_id)
);

-- "Mis tiendas" se busca por usuario en cada petición del panel; la PK empieza por
-- store_id y no sirve para eso.
CREATE INDEX idx_store_members_user ON store_members (user_id);

-- Cada tienda existente pasa a tener como miembro al usuario que la era.
INSERT INTO store_members (store_id, user_id)
SELECT id, id FROM seller;
