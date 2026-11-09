-- Vuelve a dejar la tabla tal y como la creó la migración inicial
-- (2024-10-03-170959_create_tables, líneas 385-392).

DROP TABLE IF EXISTS favorites;

create table favorites(
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES base_user(id) ON DELETE CASCADE,
    product_var_id UUID REFERENCES Product_variations(id) ON DELETE CASCADE,
    added_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    notes VARCHAR(255),
    UNIQUE(user_id, product_var_id)
);
