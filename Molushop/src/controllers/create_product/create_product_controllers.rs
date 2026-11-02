use actix_web::{get, post, web,Scope, HttpRequest, HttpResponse, Responder, http::StatusCode};
use lazy_static::lazy_static;

use askama::Template;

use uuid::Uuid;

use crate::services::category_service::{get_root_categories, get_child_categories, get_category_ancestors};
use crate::services::product_service::create_full_product;
use crate::services::panel_role_service::{resolve_store, PanelUser};
use crate::utils::hx::error_modal;

use crate::models::models_x::ProductForm;
use crate::models::extra::BaseSpecs;
//use crate::models::product_variation::{VariationValue,Attribute};
//use crate::models::get_product::ProductForm;


use crate::models::components::create_product::*;



//use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
//use bb8::Pool;
//type DbPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
type DbPool = Pool<AsyncPgConnection>;

static SCOPE: &str = "/create-product";
/// Se registra dentro del scope `/panel` (`routes/panel.rs`); `ROUTES` lleva la URL completa.
static SCOPE_FULL: &str = "/panel/create-product";
/// Cabecera con la tienda a cuyo nombre se crea el producto.
pub static STORE_HEADER: &str = "X-Store-Id";
/// La cabecera falta, no es un UUID o es de una tienda que el usuario no gestiona.
static SIN_TIENDA: &str = "No se puede crear el producto: tienda no válida.";

lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza
  
    //IDs de ejemplo
    static ref ID_BASE:Uuid =Uuid::parse_str("95022733-f013-301a-0ada-abc18f151006").unwrap();
    static ref ID_BASE_JOKER:Uuid =Uuid::parse_str("95023733-f013-301a-0ada-abc18f151006").unwrap();  
    /* 
    static ref CATEGORY_CHILDREN_URL: String = format!("{}/category-children", SCOPE);
    static ref RESET_CATEGORY_URL: String = format!("{}/reset-category", SCOPE);
    static ref NEXT_SELECT_URL: String = format!("{}/next-select", SCOPE);
    static ref CREATE_PRODUCT_URL: String = format!("{}/create-product", SCOPE);
    static ref ADD_VARIATION_URL: String = format!("{}/add-variations", SCOPE);
    static ref ADD_VARIATION_VALUE_URL: String = format!("{}/add-variations-attributes", SCOPE);
    static ref ADD_SPECS_URL: String = format!("{}/add-specs", SCOPE);
    */
    pub static ref ROUTES: Routes = Routes{
        category_children: Box::leak(format!("{}/category-children", SCOPE_FULL).into_boxed_str()),
            reset_category: Box::leak(format!("{}/reset-category", SCOPE_FULL).into_boxed_str()),
            next_select: Box::leak(format!("{}/next-select", SCOPE_FULL).into_boxed_str()),
            create_product: Box::leak(format!("{}/create-product", SCOPE_FULL).into_boxed_str()),
    };

}

pub fn scope_create_product() -> Scope {
    web::scope(SCOPE)
        .configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(category_children);
    cfg.service(reset_category);
    cfg.service(next_category);
    cfg.service(create_product);
}


#[get("/category-children/{category_id}")]
async fn category_children(path: web::Path<String>, pool_data:web::Data<DbPool>) -> impl Responder {
    let pool = pool_data.get_ref();
    let category_id= path.into_inner();
    let categories_result = get_child_categories(&category_id,pool).await;
    let ancestors_result: Result<Vec<crate::models::models_x::Category>, diesel::result::Error> = get_category_ancestors(&category_id,pool).await;
    match (categories_result, ancestors_result) {
        (Ok(categories), Ok(ancestors)) => {

            let list_category_base = ListCategoryBase::new_2(
                ancestors,
                categories,
                &ROUTES
            );
            let rendered = list_category_base.render().unwrap();
                        HttpResponse::Ok().body(rendered)
        },
        (Err(e), Ok(_)) => {
            tracing::error!(%category_id, error = %e, "could not load the child categories");
            return error_modal(StatusCode::INTERNAL_SERVER_ERROR, "No se han podido cargar las categorías");
        },
        (Ok(_), Err(e)) => {
            tracing::error!(%category_id, error = %e, "could not load the category ancestors");
            return error_modal(StatusCode::INTERNAL_SERVER_ERROR, "No se han podido cargar las categorías");
        },
        (Err(e1), Err(e2)) => {
            tracing::error!(%category_id, categories_error = %e1, ancestors_error = %e2, "could not load the child categories nor the ancestors");
            return error_modal(StatusCode::INTERNAL_SERVER_ERROR, "No se han podido cargar las categorías");
        }
    }
}

#[get("/reset-category")]
async fn reset_category(pool_data: web::Data<DbPool>) -> impl Responder {
    let pool = pool_data.get_ref();
    let categories = get_root_categories(pool).await;
    match categories{
        Ok(categories) => {
            let base_category = BaseCategory::new(categories,&ROUTES); 
            let rendered = base_category.render().unwrap();
            HttpResponse::Ok().body(rendered)
        },
        Err(e) => {
            tracing::error!(error = %e, "could not load the root categories");
            return error_modal(StatusCode::INTERNAL_SERVER_ERROR, "No se han podido cargar las categorías");
        }
    }
}

/// Formulario del alta para la categoría final elegida. Los antepasados (la
/// categoría incluida, al final) dan la miga de pan de su cabecera.
#[get("/next-select/{category_id}")]
async fn next_category(path: web::Path<String>, pool_data: web::Data<DbPool>) -> impl Responder {
    let category_id= path.into_inner();
    match get_category_ancestors(&category_id, pool_data.get_ref()).await {
        Ok(mut ancestors) if !ancestors.is_empty() => {
            // Las specs son opcionales: una categoría sin plantilla (o con una
            // que no se entiende) abre el formulario sin specs sugeridas.
            let base_specs = ancestors.last_mut()
                .and_then(|category| category.base_specs.take())
                .and_then(|specs| match serde_json::from_value::<BaseSpecs>(specs) {
                    Ok(base_specs) => Some(base_specs),
                    Err(e) => {
                        tracing::warn!(%category_id, error = %e, "invalid base_specs format");
                        None
                    }
                })
                .unwrap_or(BaseSpecs { specs: Vec::new() });
            let base_product = BaseProducto::new(base_specs,category_id, &ROUTES,ancestors);
            HttpResponse::Ok().body(base_product.render().unwrap())
        },
        Ok(_) => {
            error_modal(StatusCode::NOT_FOUND, "No se ha encontrado la categoría")
        },
        Err(e) => {
            tracing::error!(%category_id, error = %e, "could not load the category");
            return error_modal(StatusCode::INTERNAL_SERVER_ERROR, "No se han podido cargar las categorías");
        }
    }
}

#[post("/create-product/{category_id}")]
async fn create_product(pool_data: web::Data<DbPool>,path:web::Path<String>,panel_user: web::ReqData<PanelUser>,req: HttpRequest,data: web::Json<ProductForm>) -> impl Responder {
    // El producto se crea a nombre de la tienda desde cuya página se manda el
    // formulario (`hx-headers` en `pages/panel_product_create/panel_product_create.html`). La
    // cabecera la pone el cliente, así que se comprueba como la tienda de una URL.
    let store_id = req.headers().get(STORE_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok());
    let seller_id = match store_id {
        Some(store_id) => match resolve_store(&panel_user, &store_id, pool_data.get_ref()).await {
            Ok(Some(store)) => store.id,
            Ok(None) => return error_modal(StatusCode::FORBIDDEN, SIN_TIENDA),
            Err(e) => {
                tracing::error!(error = %e, "could not load the seller store");
                return error_modal(StatusCode::INTERNAL_SERVER_ERROR, "Error al crear el producto");
            }
        },
        None => return error_modal(StatusCode::FORBIDDEN, SIN_TIENDA),
    };

    let category_id= path.into_inner();
    let pool = pool_data.get_ref();
    let datox = data.into_inner();
    //match insert_product(&datox,pool).await{
    // Producto, vendedor, categoría, variaciones y precios van en UNA
    // transacción: o se crea todo o no se crea nada.
    match create_full_product(seller_id,&datox,&category_id,pool).await{
        Ok(new_id) => {
            tracing::info!(product_id = %new_id, "product created");
            // A su página de edición: el formulario ya no sirve (enviarlo otra
            // vez crearía otro producto) y allí se siguen sus imágenes y stock.
            HttpResponse::Ok()
                .insert_header(("HX-Redirect", format!("/panel/stores/{}/products/edit/{}", seller_id, new_id)))
                .finish()
        },
        Err(crate::models::error::ServiceError::InvalidVariations) => {
            error_modal(StatusCode::BAD_REQUEST, "Las variaciones del producto no tienen un formato válido")
        },
        Err(crate::models::error::ServiceError::InvalidSpecs) => {
            error_modal(StatusCode::BAD_REQUEST, "Las specs del producto no tienen un formato válido")
        },
        Err(e) => {
            tracing::error!(error = %e, "could not create the product");
            error_modal(StatusCode::INTERNAL_SERVER_ERROR, "Error al crear el producto")
        }
    }
}
