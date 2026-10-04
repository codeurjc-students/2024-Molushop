use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use uuid::Uuid;

use crate::schema::{admins, base_user, product_seller, product_variations, products, seller, store_members};

type DbPool = Pool<AsyncPgConnection>;

// Valores de `seller.status` (CHECK 0-3 en la migración `stores_multi`).
pub const STORE_PENDING: i16 = 0;
pub const STORE_ACTIVE: i16 = 1;
pub const STORE_INACTIVE: i16 = 2;
pub const STORE_REJECTED: i16 = 3;

/// Una tienda de la que el usuario es miembro. La tabla se llama `seller`, pero
/// una fila es una tienda: su `id` es el `seller_id` de `product_seller`.
#[derive(Debug, Clone)]
pub struct PanelStore {
    pub id: Uuid,
    pub store_name: String,
    pub status: i16,
}

impl PanelStore {
    /// Activa o inactiva: el vendedor gestiona sus productos (la inactiva no se
    /// ve en la tienda pública, pero se puede preparar). Pendiente o rechazada: no.
    pub fn is_manageable(&self) -> bool {
        self.status == STORE_ACTIVE || self.status == STORE_INACTIVE
    }

    pub fn status_label(&self) -> &'static str {
        match self.status {
            STORE_PENDING => "Pendiente de aprobación",
            STORE_ACTIVE => "Activa",
            STORE_INACTIVE => "Inactiva",
            STORE_REJECTED => "Rechazada",
            _ => "Desconocido",
        }
    }
}

/// Quién está usando el panel y qué puede hacer en él.
///
/// Los roles no son una columna: ser admin es tener fila en `admins` y ser
/// vendedor es ser miembro de alguna tienda (`store_members`). Un usuario puede
/// ser las dos cosas a la vez y gestionar varias tiendas.
#[derive(Debug, Clone)]
pub struct PanelUser {
    pub user_id: Uuid,
    /// Para enseñarlo en la barra del panel.
    pub username: String,
    pub is_admin: bool,
    /// Tiendas de las que es miembro, en cualquier estado, por nombre.
    pub stores: Vec<PanelStore>,
}

impl PanelUser {
    /// Entra al panel quien sea admin o miembro de alguna tienda (aunque esté
    /// pendiente: tiene que poder ver el aviso). Un comprador sin más no.
    pub fn has_panel_access(&self) -> bool {
        self.is_admin || !self.stores.is_empty()
    }

    /// Ids de las tiendas cuyos productos puede gestionar.
    pub fn manageable_store_ids(&self) -> Vec<Uuid> {
        self.stores.iter().filter(|s| s.is_manageable()).map(|s| s.id).collect()
    }
}

/// La tienda de `/panel/stores/{store_id}/...`, si este usuario puede entrar.
///
/// Miembro: solo si la tienda es gestionable (pendiente o rechazada -> `None`).
/// Admin: cualquier tienda que exista, en cualquier estado. `Ok(None)` también
/// si la tienda no existe: el handler responde 404 en todos los casos.
pub async fn resolve_store(
    panel_user: &PanelUser,
    store_id: &Uuid,
    pool: &DbPool,
) -> Result<Option<PanelStore>, String> {
    if let Some(store) = panel_user.stores.iter().find(|s| s.id == *store_id) {
        if store.is_manageable() || panel_user.is_admin {
            return Ok(Some(store.clone()));
        }
        return Ok(None);
    }
    if !panel_user.is_admin {
        return Ok(None);
    }

    let connection = &mut pool
        .get()
        .await
        .map_err(|e| format!("Error obteniendo conexión para la tienda: {:?}", e))?;

    seller::table
        .filter(seller::id.eq(store_id))
        .select((seller::id, seller::store_name, seller::status))
        .first::<(Uuid, String, i16)>(connection)
        .await
        .optional()
        .map(|row| row.map(|(id, store_name, status)| PanelStore { id, store_name, status }))
        .map_err(|e| format!("Error leyendo la tienda {}: {:?}", store_id, e))
}

/// ¿Está el producto a nombre de esta tienda? Para las páginas con la tienda en
/// la URL: un producto de otra tienda es 404 aunque el usuario también la gestione.
pub async fn product_in_store(
    product_id: &Uuid,
    store_id: &Uuid,
    pool: &DbPool,
) -> Result<bool, String> {
    let connection = &mut pool
        .get()
        .await
        .map_err(|e| format!("Error obteniendo conexión para la propiedad: {:?}", e))?;

    diesel::select(diesel::dsl::exists(
        product_seller::table
            .filter(product_seller::product_id.eq(product_id))
            .filter(product_seller::seller_id.eq(store_id)),
    ))
    .get_result::<bool>(connection)
    .await
    .map_err(|e| {
        format!("Error comprobando el producto {} en la tienda {}: {:?}", product_id, store_id, e)
    })
}

/// Lee los roles de un usuario: una consulta para el admin y otra para sus tiendas.
///
/// Se llama en cada petición del panel (no se guarda en el token), así que
/// quitarle a alguien la fila de `admins` o `seller` surte efecto al momento.
///
/// `Err` si falla la BD o el usuario no existe: quien llame tiene que tratarlo
/// como "sin acceso", nunca dejar pasar.
pub async fn get_panel_user(user_id: &Uuid, pool: &DbPool) -> Result<PanelUser, String> {
    let connection = &mut pool
        .get()
        .await
        .map_err(|e| format!("Error obteniendo conexión para los roles: {:?}", e))?;

    let (username, admin_id): (String, Option<Uuid>) = base_user::table
        .left_join(admins::table)
        .filter(base_user::id.eq(user_id))
        .select((base_user::username, admins::id.nullable()))
        .first(connection)
        .await
        .map_err(|e| format!("Error leyendo los roles del usuario {}: {:?}", user_id, e))?;

    let stores = store_members::table
        .inner_join(seller::table)
        .filter(store_members::user_id.eq(user_id))
        .select((seller::id, seller::store_name, seller::status))
        .order_by(seller::store_name)
        .load::<(Uuid, String, i16)>(connection)
        .await
        .map_err(|e| format!("Error leyendo las tiendas del usuario {}: {:?}", user_id, e))?
        .into_iter()
        .map(|(id, store_name, status)| PanelStore { id, store_name, status })
        .collect();

    Ok(PanelUser {
        user_id: *user_id,
        username,
        is_admin: admin_id.is_some(),
        stores,
    })
}

/// ¿Puede este usuario del panel tocar el producto?
///
/// Admin: cualquier producto que exista. Vendedor: solo los que están en
/// `product_seller` a nombre de una de sus tiendas gestionables. `Ok(false)` también si el producto no existe:
/// el handler responde 404 en los dos casos para no revelar productos ajenos.
pub async fn can_edit_product(
    panel_user: &PanelUser,
    product_id: &Uuid,
    pool: &DbPool,
) -> Result<bool, String> {
    let connection = &mut pool
        .get()
        .await
        .map_err(|e| format!("Error obteniendo conexión para la propiedad: {:?}", e))?;

    let query_result = if panel_user.is_admin {
        diesel::select(diesel::dsl::exists(
            products::table.filter(products::id.eq(product_id)),
        ))
        .get_result::<bool>(connection)
        .await
    } else {
        let store_ids = panel_user.manageable_store_ids();
        if store_ids.is_empty() {
            return Ok(false);
        }
        diesel::select(diesel::dsl::exists(
            product_seller::table
                .filter(product_seller::product_id.eq(product_id))
                .filter(product_seller::seller_id.eq_any(store_ids)),
        ))
        .get_result::<bool>(connection)
        .await
    };

    query_result.map_err(|e| {
        format!("Error comprobando la propiedad del producto {}: {:?}", product_id, e)
    })
}

/// Igual que [`can_edit_product`] pero partiendo de una variación: la variación
/// es del dueño de su producto. `Ok(false)` si la variación no existe.
pub async fn can_edit_variation(
    panel_user: &PanelUser,
    variation_id: &Uuid,
    pool: &DbPool,
) -> Result<bool, String> {
    let product_id: Option<Uuid> = {
        let connection = &mut pool
            .get()
            .await
            .map_err(|e| format!("Error obteniendo conexión para la propiedad: {:?}", e))?;

        product_variations::table
            .filter(product_variations::id.eq(variation_id))
            .select(product_variations::product_id)
            .first(connection)
            .await
            .optional()
            .map_err(|e| {
                format!("Error leyendo el producto de la variación {}: {:?}", variation_id, e)
            })?
    };

    match product_id {
        Some(product_id) => can_edit_product(panel_user, &product_id, pool).await,
        None => Ok(false),
    }
}
