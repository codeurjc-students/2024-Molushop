select * from carts;

select * from cart_products;

SELECT * FROM base_user u
  LEFT JOIN carts c               ON c.user_id = u.id AND c.status = 1
  LEFT JOIN cart_products cp      ON cp.cart_id = c.id
  LEFT JOIN product_variations pv ON pv.id = cp.product_var_id
  ORDER BY u.username, cp.added_at;

--Checkear los items del carrito
  SELECT u.username,
         cp.id                        AS linea,
         cp.product_var_id,
         cp.quantity,
         pv.stock,
         cp.price_at_time_of_addition AS precio_guardado
  FROM cart_products cp
  JOIN carts c              ON c.id  = cp.cart_id
  JOIN base_user u          ON u.id  = c.user_id
  JOIN product_variations pv ON pv.id = cp.product_var_id
  WHERE c.status = 1
  ORDER BY u.username, cp.added_at;