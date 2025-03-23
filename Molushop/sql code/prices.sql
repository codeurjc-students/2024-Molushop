INSERT INTO prices (variation_id, price, currency, start_date)
VALUES ($1, $2, $3, $4)
ON CONFLICT (variation_id, currency) DO UPDATE 
SET price = EXCLUDED.price,
start_date = EXCLUDED.start_date


INSERT INTO prices (variation_id, price, currency, start_date)
VALUES ($1, $2, $3, $4)
ON CONFLICT (variation_id, currency) DO UPDATE 
SET price = EXCLUDED.price,
start_date = EXCLUDED.start_date


select * from prices;
select * from price_history;

DROP table discounts;

select * from discounts;
select * from discount_history;

CREATE OR REPLACE FUNCTION update_discount_history_end_date()
RETURNS TRIGGER AS $$
BEGIN
    -- Actualizar solo el registro más reciente del mismo variation_id, discount_type y currency
    UPDATE discount_history 
    SET end_date = NEW.start_date
    WHERE id = (
        SELECT id 
        FROM discount_history
        WHERE variation_id = NEW.variation_id 
          AND discount_type = NEW.discount_type
          AND currency = NEW.currency
          AND end_date IS NULL
          AND id <> NEW.id  -- No actualizar el registro que acabamos de insertar
        ORDER BY created_at DESC, id DESC  -- Ordenar por fecha de creación y luego por ID
        LIMIT 1              -- Solo obtener el más reciente
    );
      
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER discount_history_update_end_date
AFTER INSERT ON discount_history
FOR EACH ROW
EXECUTE FUNCTION update_discount_history_end_date();