//! `products.status`. Solo se vende lo ACTIVO (la home, la ficha, el carrito y
//! el checkout lo comprueban).

pub const PRODUCT_DRAFT: i16 = 0;
pub const PRODUCT_ACTIVE: i16 = 1;
pub const PRODUCT_INACTIVE: i16 = 2;
/// "Eliminar" del panel. No se borra la fila: los pedidos (`order_items`) y las
/// opiniones siguen apuntando a sus variaciones. Desaparece de la tienda y del
/// panel, y no se puede volver a editar.
pub const PRODUCT_DELETED: i16 = 3;
