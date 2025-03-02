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