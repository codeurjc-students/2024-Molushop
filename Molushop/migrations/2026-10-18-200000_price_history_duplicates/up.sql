-- Hasta el 2026-10-11 `update_price_variation` insertaba en `price_history` a
-- mano además del trigger `price_history_trigger`, y cada cambio de precio
-- dejaba una fila de más: mismo precio, abierta y cerrada en el mismo instante
-- (start_date = end_date) y a menos de 2 s de la buena. Se borran esas.
DELETE FROM price_history dup
WHERE dup.end_date = dup.start_date
  AND EXISTS (
      SELECT 1 FROM price_history ok
      WHERE ok.id <> dup.id
        AND ok.variation_id = dup.variation_id
        AND ok.currency = dup.currency
        AND ok.price = dup.price
        AND abs(extract(epoch FROM ok.start_date - dup.start_date)) < 2
        AND ok.end_date IS DISTINCT FROM ok.start_date
  );
