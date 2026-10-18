//! `/admin/stores`: gestión de tiendas, solo para el admin. Se registra dentro
//! del scope `/admin`, así que `PanelGuard::admin()` ya ha comprobado el rol.
//!
//! Son formularios HTML normales: cada POST hace su cambio y redirige (303) al
//! listado con `?ok=` o `?error=`, para que recargar no repita la operación.

use actix_web::{http::header, web, HttpResponse};
use askama::Template;
use serde::Deserialize;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::models::components::panel_nav::PanelNav;
use crate::models::pages_models::admin_stores_page_model::AdminStoresPage;
use crate::services::admin_stores_service::{
    add_member, create_store, delete_store, list_stores, list_usernames, remove_member, set_store_status, AdminStoreError,
};
use crate::services::panel_role_service::PanelUser;

pub static ADMIN_STORES: &str = "/admin/stores";

/// Rutas relativas al scope `/admin`.
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/stores", web::get().to(stores_page))
        .route("/stores", web::post().to(store_create))
        .route("/stores/{store_id}/status", web::post().to(store_status))
        .route("/stores/{store_id}/delete", web::post().to(store_delete))
        .route("/stores/{store_id}/members", web::post().to(member_add))
        .route("/stores/{store_id}/members/{user_id}/remove", web::post().to(member_remove));
}

#[derive(Deserialize)]
pub struct StoresQuery {
    ok: Option<String>,
    error: Option<String>,
}

// Los avisos viajan como códigos y el texto se pone aquí: así la URL no puede
// hacer que la página diga lo que alguien quiera.
fn ok_message(code: &str) -> Option<&'static str> {
    match code {
        "created" => Some("Tienda creada."),
        "status" => Some("Estado de la tienda actualizado."),
        "deleted" => Some("Tienda borrada."),
        "member_added" => Some("Usuario asignado a la tienda."),
        "member_removed" => Some("Usuario quitado de la tienda."),
        _ => None,
    }
}

fn error_message(code: &str) -> Option<&'static str> {
    match code {
        "name" => Some("El nombre de la tienda no puede estar vacío ni pasar de 255 caracteres."),
        "duplicate" => Some("Ya existe una tienda con ese nombre."),
        "user" => Some("No existe ningún usuario con ese nombre de usuario."),
        "store" => Some("La tienda no existe."),
        "transition" => Some("Ese cambio de estado no es posible (puede que la tienda ya haya cambiado)."),
        "products" => Some("No se puede borrar una tienda que tiene productos. Borra antes sus productos o desactívala."),
        "db" => Some("Ha ocurrido un error. No se ha guardado nada."),
        _ => None,
    }
}

fn back(result: Result<&'static str, AdminStoreError>) -> HttpResponse {
    let location = match result {
        Ok(code) => format!("{}?ok={}", ADMIN_STORES, code),
        Err(e) => {
            if let AdminStoreError::Db(detail) = &e {
                println!("Admin tiendas: {}", detail);
            }
            format!("{}?error={}", ADMIN_STORES, e.code())
        }
    };
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, location))
        .finish()
}

async fn stores_page(
    query: web::Query<StoresQuery>,
    panel_user: web::ReqData<PanelUser>,
    pool_data: web::Data<DbPool>,
) -> HttpResponse {
    let stores = match list_stores(pool_data.get_ref()).await {
        Ok(stores) => stores,
        Err(e) => {
            println!("Admin tiendas: {:?}", e);
            return HttpResponse::InternalServerError()
                .content_type("text/html; charset=utf-8")
                .body("Error cargando las tiendas");
        }
    };

    // Sin sugerencias la página sigue sirviendo: se escribe el username a mano.
    let usernames = list_usernames(pool_data.get_ref()).await.unwrap_or_else(|e| {
        println!("Admin tiendas, usernames: {:?}", e);
        Vec::new()
    });

    let query = query.into_inner();
    let page = AdminStoresPage {
        user_logged: true,
        page_name: "Panel - Tiendas".to_string(),
        panel_nav: PanelNav::new(&panel_user, "admin"),
        stores,
        usernames,
        ok_message: query.ok.as_deref().and_then(ok_message),
        error_message: query.error.as_deref().and_then(error_message),
    }.render().unwrap();

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page)
}

#[derive(Deserialize)]
pub struct CreateStoreForm {
    store_name: String,
    /// Opcional: usuario que la va a gestionar.
    username: Option<String>,
}

async fn store_create(form: web::Form<CreateStoreForm>, pool_data: web::Data<DbPool>) -> HttpResponse {
    let form = form.into_inner();
    back(
        create_store(&form.store_name, form.username.as_deref(), pool_data.get_ref())
            .await
            .map(|_| "created"),
    )
}

#[derive(Deserialize)]
pub struct StatusForm {
    status: i16,
}

async fn store_status(
    path: web::Path<Uuid>,
    form: web::Form<StatusForm>,
    pool_data: web::Data<DbPool>,
) -> HttpResponse {
    back(
        set_store_status(&path, form.status, pool_data.get_ref())
            .await
            .map(|_| "status"),
    )
}

async fn store_delete(path: web::Path<Uuid>, pool_data: web::Data<DbPool>) -> HttpResponse {
    back(
        delete_store(&path, pool_data.get_ref())
            .await
            .map(|_| "deleted"),
    )
}

#[derive(Deserialize)]
pub struct MemberForm {
    username: String,
}

async fn member_add(
    path: web::Path<Uuid>,
    form: web::Form<MemberForm>,
    pool_data: web::Data<DbPool>,
) -> HttpResponse {
    back(
        add_member(&path, &form.username, pool_data.get_ref())
            .await
            .map(|_| "member_added"),
    )
}

async fn member_remove(path: web::Path<(Uuid, Uuid)>, pool_data: web::Data<DbPool>) -> HttpResponse {
    let (store_id, user_id) = path.into_inner();
    back(
        remove_member(&store_id, &user_id, pool_data.get_ref())
            .await
            .map(|_| "member_removed"),
    )
}
