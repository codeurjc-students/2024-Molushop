
pub const PRODUCT_URL_PREFIX: &str = "/producto/";
pub const CART_URL: &str = "/cart";
pub const CHECKOUT_URL: &str = "/checkout";
/// Página de un pedido ya confirmado, a donde redirige el POST de confirmación.
pub const ORDER_URL_PREFIX: &str = "/orders/";
/// Lista de "mis pedidos".
pub const ORDERS_URL: &str = "/orders";
/// "Mis favoritos" (plan de favoritos, paso 5).
pub const FAVORITES_URL: &str = "/favorites";
/// "Mi cuenta": cambiar la contraseña y cerrar las demás sesiones.
pub const ACCOUNT_URL: &str = "/account";
/// Portada. Ojo: `nav1_controller::ROUTES` todavía la lleva escrita a mano;
/// unificarlo es parte de la limpieza de rutas pendiente.
pub const HOME_URL: &str = "/home";
/// Búsqueda de la tienda (`?q=` y `?category=`); el buscador de nav1 envía aquí.
pub const SEARCH_URL: &str = "/search";
