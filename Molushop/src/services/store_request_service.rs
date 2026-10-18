//! Solicitud de tienda desde `/panel/register`: la pide el propio vendedor y
//! nace PENDIENTE hasta que un admin la apruebe en `/admin/stores`.
//!
//! Dos caminos: un usuario que ya tiene cuenta pide una tienda, o alguien nuevo
//! crea la cuenta y la tienda a la vez (todo o nada).

use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::models::components::login_base_model::FormRegister;
use crate::schema::{base_user, seller, store_members};
use crate::services::admin_stores_service::clean_store_name;
use crate::services::auth::hash_password;
use crate::services::components::login_base_service::form_register_validation;
use crate::services::panel_role_service::STORE_PENDING;

type DbPool = Pool<AsyncPgConnection>;

/// Por qué no se ha podido solicitar la tienda. Todo menos `Db` es culpa de los
/// datos del formulario y se le explica al usuario.
#[derive(Debug)]
pub enum StoreRequestError {
    InvalidName,
    DuplicateName,
    /// Ya tiene una solicitud esperando: una a la vez, para que nadie llene de
    /// solicitudes la lista del admin.
    AlreadyPending,
    UsernameEmpty,
    EmailInvalid,
    PasswordMismatch,
    PasswordWeak,
    EmailTaken,
    UsernameTaken,
    Db(String),
}

impl StoreRequestError {
    pub fn message(&self) -> &'static str {
        match self {
            StoreRequestError::InvalidName => "El nombre de la tienda no puede estar vacío ni pasar de 255 caracteres.",
            StoreRequestError::DuplicateName => "Ya existe una tienda con ese nombre.",
            StoreRequestError::AlreadyPending => "Ya tienes una tienda pendiente de aprobación. Espera a que se resuelva antes de solicitar otra.",
            StoreRequestError::UsernameEmpty => "El nombre de usuario no puede estar vacío.",
            StoreRequestError::EmailInvalid => "El correo electrónico no es válido.",
            StoreRequestError::PasswordMismatch => "Las contraseñas no coinciden.",
            StoreRequestError::PasswordWeak => "La contraseña necesita al menos 8 caracteres, con mayúscula, minúscula, número y un carácter especial.",
            StoreRequestError::EmailTaken => "Ya hay una cuenta con ese correo electrónico.",
            StoreRequestError::UsernameTaken => "Ese nombre de usuario ya está en uso.",
            StoreRequestError::Db(_) => "Ha ocurrido un error. No se ha guardado nada.",
        }
    }
}

impl From<DieselError> for StoreRequestError {
    fn from(e: DieselError) -> Self {
        match e {
            // El único índice único que puede saltar con los datos ya validados
            // es el del nombre de tienda (`uq_seller_store_name`).
            DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                StoreRequestError::DuplicateName
            }
            other => StoreRequestError::Db(format!("{:?}", other)),
        }
    }
}

async fn insert_pending_store(
    store_name: &str,
    user_id: &Uuid,
    conn: &mut AsyncPgConnection,
) -> Result<Uuid, StoreRequestError> {
    let store_id = Uuid::new_v4();
    diesel::insert_into(seller::table)
        .values((
            seller::id.eq(store_id),
            seller::rating.eq(0),
            seller::store_name.eq(store_name),
            seller::status.eq(STORE_PENDING),
        ))
        .execute(conn)
        .await?;

    diesel::insert_into(store_members::table)
        .values((
            store_members::store_id.eq(store_id),
            store_members::user_id.eq(user_id),
        ))
        .execute(conn)
        .await?;
    Ok(store_id)
}

/// Username de un usuario con sesión, para enseñarlo en el formulario.
pub async fn get_username(user_id: &Uuid, pool: &DbPool) -> Result<String, StoreRequestError> {
    let conn = &mut pool
        .get()
        .await
        .map_err(|e| StoreRequestError::Db(format!("Error obteniendo conexión: {:?}", e)))?;

    Ok(base_user::table
        .filter(base_user::id.eq(user_id))
        .select(base_user::username)
        .first::<String>(conn)
        .await?)
}

/// Un usuario que ya tiene cuenta pide una tienda.
pub async fn request_store_for_user(
    user_id: &Uuid,
    store_name: &str,
    pool: &DbPool,
) -> Result<Uuid, StoreRequestError> {
    let store_name = clean_store_name(store_name).ok_or(StoreRequestError::InvalidName)?;

    let conn = &mut pool
        .get()
        .await
        .map_err(|e| StoreRequestError::Db(format!("Error obteniendo conexión: {:?}", e)))?;

    conn.transaction::<Uuid, StoreRequestError, _>(|conn| async move {
        let already_pending = diesel::select(diesel::dsl::exists(
            store_members::table
                .inner_join(seller::table)
                .filter(store_members::user_id.eq(user_id))
                .filter(seller::status.eq(STORE_PENDING)),
        ))
        .get_result::<bool>(conn)
        .await?;
        if already_pending {
            return Err(StoreRequestError::AlreadyPending);
        }

        insert_pending_store(&store_name, user_id, conn).await
    }.scope_boxed())
    .await
}

/// Alguien sin cuenta se registra como vendedor: usuario + tienda pendiente en
/// una sola transacción. Las reglas de la cuenta son las del registro de la
/// tienda (`form_register_validation`). Devuelve el id del usuario nuevo, para
/// abrirle la sesión.
pub async fn register_seller(
    form: &FormRegister,
    store_name: &str,
    pool: &DbPool,
) -> Result<Uuid, StoreRequestError> {
    let store_name = clean_store_name(store_name).ok_or(StoreRequestError::InvalidName)?;
    if form.username.trim().is_empty() {
        return Err(StoreRequestError::UsernameEmpty);
    }

    let validation = form_register_validation(form, pool)
        .await
        .map_err(|e| StoreRequestError::Db(format!("Error validando el registro: {:?}", e)))?;
    if !validation.email_valid {
        return Err(StoreRequestError::EmailInvalid);
    }
    if !validation.password_match {
        return Err(StoreRequestError::PasswordMismatch);
    }
    if !validation.password_strong {
        return Err(StoreRequestError::PasswordWeak);
    }
    if !validation.username_unique {
        return Err(StoreRequestError::UsernameTaken);
    }
    if !validation.email_unique {
        return Err(StoreRequestError::EmailTaken);
    }

    let hashed_pass = hash_password(&form.pass)
        .map_err(|e| StoreRequestError::Db(format!("Error hasheando la contraseña: {}", e)))?;

    let conn = &mut pool
        .get()
        .await
        .map_err(|e| StoreRequestError::Db(format!("Error obteniendo conexión: {:?}", e)))?;

    conn.transaction::<Uuid, StoreRequestError, _>(|conn| async move {
        let user_id = Uuid::new_v4();
        diesel::insert_into(base_user::table)
            .values((
                base_user::id.eq(user_id),
                base_user::username.eq(&form.username),
                base_user::email.eq(&form.email),
                base_user::password.eq(&hashed_pass),
            ))
            .execute(conn)
            .await
            // Aquí un UNIQUE roto es del usuario (alguien se ha registrado con
            // el mismo nombre entre la validación y el insert), no de la tienda.
            .map_err(|e| match e {
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                    StoreRequestError::UsernameTaken
                }
                other => StoreRequestError::Db(format!("{:?}", other)),
            })?;

        insert_pending_store(&store_name, &user_id, conn).await.map(|_| user_id)
    }.scope_boxed())
    .await
}
