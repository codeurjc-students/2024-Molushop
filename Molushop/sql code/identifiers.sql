--añadir identificadores a a7b51bc2-b5dd-4bda-b30a-f14c2ef9cd7c
select * from product_variations_identifiers;

INSERT INTO identifiers_var (value) VALUES
    ('SKU'),         -- Stock Keeping Unit
    ('EAN'),         -- European Article Number
    ('UPC'),         -- Universal Product Code
    ('ISBN'),        -- International Standard Book Number
    ('GTIN'),        -- Global Trade Item Number
    ('MPN'),         -- Manufacturer Part Number
    ('ASIN'),        -- Amazon Standard Identification Number
    ('JAN'),         -- Japanese Article Number
    ('ISBN-13'),     -- ISBN de 13 dígitos
    ('ISBN-10');     -- ISBN de 10 dígitos

INSERT INTO product_variations_identifiers (product_variation_id,identifier,value) VALUES
    ('a7b51bc2-b5dd-4bda-b30a-f14c2ef9cd7c','SKU','PAN-VAQ-AZU-32'),
    ('a7b51bc2-b5dd-4bda-b30a-f14c2ef9cd7c','ISBN','978-84-670-2934-4');
