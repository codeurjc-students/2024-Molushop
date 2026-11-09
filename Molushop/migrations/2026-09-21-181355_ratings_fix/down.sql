-- Vuelve a dejar las dos tablas tal y como las creó la migración inicial
-- (2024-10-03-170959_create_tables, líneas 394-413).

DROP TABLE IF EXISTS ratings;

CREATE TABLE ratings (
    rating_id SERIAL PRIMARY KEY,
    product_id UUID NOT NULL REFERENCES products (id),
    user_id UUID NOT NULL REFERENCES base_user (id),
    rating_value SMALLINT NOT NULL,
    comment TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CHECK (rating_value BETWEEN 1 AND 5)
);

CREATE TABLE IF NOT EXISTS product_summary_ratings (
    product_id UUID PRIMARY KEY REFERENCES products (id),
    star_1_count INTEGER NOT NULL DEFAULT 0,
    star_2_count INTEGER NOT NULL DEFAULT 0,
    star_3_count INTEGER NOT NULL DEFAULT 0,
    star_4_count INTEGER NOT NULL DEFAULT 0,
    star_5_count INTEGER NOT NULL DEFAULT 0,
    total_reviews INTEGER NOT NULL DEFAULT 0,
    average_rating NUMERIC(3, 2) NOT NULL DEFAULT 0.00
);
