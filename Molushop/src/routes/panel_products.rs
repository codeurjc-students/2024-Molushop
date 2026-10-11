//! Páginas de productos del panel, siempre dentro de una tienda:
//! `/panel/stores/{store_id}/products`, `.../create` y `.../edit/{id}`. Se
//! registran dentro del scope `/panel`, así que `PanelGuard` ya ha comprobado el
//! rol y dejado un `PanelUser`; aquí se comprueba la tienda de la URL.

use actix_web::{http::StatusCode, web, HttpRequest, HttpResponse};
use askama::Template;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::controllers::components::edit_product_controller::ROUTES as ROUTES_EDIT_PRODUCT;
use crate::controllers::components::product_panel_group::ROUTES as ROUTES_PRODUCT_PANEL_GROUP;
use crate::controllers::createProduct::create_product_controllers::ROUTES as ROUTES_CREATE_PRODUCT;
use crate::models::components::edit_product::EditProduct;
use crate::models::components::panel_nav::PanelNav;
use crate::models::components::title::Title;
use crate::models::pages::{CategoryTemplate, ProductsPanel, ProductsPanelEdit};
use crate::services::components::edit_product;
use crate::services::panel_role_service::{product_in_store, resolve_store, PanelStore, PanelUser};
use crate::services::products_panel;
use crate::services::servicesX::obtain_base_categories;

/// Rutas relativas al scope `/panel`.
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/stores/{store_id}/products", web::get().to(products_list))
        .route("/stores/{store_id}/products/create", web::get().to(products_create))
        .route("/stores/{store_id}/products/edit/{id}", web::get().to(products_edit));
}

pub fn html(status: StatusCode, body: String) -> HttpResponse {
    HttpResponse::build(status)
        .content_type("text/html; charset=utf-8")
        .body(body)
}

/// La tienda de la URL o la respuesta de error. 404 tanto si no existe como si
/// el usuario no puede entrar (ajena, pendiente o rechazada): no se revela cuál.
pub async fn store_or_response(
    panel_user: &PanelUser,
    store_id: &Uuid,
    pool: &DbPool,
) -> Result<PanelStore, HttpResponse> {
    match resolve_store(panel_user, store_id, pool).await {
        Ok(Some(store)) => Ok(store),
        Ok(None) => Err(html(StatusCode::NOT_FOUND, "Tienda no encontrada".to_string())),
        Err(e) => {
            println!("Panel: {}", e);
            Err(html(StatusCode::INTERNAL_SERVER_ERROR, "Error cargando la tienda".to_string()))
        }
    }
}

async fn products_list(
    path: web::Path<Uuid>,
    panel_user: web::ReqData<PanelUser>,
    pool_data: web::Data<DbPool>,
) -> HttpResponse {
    let pool = pool_data.get_ref();
    let store = match store_or_response(&panel_user, &path, pool).await {
        Ok(store) => store,
        Err(resp) => return resp,
    };

    match products_panel::get_products(&store.id, pool).await {
        Ok(products) => {
            let page = ProductsPanel {
                user_logged: false,
                products,
                routes: &ROUTES_PRODUCT_PANEL_GROUP,
                page_name: format!("Productos - {}", store.store_name),
                store_id: store.id,
                store_name: store.store_name.clone(),
                panel_nav: PanelNav::new(&panel_user, "products").with_store(&store),
            }.render().unwrap();
            html(StatusCode::OK, page)
        }
        Err(e) => {
            println!("Panel: error cargando productos de la tienda {}: {}", store.id, e);
            html(StatusCode::INTERNAL_SERVER_ERROR, "Error cargando los productos".to_string())
        }
    }
}

async fn products_create(
    path: web::Path<Uuid>,
    panel_user: web::ReqData<PanelUser>,
    pool_data: web::Data<DbPool>,
) -> HttpResponse {
    let pool = pool_data.get_ref();
    let store = match store_or_response(&panel_user, &path, pool).await {
        Ok(store) => store,
        Err(resp) => return resp,
    };

    match obtain_base_categories(pool).await {
        Ok(categories) => {
            let page = CategoryTemplate::new_2(
                categories,
                &ROUTES_CREATE_PRODUCT,
                PanelNav::new(&panel_user, "create").with_store(&store),
                store.id,
            ).render().unwrap();
            html(StatusCode::OK, page)
        }
        Err(e) => {
            println!("Panel: error cargando categorías: {}", e);
            html(StatusCode::INTERNAL_SERVER_ERROR, "Error cargando las categorías".to_string())
        }
    }
}

/// 404 tanto si el producto no existe como si es de otra tienda: así no se
/// revela qué productos existen.
async fn products_edit(
    path: web::Path<(Uuid, Uuid)>,
    panel_user: web::ReqData<PanelUser>,
    pool_data: web::Data<DbPool>,
    req: HttpRequest,
) -> HttpResponse {
    let (store_id, product_id) = path.into_inner();
    let pool = pool_data.get_ref();
    let store = match store_or_response(&panel_user, &store_id, pool).await {
        Ok(store) => store,
        Err(resp) => return resp,
    };

    match product_in_store(&product_id, &store.id, pool).await {
        Ok(true) => {}
        Ok(false) => return html(StatusCode::NOT_FOUND, "Producto no encontrado".to_string()),
        Err(e) => {
            println!("Panel: {}", e);
            return html(StatusCode::INTERNAL_SERVER_ERROR, "Error cargando el producto".to_string());
        }
    }

    let product_edit = match edit_product::get_product(&product_id, pool).await {
        Ok(product_edit) => product_edit,
        Err(e) => {
            println!("Panel: error cargando el producto {}: {}", product_id, e);
            return html(StatusCode::INTERNAL_SERVER_ERROR, "Error cargando el producto".to_string());
        }
    };

    // Desde el listado llega por htmx: solo el título y el editor.
    let page = if req.headers().contains_key("HX-Request") {
        let title = Title { title: "Edit Product".to_string() }.render().unwrap();
        let editor = EditProduct {
            product: product_edit,
            routes_edit_product: &ROUTES_EDIT_PRODUCT,
        }.render().unwrap();
        format!("{}{}", title, editor)
    } else {
        ProductsPanelEdit {
            user_logged: false,
            product: product_edit,
            routes_edit_product: &ROUTES_EDIT_PRODUCT,
            page_name: "Edit Product".to_string(),
            panel_nav: PanelNav::new(&panel_user, "products").with_store(&store),
            store_id: store.id,
        }.render().unwrap()
    };
    html(StatusCode::OK, page)
}
