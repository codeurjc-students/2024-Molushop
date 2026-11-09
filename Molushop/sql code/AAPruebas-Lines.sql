SELECT cp.id, cp.quantity, c.user_id
  FROM cart_products cp JOIN carts c ON c.id = cp.cart_id
  WHERE c.user_id = '703cb39d-358c-4b13-82ee-382f11193136' AND c.status = 1;

  SELECT max(order_number) FROM orders;
  SELECT pv.id, pv.stock FROM product_variations pv
  WHERE pv.id IN (SELECT product_var_id FROM cart_products cp
                  JOIN carts c ON c.id = cp.cart_id WHERE c.status = 1);