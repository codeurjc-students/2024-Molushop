//! Selector de idioma del pie: `/lang/en?next=/search?q=pala` guarda la
//! elección en la cookie `lang` y lleva a esa misma página en inglés
//! (`/en/search?q=pala`). La cookie solo cuenta al entrar por la raíz `/`
//! (`home::redirect_home`); en el resto manda la URL.

use actix_web::cookie::{time::Duration, Cookie, SameSite};
use actix_web::{http::header, web, HttpResponse};
use serde::Deserialize;

use crate::constants::urls::HOME_URL;
use crate::i18n::{safe_next, Lang, LANG_COOKIE};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/lang/{code}", web::get().to(switch_lang));
}

#[derive(Deserialize)]
pub struct SwitchQuery {
    /// La página sin prefijo de idioma. Si falta o apunta fuera, la portada.
    pub next: Option<String>,
}

async fn switch_lang(code: web::Path<String>, query: web::Query<SwitchQuery>) -> HttpResponse {
    let Some(lang) = Lang::from_code(&code) else {
        return HttpResponse::NotFound().finish();
    };
    let next = query.next.as_deref().and_then(safe_next).unwrap_or(HOME_URL);
    // Por si llega con prefijo (un enlace escrito a mano): no se dobla.
    let (_, next) = Lang::split_path(next);

    let cookie = Cookie::build(LANG_COOKIE, lang.code())
        .path("/")
        .max_age(Duration::days(365))
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();
    HttpResponse::Found()
        .insert_header((header::LOCATION, format!("{}{}", lang.prefix(), next)))
        .cookie(cookie)
        .finish()
}
