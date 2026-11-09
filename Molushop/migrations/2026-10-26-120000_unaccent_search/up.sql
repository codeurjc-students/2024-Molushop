-- Búsqueda de la tienda sin tener en cuenta los acentos: "camara" encuentra
-- "Cámara" (y la ñ cuenta como n). unaccent viene con Postgres y es "trusted"
-- (Postgres 13+): la puede instalar el dueño de la base de datos.
CREATE EXTENSION IF NOT EXISTS unaccent;

-- unaccent() no es IMMUTABLE (depende del diccionario y del search_path), así
-- que no vale para un índice. Con el diccionario fijado sí lo es: las consultas
-- usan esta función para que un índice futuro (pg_trgm sobre f_unaccent(name))
-- no obligue a cambiarlas.
CREATE FUNCTION f_unaccent(text) RETURNS text
    LANGUAGE sql IMMUTABLE PARALLEL SAFE STRICT
    AS $$ SELECT public.unaccent('public.unaccent'::regdictionary, $1) $$;
