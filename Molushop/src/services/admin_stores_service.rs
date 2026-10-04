//! Gestión de tiendas desde `/admin/stores`: listar, crear, cambiar el estado y
//! asignar o quitar usuarios. Todo esto es exclusivo del admin; quien llama ya
//! ha pasado por `PanelGuard::admin()`.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::schema::{base_user, product_seller, seller, store_members};
use crate::services::panel_role_service::{
    PanelStore, STORE_ACTIVE, STORE_INACTIVE, STORE_PENDING, STORE_REJECTED,
};

type DbPool = Pool<AsyncPgConnection>;

/// Tope de `seller.store_name` (VARCHAR(255)).
const STORE_NAME_MAX: usize = 255;

/// Por qué no se ha podido hacer una operación. Los cuatro primeros son fallos
/// del admin (se le explican); `Db` es un fallo nuestro (se registra).
#[derive(Debug)]
pub enum AdminStoreError {
    /// Nombre vacío o demasiado largo.
    InvalidName,
    /// Ya hay una tienda con ese nombre (índice `uq_seller_store_name`).
    DuplicateName,
    /// No existe ningún usuario con ese username.
    UserNotFound,
    /// La tienda no existe.
    StoreNotFound,
    /// El cambio de estado no tiene sentido (p. ej. desactivar una pendiente).
    InvalidTransition,
    /// No se borra una tienda que todavía tiene productos.
    HasProducts,
    Db(String),
}

impl AdminStoreError {
    /// Código corto para llevar el error en la URL tras el POST (`?error=...`).
    pub fn code(&self) -> &'static str {
        match self {
            AdminStoreError::InvalidName => "name",
            AdminStoreError::DuplicateName => "duplicate",
            AdminStoreError::UserNotFound => "user",
            AdminStoreError::StoreNotFound => "store",
            AdminStoreError::InvalidTransition => "transition",
            AdminStoreError::HasProducts => "products",
            AdminStoreError::Db(_) => "db",
        }
    }
}

impl From<DieselError> for AdminStoreError {
    fn from(e: DieselError) -> Self {
        match e {
            DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                AdminStoreError::DuplicateName
            }
            other => AdminStoreError::Db(format!("{:?}", other)),
        }
    }
}

async fn connection(
    pool: &DbPool,
) -> Result<diesel_async::pooled_connection::deadpool::Object<AsyncPgConnection>, AdminStoreError> {
    pool.get()
        .await
        .map_err(|e| AdminStoreError::Db(format!("Error obteniendo conexión: {:?}", e)))
}

#[derive(Debug, Clone)]
pub struct StoreMember {
    pub user_id: Uuid,
    pub username: String,
}

/// Una fila de `/admin/stores`.
#[derive(Debug, Clone)]
pub struct AdminStore {
    pub store: PanelStore,
    /// Fecha de alta, ya formateada (dd/mm/aaaa).
    pub created: String,
    pub members: Vec<StoreMember>,
}

/// Todas las tiendas con sus miembros. Las pendientes primero (son las que
/// esperan al admin) y, dentro de cada estado, las más antiguas arriba.
pub async fn list_stores(pool: &DbPool) -> Result<Vec<AdminStore>, AdminStoreError> {
    let conn = &mut connection(pool).await?;

    let stores = seller::table
        .select((seller::id, seller::store_name, seller::status, seller::created_at))
        .order_by((seller::status, seller::created_at))
        .load::<(Uuid, String, i16, DateTime<Utc>)>(conn)
        .await?;

    let members = store_members::table
        .inner_join(base_user::table)
        .select((store_members::store_id, base_user::id, base_user::username))
        .order_by(base_user::username)
        .load::<(Uuid, Uuid, String)>(conn)
        .await?;

    Ok(stores
        .into_iter()
        .map(|(id, store_name, status, created_at)| AdminStore {
            members: members
                .iter()
                .filter(|(store_id, _, _)| *store_id == id)
                .map(|(_, user_id, username)| StoreMember {
                    user_id: *user_id,
                    username: username.clone(),
                })
                .collect(),
            store: PanelStore { id, store_name, status },
            created: created_at.format("%d/%m/%Y").to_string(),
        })
        .collect())
}

/// El nombre sin espacios a los lados, o `None` si queda vacío o no cabe.
pub fn clean_store_name(store_name: &str) -> Option<String> {
    let store_name = store_name.trim();
    if store_name.is_empty() || store_name.chars().count() > STORE_NAME_MAX {
        return None;
    }
    Some(store_name.to_string())
}

async fn find_user_id(username: &str, conn: &mut AsyncPgConnection) -> Result<Uuid, AdminStoreError> {
    base_user::table
        .filter(base_user::username.eq(username))
        .select(base_user::id)
        .first::<Uuid>(conn)
        .await
        .optional()?
        .ok_or(AdminStoreError::UserNotFound)
}

/// Crea una tienda ya ACTIVA (la crea el admin: no hay nada que aprobar) y, si
/// se indica, hace miembro al usuario. Todo o nada: con un username que no
/// existe no queda la tienda creada.
pub async fn create_store(
    store_name: &str,
    username: Option<&str>,
    pool: &DbPool,
) -> Result<Uuid, AdminStoreError> {
    let store_name = clean_store_name(store_name).ok_or(AdminStoreError::InvalidName)?;
    let username = username.map(|u| u.trim().to_string()).filter(|u| !u.is_empty());

    let conn = &mut connection(pool).await?;
    conn.transaction::<Uuid, AdminStoreError, _>(|conn| async move {
        let user_id = match &username {
            Some(username) => Some(find_user_id(username, conn).await?),
            None => None,
        };

        let store_id = Uuid::new_v4();
        diesel::insert_into(seller::table)
            .values((
                seller::id.eq(store_id),
                seller::rating.eq(0),
                seller::store_name.eq(&store_name),
                seller::status.eq(STORE_ACTIVE),
            ))
            .execute(conn)
            .await?;

        if let Some(user_id) = user_id {
            diesel::insert_into(store_members::table)
                .values((
                    store_members::store_id.eq(store_id),
                    store_members::user_id.eq(user_id),
                ))
                .execute(conn)
                .await?;
        }
        Ok(store_id)
    }.scope_boxed())
    .await
}

/// Cambios de estado que puede hacer el admin:
///   pendiente -> activa (aprobar) o rechazada
///   rechazada -> activa (aprobar al final)
///   activa    -> inactiva (desactivar)
///   inactiva  -> activa (reactivar)
fn transition_allowed(from: i16, to: i16) -> bool {
    matches!(
        (from, to),
        (STORE_PENDING, STORE_ACTIVE)
            | (STORE_PENDING, STORE_REJECTED)
            | (STORE_REJECTED, STORE_ACTIVE)
            | (STORE_ACTIVE, STORE_INACTIVE)
            | (STORE_INACTIVE, STORE_ACTIVE)
    )
}

pub async fn set_store_status(
    store_id: &Uuid,
    new_status: i16,
    pool: &DbPool,
) -> Result<(), AdminStoreError> {
    let conn = &mut connection(pool).await?;

    let current = seller::table
        .filter(seller::id.eq(store_id))
        .select(seller::status)
        .first::<i16>(conn)
        .await
        .optional()?
        .ok_or(AdminStoreError::StoreNotFound)?;

    if !transition_allowed(current, new_status) {
        return Err(AdminStoreError::InvalidTransition);
    }

    // El filtro por el estado leído evita pisar un cambio hecho entre medias
    // (dos admins, o un doble clic): si ya no está como se leyó, no se toca.
    let updated = diesel::update(
        seller::table
            .filter(seller::id.eq(store_id))
            .filter(seller::status.eq(current)),
    )
    .set(seller::status.eq(new_status))
    .execute(conn)
    .await?;

    if updated == 0 {
        return Err(AdminStoreError::InvalidTransition);
    }
    Ok(())
}

/// Borra la tienda y, en cascada, sus miembros. Solo si no tiene productos:
/// borrarla con productos los dejaría en `products` sin dueño (la cascada solo
/// se lleva `product_seller`). Para retirar una tienda con catálogo está
/// "desactivar", que no pierde nada.
pub async fn delete_store(store_id: &Uuid, pool: &DbPool) -> Result<(), AdminStoreError> {
    let conn = &mut connection(pool).await?;

    conn.transaction::<(), AdminStoreError, _>(|conn| async move {
        let has_products = diesel::select(diesel::dsl::exists(
            product_seller::table.filter(product_seller::seller_id.eq(store_id)),
        ))
        .get_result::<bool>(conn)
        .await?;
        if has_products {
            return Err(AdminStoreError::HasProducts);
        }

        let deleted = diesel::delete(seller::table.filter(seller::id.eq(store_id)))
            .execute(conn)
            .await?;
        if deleted == 0 {
            return Err(AdminStoreError::StoreNotFound);
        }
        Ok(())
    }.scope_boxed())
    .await
}

/// Hace miembro de la tienda al usuario. Si ya lo era, no pasa nada.
pub async fn add_member(store_id: &Uuid, username: &str, pool: &DbPool) -> Result<(), AdminStoreError> {
    let conn = &mut connection(pool).await?;

    let exists = diesel::select(diesel::dsl::exists(seller::table.filter(seller::id.eq(store_id))))
        .get_result::<bool>(conn)
        .await?;
    if !exists {
        return Err(AdminStoreError::StoreNotFound);
    }

    let user_id = find_user_id(username.trim(), conn).await?;

    diesel::insert_into(store_members::table)
        .values((
            store_members::store_id.eq(store_id),
            store_members::user_id.eq(user_id),
        ))
        .on_conflict_do_nothing()
        .execute(conn)
        .await?;
    Ok(())
}

/// Quita al usuario de la tienda. La tienda y sus productos no se tocan: una
/// tienda sin miembros la sigue gestionando el admin.
pub async fn remove_member(store_id: &Uuid, user_id: &Uuid, pool: &DbPool) -> Result<(), AdminStoreError> {
    let conn = &mut connection(pool).await?;

    diesel::delete(
        store_members::table
            .filter(store_members::store_id.eq(store_id))
            .filter(store_members::user_id.eq(user_id)),
    )
    .execute(conn)
    .await?;
    Ok(())
}
