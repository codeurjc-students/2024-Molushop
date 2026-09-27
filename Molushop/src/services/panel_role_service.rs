use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use uuid::Uuid;

use crate::schema::{admins, base_user, seller};

type DbPool = Pool<AsyncPgConnection>;

/// Quién está usando el panel y qué puede hacer en él.
///
/// Los roles no son una columna: ser admin es tener fila en `admins` y ser
/// vendedor es tener fila en `seller` (las dos con PK = `base_user.id`). Un
/// usuario puede ser las dos cosas a la vez: el admin crea productos a nombre
/// de su propia tienda.
#[derive(Debug, Clone)]
pub struct PanelUser {
    pub user_id: Uuid,
    pub is_admin: bool,
    /// `Some` si el usuario tiene tienda. Es el `seller_id` de `product_seller`,
    /// que coincide con el id del usuario.
    pub seller_id: Option<Uuid>,
}

impl PanelUser {
    /// Entra al panel quien sea admin o vendedor. Un comprador sin más no.
    pub fn has_panel_access(&self) -> bool {
        self.is_admin || self.seller_id.is_some()
    }
}

/// Lee los roles de un usuario en una sola consulta.
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

    let (admin_id, seller_id): (Option<Uuid>, Option<Uuid>) = base_user::table
        .left_join(admins::table)
        .left_join(seller::table)
        .filter(base_user::id.eq(user_id))
        .select((admins::id.nullable(), seller::id.nullable()))
        .first(connection)
        .await
        .map_err(|e| format!("Error leyendo los roles del usuario {}: {:?}", user_id, e))?;

    Ok(PanelUser {
        user_id: *user_id,
        is_admin: admin_id.is_some(),
        seller_id,
    })
}
