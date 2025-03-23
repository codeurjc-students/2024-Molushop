select * from product_variations join prices on 

SELECT pv.*, p.price, p.currency, p.start_date
FROM product_variations pv
JOIN prices p ON pv.id = p.variation_id
WHERE p.currency = 'EUR';