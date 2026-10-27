-- Favoritos de la tienda. La tabla `favorites` existe desde la migración inicial
-- pero nunca se usó: está vacía y ningún servicio la toca. Como con ratings, se
-- rehace en limpio en lugar de encadenar ALTERs.
--
-- Las decisiones que gobiernan el diseño:
--   1. Se guarda el PRODUCTO, no la variación: el corazón se pulsa también desde
--      las tarjetas de la portada, donde no hay variación elegida; la variación
--      se elige en la ficha al comprar.
--   2. Solo con sesión: user_id obligatorio.
--   3. Un "me gusta" no lleva texto -> fuera `notes`.

DROP TABLE IF EXISTS favorites;

CREATE TABLE favorites (
    id UUID PRIMARY KEY,

    -- CASCADE: un favorito sin usuario o sin producto no significa nada.
    user_id    UUID NOT NULL REFERENCES base_user(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,

    -- Para ordenar la página de favoritos (lo último guardado, primero).
    added_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Guardar dos veces el mismo producto no duplica. Su índice empieza por
    -- user_id, así que sirve también para "los favoritos de este usuario".
    CONSTRAINT uq_favorite_user_product UNIQUE (user_id, product_id)
);
