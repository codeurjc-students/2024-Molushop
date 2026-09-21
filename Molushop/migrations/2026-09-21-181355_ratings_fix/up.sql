-- Reseñas de producto (estrella + comentario). La tabla `ratings` existe desde la
-- migración inicial pero nunca se usó: está vacía y ningún servicio la toca. Por eso
-- aquí se rehace en limpio en lugar de encadenar ALTERs sobre un diseño que se quedó
-- a medias.
--
-- Las cuatro decisiones que gobiernan el diseño:
--   1. Una reseña por usuario y producto, editable  -> UNIQUE + updated_at.
--   2. Opinar no exige haber comprado; la etiqueta "compra verificada" se calcula al
--      leer cruzando order_items -> product_variations, así que la BD no guarda nada.
--   3. La media se agrega al vuelo con AVG/COUNT      -> fuera la tabla de caché.
--   4. El usuario puede borrar la suya, borrado duro  -> no hace falta columna status.

DROP TABLE IF EXISTS ratings;

CREATE TABLE ratings (
    -- UUID como el resto del proyecto (orders, favorites, carts). La tabla estaba
    -- vacía, así que cambiar el SERIAL por UUID ahora sale gratis.
    id UUID PRIMARY KEY,

    -- La reseña es del PRODUCTO, no de la variación: se opina del artículo, no del
    -- color que tocó comprar. El carrito y favorites sí van por product_var_id.
    -- CASCADE como en favorites: una reseña sin producto no significa nada.
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    user_id    UUID NOT NULL REFERENCES base_user(id) ON DELETE CASCADE,

    rating_value SMALLINT NOT NULL CHECK (rating_value BETWEEN 1 AND 5),

    -- La estrella es obligatoria; el comentario no. Lo que no se admite es un
    -- comentario en blanco: o es NULL o tiene texto de verdad. El tope de 2000 lo
    -- impone la BD para que no dependa solo de la validación del formulario.
    comment TEXT CHECK (
        comment IS NULL OR char_length(btrim(comment)) BETWEEN 1 AND 2000
    ),

    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- El agujero del diseño viejo: sin esto un usuario podía dejar 50 opiniones del
    -- mismo producto. Con el UNIQUE, "volver a opinar" es editar la que ya hay.
    CONSTRAINT uq_rating_user_product UNIQUE (product_id, user_id)
);

-- update_timestamp() ya existe desde la migración inicial.
CREATE TRIGGER trg_ratings_updated_at
BEFORE UPDATE ON ratings
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- La consulta de la sección de reseñas: las de un producto, de la más nueva a la más
-- vieja. El UNIQUE de arriba ya indexa product_id, pero no en este orden.
CREATE INDEX idx_ratings_product_created ON ratings (product_id, created_at DESC);

-- Abarata el CASCADE al borrar un usuario y la futura pantalla "mis opiniones".
CREATE INDEX idx_ratings_user ON ratings (user_id);


-- product_summary_ratings era una caché de contadores que nadie mantenía. Con la media
-- agregada al vuelo sobraría, y dejarla ahí solo invita a que alguien la lea creyendo
-- que está al día. Si algún día el catálogo pide velocidad, vuelve en su propia
-- migración con el trigger que le faltaba.
DROP TABLE IF EXISTS product_summary_ratings;
