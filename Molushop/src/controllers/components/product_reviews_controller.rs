use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::{delete, get, post, web, HttpMessage, HttpRequest, HttpResponse, Scope};
use lazy_static::lazy_static;
use serde::Deserialize;
use uuid::Uuid;

use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
type DbPool = Pool<AsyncPgConnection>;

use super::scope::SCOPE_COMPONENTS;
use crate::middleware::auth::{Auth, SessionData};
use crate::models::components::product_reviews_v1::{ProductReviewsRoutes, MAX_COMMENT_LEN};
use crate::models::error::ServiceError;
use crate::services::components::product_reviews_service::{
    delete_review, get_product_reviews_body_render, save_review,
};

pub static SCOPE: &str = "/product-reviews";

lazy_static! {
    pub static ref ROUTES: ProductReviewsRoutes = ProductReviewsRoutes {
        list: format!("{}{}/list", SCOPE_COMPONENTS, SCOPE),
        save: format!("{}{}/save", SCOPE_COMPONENTS, SCOPE),
        delete: format!("{}{}/delete", SCOPE_COMPONENTS, SCOPE),
    };
}

/// `Auth` no rechaza a nadie: solo mete la `SessionData` si la cookie vale. Por
/// eso puede envolver el scope entero aunque leer las opiniones sea público —
/// el GET la usa nada más que para saber si pinta el formulario o el botón de
/// identificarse, y el POST sí la necesita de verdad.
pub fn scope() -> Scope<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    web::scope(SCOPE).wrap(Auth::new()).configure(config)
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list_reviews);
    cfg.service(save_product_review);
    cfg.service(delete_product_review);
}

fn session_user_id(req: &HttpRequest) -> Option<Uuid> {
    req.extensions().get::<SessionData>().cloned().map(|s| s.id)
}

/// El hueco de errores que ya está en el formulario, con el aviso dentro. El
/// formulario lo pide con `hx-target-400`/`hx-target-500`, así que el fallo se
/// queda donde el usuario está escribiendo en vez de repintar la sección.
fn form_error(message: &str) -> String {
    format!(
        "<div class=\"form-error\" id=\"review-form-error\">{}</div>",
        message
    )
}

#[derive(Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
}

/// El "ver más": devuelve la sección repintada con una página más de opiniones.
/// Es el cuerpo del componente, sin el wrapper, porque sustituye al que ya está
/// puesto en la ficha.
#[get("/list/{product_id}")]
async fn list_reviews(
    path: web::Path<Uuid>,
    query: web::Query<PageQuery>,
    pool_data: web::Data<DbPool>,
    req: HttpRequest,
) -> HttpResponse {
    let product_id = path.into_inner();
    let page = query.page.unwrap_or(1);
    let user_id = session_user_id(&req);
    let body = get_product_reviews_body_render(&product_id, page, user_id.as_ref(), pool_data.get_ref()).await;

    HttpResponse::Ok().body(body)
}

#[derive(Deserialize)]
pub struct ReviewForm {
    pub product_id: Uuid,
    /// `Option` para que "no he marcado ninguna estrella" sea un error nuestro
    /// con su aviso, y no el 400 pelado de la deserialización del formulario.
    pub rating_value: Option<i16>,
    pub comment: Option<String>,
}

/// Publica (o corrige) la opinión del usuario y devuelve la sección repintada,
/// ya con la suya dentro y la media recalculada.
#[post("/save")]
async fn save_product_review(
    form: web::Form<ReviewForm>,
    pool_data: web::Data<DbPool>,
    req: HttpRequest,
) -> HttpResponse {
    // Mismo 401 que el carrito: el `HX-Trigger` lo recoge login_modal.js y abre
    // el login. htmx no cambia nada de la página con un 4xx sin `hx-target-*`,
    // así que lo escrito sigue en su sitio.
    let user_id = match session_user_id(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized()
                .insert_header(("HX-Trigger", "auth-required"))
                .body("No autenticado");
        }
    };

    let form = form.into_inner();
    let pool = pool_data.get_ref();

    let rating_value = match form.rating_value {
        Some(v) => v,
        None => return HttpResponse::BadRequest().body(form_error("Elige cuántas estrellas le das.")),
    };

    match save_review(&user_id, &form.product_id, rating_value, form.comment, pool).await {
        Ok(()) => {
            // Vuelve a la primera página: la opinión recién escrita es la más
            // reciente, así que sale arriba del todo.
            let body = get_product_reviews_body_render(&form.product_id, 1, Some(&user_id), pool).await;
            HttpResponse::Ok().body(body)
        }
        Err(ServiceError::InvalidRating) => {
            HttpResponse::BadRequest().body(form_error("La puntuación tiene que ir de 1 a 5 estrellas."))
        }
        Err(ServiceError::CommentTooLong) => HttpResponse::BadRequest().body(form_error(&format!(
            "El comentario no puede pasar de {} caracteres.",
            MAX_COMMENT_LEN
        ))),
        Err(ServiceError::ProductNotFound) => {
            HttpResponse::BadRequest().body(form_error("Este producto ya no está disponible."))
        }
        // 403 y no 401: la sesión es buena, lo que falta es la compra. Con un
        // 401 se abriría el login, que no arregla nada.
        Err(ServiceError::NotPurchased) => HttpResponse::Forbidden()
            .body(form_error("Solo pueden opinar quienes han comprado este producto.")),
        Err(e) => {
            println!("Error al guardar la opinión: {}", e);
            HttpResponse::InternalServerError()
                .body(form_error("No se ha podido guardar tu opinión. Inténtalo otra vez."))
        }
    }
}

/// Borra la opinión del usuario y devuelve la sección repintada, ya sin ella y
/// con la media recalculada. Va por producto: la opinión la determina el
/// `user_id` de la sesión, así que no hay id ajeno que probar.
///
/// No pide haber comprado: el contenido es suyo, y tiene que poder quitarlo
/// aunque el pedido se cancelase después de opinar.
#[delete("/delete/{product_id}")]
async fn delete_product_review(
    path: web::Path<Uuid>,
    pool_data: web::Data<DbPool>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match session_user_id(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized()
                .insert_header(("HX-Trigger", "auth-required"))
                .body("No autenticado");
        }
    };

    let product_id = path.into_inner();
    let pool = pool_data.get_ref();

    match delete_review(&user_id, &product_id, pool).await {
        Ok(()) => {
            let body = get_product_reviews_body_render(&product_id, 1, Some(&user_id), pool).await;
            HttpResponse::Ok().body(body)
        }
        Err(e) => {
            println!("Error al borrar la opinión: {}", e);
            HttpResponse::InternalServerError()
                .body(form_error("No se ha podido borrar tu opinión. Inténtalo otra vez."))
        }
    }
}
