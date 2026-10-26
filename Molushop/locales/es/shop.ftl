# Textos de la tienda en español (el idioma por defecto: si a otro idioma le
# falta una clave, sale esta). Ver src/i18n.rs.
# Sin HTML: si una frase lleva negrita o un enlace, se parte en trozos.

## Comunes
categories = Categorías
products = Productos
all-products = Todos los productos
see-all-products = Ver todos los productos

## Barra (nav1)
nav-search-placeholder = Busca productos o marcas
nav-search-label = Buscar productos
nav-search-submit = Buscar
nav-cart = Carrito
nav-profile = Perfil
nav-orders = Mis pedidos
nav-account = Mi cuenta
nav-logout = Cerrar sesión
nav-login = Identifícate
nav-register = Registrarse

## Sugerencias del buscador
suggest-label = Sugerencias de búsqueda
suggest-all = Ver todos los resultados de “{ $q }”
suggest-empty = No hay productos para “{ $q }”

## Pie (footer1) y selector de idioma
footer-tagline = Productos de tiendas independientes, en un solo carrito.
footer-shop = Tienda
footer-home = Inicio
footer-about = Sobre nosotros
footer-sell = Vende en Molushop
lang-label = Idioma

## Portada
home-page-title = Inicio | Molushop
home-hero-title = Todo lo que buscas, de tiendas independientes
home-hero-text = Compra a varias tiendas a la vez y paga en un solo carrito.
home-hero-cta = Ver productos

## Tarjetas de producto
card-add-to-cart = Añadir al carrito
card-unavailable = No disponible
card-load-more = Cargar más productos
# $rating ya viene escrito con la coma o el punto del idioma ("4,5").
stars-label = { $rating } de 5 estrellas

## Búsqueda
search-title-query = Resultados de “{ $q }”
search-title-query-category = “{ $q }” en { $category }
search-breadcrumb-label = Categoría
search-all-categories = Todas
search-clear-query = Quitar la búsqueda
search-filter-label = Filtrar por categoría
search-empty-title = No hemos encontrado productos
search-empty-text = Prueba con otras palabras o con menos filtros.

## Página no encontrada (404)
notfound-page-title = Página no encontrada
notfound-title = Esta página no existe
notfound-text = Puede que el enlace no sea correcto o que la página ya no esté.
product-notfound-page-title = Producto no disponible
product-notfound-title = Este producto no está disponible
product-notfound-text = Puede que ya no esté a la venta o que el enlace no sea correcto.

## Sobre nosotros
about-page-title = Sobre nosotros
about-title = Sobre nosotros
# Cada párrafo con negrita va en tres trozos: antes, la negrita y después.
about-p1-start = ¡Bienvenido a Molushop, tu destino definitivo para encontrar
about-p1-strong = productos únicos y especiales
about-p1-end = que te encantarán!
about-p2-start = En Molushop, creemos que cada objeto tiene una historia y puede añadir un toque de magia a tu día a día. Somos una tienda online dedicada a buscar y ofrecerte una
about-p2-strong = cuidadosa selección de artículos
about-p2-end = que combinan calidad, originalidad y estilo. Desde decoraciones para tu hogar que transforman espacios, hasta accesorios que realzan tu personalidad, cada producto en nuestro catálogo es elegido pensando en ti.
about-p3-start = Nuestra pasión es descubrir esas piezas que no encuentras en cualquier lugar, las que te hacen sonreír y las que convierten un regalo en algo inolvidable. Trabajamos directamente con
about-p3-strong = artesanos y proveedores cuidadosamente seleccionados
about-p3-end = para asegurar que cada compra en Molushop sea una experiencia satisfactoria y que recibas productos de la más alta calidad.
about-p4 = En Molushop, no solo vendemos productos, creamos conexiones. Queremos ser parte de tus momentos especiales, ayudarte a expresar tu creatividad y ofrecerte siempre algo nuevo y emocionante.
about-p5 = Gracias por elegir Molushop. ¡Esperamos que disfrutes explorando nuestra tienda tanto como nosotros disfrutamos creándola para ti!

## Solo para los tests de src/i18n.rs: comprueban los plurales.
i18n-test-count = { $count ->
    [one] { $count } producto
   *[other] { $count } productos
}
