UPDATE products 
SET images = jsonb_set(
images,
'{images}',
(images->'images') || '[{
    "tipo": "principal",
    "url": "https://ejemplo.com/principal.jpg"
}]'::jsonb
)
WHERE id = '2fcddbef-1c29-4601-8879-8671bc77160b';