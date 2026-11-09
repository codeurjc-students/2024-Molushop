-- Los borrados vuelven a borrador (los dos de antes de esta migración lo eran).
UPDATE products SET status = 0 WHERE status = 3;
ALTER TABLE products DROP CONSTRAINT products_status_check;
ALTER TABLE products ADD CONSTRAINT products_status_check CHECK (status BETWEEN 0 AND 2);
