# Comprar: ficha, opiniones, carrito y checkout (fase 3 del i18n).

## Átomos: precio y cantidad
price-unavailable = No disponible
qty-label = Cantidad
qty-decrease = Quitar una unidad
qty-increase = Añadir una unidad

## Ficha de producto
product-reviews-count = { $count ->
    [one] 1 opinión
   *[other] { $count } opiniones
}
product-no-reviews = Sin opiniones todavía
product-sold-out = Agotado
product-sold-out-msg = Esta variación está agotada ahora mismo. Elige otra o vuelve más tarde.
product-pick-variation = Selecciona una variación válida.
product-variation-unavailable = Esta variación no está disponible ahora mismo.
product-quantity-positive = La cantidad debe ser mayor que 0.

## Opiniones
reviews-title = Opiniones
reviews-edit = Edita tu opinión
reviews-write = Deja tu opinión
reviews-your-rating = Tu puntuación
reviews-stars = { $count ->
    [one] 1 estrella
   *[other] { $count } estrellas
}
reviews-comment-label = Comentario
reviews-comment-placeholder = Cuéntanos qué te ha parecido (opcional)
reviews-save = Guardar cambios
reviews-publish = Publicar opinión
reviews-buyers-only = Solo pueden opinar quienes ya han recibido este producto.
reviews-login-prompt = Identifícate para dejar tu opinión.
reviews-login-button = Identificarme
reviews-empty = Este producto todavía no tiene opiniones.
reviews-average-total = sobre { $count ->
    [one] 1 opinión
   *[other] { $count } opiniones
}
reviews-mine = Tu opinión
reviews-more = Ver más opiniones ({ $shown } de { $total })
reviews-delete = Borrar mi opinión
reviews-delete-confirm = ¿Borrar tu opinión? No se puede deshacer.
reviews-error-no-rating = Elige cuántas estrellas le das.
reviews-error-rating-range = La puntuación tiene que ir de 1 a 5 estrellas.
reviews-error-too-long = El comentario no puede pasar de { $max } caracteres.
reviews-error-product-gone = Este producto ya no está disponible.
reviews-error-save = No se ha podido guardar tu opinión. Inténtalo otra vez.
reviews-error-delete = No se ha podido borrar tu opinión. Inténtalo otra vez.

## Carrito
cart-page-title = Cesta | Molushop
cart-title = Cesta
cart-select-all = Seleccionar todos los artículos
cart-empty-title = Tu cesta está vacía
cart-empty-text = Añade productos desde la tienda y aparecerán aquí.
cart-line-gone = Ya no está a la venta. Quítalo de la cesta.
cart-summary = Resumen
cart-items-total = Total de artículos
cart-discount = Descuento de artículos
cart-shipping = Envío
cart-shipping-free = Gratis
cart-estimated-total = Estimación total
cart-vat-included = IVA incluido
cart-continue = Continuar
cart-added = Producto añadido al carrito.
cart-error-variation = No se ha encontrado esa variación.
cart-error-stock = No hay suficiente stock.
cart-error-add = No se ha podido añadir al carrito. Inténtalo de nuevo.
cart-error-line-gone = El producto ya no está en tu cesta.
cart-error-update = No se ha podido modificar la cesta. Inténtalo de nuevo.

## Checkout
checkout-page-title = Finalizar compra | Molushop
checkout-title = Finalizar compra
checkout-empty = No hay artículos que comprar.
checkout-back-to-cart = Volver a la cesta
checkout-address = Dirección de envío
checkout-street1 = Dirección
checkout-street2 = Dirección 2
checkout-optional = (opcional)
checkout-postal-code = Código postal
checkout-city = Ciudad
checkout-province = Provincia
checkout-items = Artículos
checkout-confirm = Confirmar pedido
checkout-sending = Confirmando pedido…
checkout-error-address = Faltan datos de la dirección de envío.
checkout-error-stock-low = Se ha quedado sin stock suficiente: de «{ $product }» sólo quedan { $available }. Ajusta la cantidad en la cesta.
checkout-error-stock-out = «{ $product }» se ha quedado sin stock mientras completabas el pedido.
checkout-error-unavailable = «{ $product }» ya no está disponible. Quítalo de la cesta para completar el pedido. No se ha cobrado nada.
checkout-error-price = Uno de los artículos no tiene precio disponible ahora mismo. No se ha cobrado nada.
checkout-error = No se ha podido completar el pedido. No se ha cobrado nada; inténtalo de nuevo.
