-- Pedidos. Hasta ahora la compra terminaba en el carrito y no había dónde
-- escribirla; estas dos tablas son la base del checkout (paso 6).
--
-- La idea que gobierna todo el diseño: un pedido es un registro contable, no una
-- vista del catálogo. Por eso guarda copia de lo que se vendió y a dónde se
-- envió, y por eso no deja que se borre lo que referencia.

CREATE TABLE orders (
    id UUID PRIMARY KEY,

    -- Referencia legible para el cliente. La genera la BD; el código nunca la escribe.
    order_number BIGSERIAL NOT NULL UNIQUE,

    -- RESTRICT y no CASCADE como el resto del proyecto: borrar un usuario no
    -- puede llevarse por delante sus pedidos. Si algún día hace falta borrar
    -- usuarios de verdad, es una migración de una línea.
    user_id UUID NOT NULL REFERENCES base_user(id) ON DELETE RESTRICT,

    -- Trazabilidad al carrito de origen, que al confirmar pasa a INACTIVE (2).
    cart_id UUID REFERENCES carts(id) ON DELETE SET NULL,

    status SMALLINT NOT NULL DEFAULT 1 CHECK (status BETWEEN 0 AND 2), -- 0: PENDING, 1: CONFIRMED, 2: CANCELLED

    currency CHAR(3) NOT NULL,

    -- Importes congelados. El total es el que se cobró: NO se recalcula al
    -- mostrar el pedido, aunque el precio del catálogo cambie después.
    -- discount_total y shipping_total nacen a 0 (hoy no hay descuentos ni
    -- gastos de envío), pero existen para que el total cuadre sin inventar nada.
    subtotal       DECIMAL(10,2) NOT NULL CHECK (subtotal >= 0),
    discount_total DECIMAL(10,2) NOT NULL DEFAULT 0 CHECK (discount_total >= 0),
    shipping_total DECIMAL(10,2) NOT NULL DEFAULT 0 CHECK (shipping_total >= 0),
    total_amount   DECIMAL(10,2) NOT NULL CHECK (total_amount >= 0),

    -- La dirección se COPIA. El id es solo referencia y puede quedar en NULL:
    -- el usuario puede editar o borrar su dirección después de comprar y el
    -- pedido tiene que seguir sabiendo a dónde se envió.
    -- Los tipos son los mismos que en customer_address.
    shipping_address_id UUID REFERENCES customer_address(id) ON DELETE SET NULL,
    ship_street1     VARCHAR(255) NOT NULL,
    ship_street2     VARCHAR(255),
    ship_postal_code VARCHAR(20)  NOT NULL,
    ship_city        VARCHAR(255) NOT NULL,
    ship_province    VARCHAR(255) NOT NULL,

    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- update_timestamp() ya existe desde la migración inicial.
CREATE TRIGGER trg_orders_updated_at
BEFORE UPDATE ON orders
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- "Mis pedidos": los del usuario, del más nuevo al más viejo.
CREATE INDEX idx_orders_user_created ON orders (user_id, created_at DESC);


CREATE TABLE order_items (
    id UUID PRIMARY KEY,

    -- Las líneas no significan nada sin su pedido: aquí sí CASCADE.
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,

    -- Pero borrar una variación ya vendida dejaría el pedido incompleto.
    product_var_id UUID NOT NULL REFERENCES product_variations(id) ON DELETE RESTRICT,

    -- Copia de lo que se vendió, tal y como se veía en el carrito. Se rellena
    -- con las mismas expresiones del SELECT de get_cart_items (string_agg de
    -- attributes, COALESCE de la imagen, tienda por LATERAL).
    product_name    VARCHAR(100) NOT NULL,  -- mismo tipo que products.name
    variation_label VARCHAR(255),           -- "Verde / S / Plástico"
    sku       TEXT,
    image_url TEXT,                         -- mismo tipo que images_product.image_url
    store_name VARCHAR(255),

    -- Aquí es donde se congela el precio, no en cart_products: el
    -- price_at_time_of_addition del carrito se sobrescribe en cada adición.
    unit_price DECIMAL(10,2) NOT NULL CHECK (unit_price >= 0),
    currency   CHAR(3) NOT NULL,
    quantity   INT NOT NULL CHECK (quantity > 0),
    line_total DECIMAL(10,2) NOT NULL CHECK (line_total >= 0),

    -- Espejo de uq_cart_product: una línea por variación y pedido.
    CONSTRAINT uq_order_item UNIQUE (order_id, product_var_id)
);

-- No hace falta índice por order_id: el UNIQUE de arriba ya crea uno con
-- order_id como primera columna y sirve para la FK.
-- Este sí hace falta: abarata la comprobación del RESTRICT al borrar una variación.
CREATE INDEX idx_order_items_variation ON order_items (product_var_id);
