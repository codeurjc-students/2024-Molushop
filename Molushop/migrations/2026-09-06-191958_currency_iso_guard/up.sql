-- La moneda se guarda SIEMPRE como código ISO ("EUR"), nunca como símbolo.
--
-- Sin esto la regla depende de que nadie se equivoque, y equivocarse era fácil:
-- CartItemData.currency contenía el SÍMBOLO (currency_symbol lo traducía al
-- construirlo), así que rellenar el snapshot del pedido desde ahí habría metido
-- "€" en la columna. Cabe en CHAR(3), no da error y se corrompe en silencio.
-- El modelo ya está arreglado; esto es el cinturón para que no vuelva a pasar.
--
-- Sólo se protegen las tablas de pedidos: prices/price_history son de antes y
-- ya están limpias, pero no son lo que se está tocando ahora.

ALTER TABLE orders
    ADD CONSTRAINT ck_orders_currency CHECK (currency ~ '^[A-Z]{3}$');

ALTER TABLE order_items
    ADD CONSTRAINT ck_order_items_currency CHECK (currency ~ '^[A-Z]{3}$');
