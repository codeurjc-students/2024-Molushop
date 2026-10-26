//! Idiomas de la tienda: español (por defecto) e inglés.
//!
//! La URL manda: `/search` es español y `/en/search` inglés, entre quien entre.
//! Las rutas de la tienda se registran dos veces, sin prefijo y bajo `/en`
//! (`routes::config_routes`). El panel y el admin solo existen en español.
//!
//! Lo de `/components/...` (fragmentos de htmx, los `fetch` del carrito y de la
//! ficha, el POST del checkout) no lleva prefijo: va en el idioma de la página
//! desde la que se pide. htmx la manda en `HX-Current-URL`; para `fetch` y los
//! formularios normales, el navegador la pone en `Referer`.
//!
//! El navegador (Accept-Language) y la cookie `lang` (el selector del pie) solo
//! deciden a dónde lleva la raíz `/` (`routes::home::redirect_home`).
//!
//! Los textos están en `locales/{es,en}/*.ftl` (Fluent), metidos en el binario
//! al compilar: cambiar un `.ftl` pide recompilar. En las plantillas:
//! `{{ i18n.t("clave") }}`.

use std::borrow::Cow;
use std::collections::HashMap;
use std::future::{ready, Ready};
use std::sync::OnceLock;

use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{langid, static_loader, LanguageIdentifier, Loader};

static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "es",
        // Sin las marcas invisibles (U+2068/U+2069) que Fluent pone alrededor de
        // cada variable: en HTML no hacen falta y se cuelan al copiar el texto.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

/// Los componentes de la tienda (`controllers::components::scope`).
const COMPONENTS_PREFIX: &str = "/components/";

/// La cookie que guarda el idioma elegido en el selector del pie.
pub const LANG_COOKIE: &str = "lang";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Es,
    En,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::Es, Lang::En];
    pub const DEFAULT: Lang = Lang::Es;

    /// El código ISO ("es", "en"), el de `<html lang>` y `hreflang`.
    pub fn code(self) -> &'static str {
        match self {
            Lang::Es => "es",
            Lang::En => "en",
        }
    }

    /// Lo que va delante de las rutas de la tienda: nada para el español.
    pub fn prefix(self) -> &'static str {
        match self {
            Lang::Es => "",
            Lang::En => "/en",
        }
    }

    /// Su nombre en su propio idioma, para el selector.
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::Es => "Español",
            Lang::En => "English",
        }
    }

    /// "en", "EN" o "en-GB" -> inglés. Lo que no es un idioma de la tienda, `None`.
    pub fn from_code(code: &str) -> Option<Lang> {
        let primary = code.trim().split(['-', '_']).next().unwrap_or("").to_ascii_lowercase();
        Lang::ALL.into_iter().find(|lang| lang.code() == primary)
    }

    /// El idioma de una ruta y la ruta sin el prefijo: "/en/search" -> (En, "/search").
    /// "/en" a secas es la raíz en inglés ("/"); "/english" no lleva prefijo.
    pub fn split_path(path: &str) -> (Lang, &str) {
        for lang in Lang::ALL {
            let prefix = lang.prefix();
            if prefix.is_empty() {
                continue;
            }
            if let Some(rest) = path.strip_prefix(prefix) {
                if rest.is_empty() {
                    return (lang, "/");
                }
                if rest.starts_with('/') {
                    return (lang, rest);
                }
            }
        }
        (Lang::DEFAULT, path)
    }

    /// El idioma de la tienda que el navegador prefiere más (Accept-Language:
    /// "en-GB,en;q=0.9,es;q=0.8"). `None` si no pide ninguno de los nuestros.
    pub fn from_accept_language(header: &str) -> Option<Lang> {
        let mut best: Option<(Lang, f32)> = None;
        for part in header.split(',') {
            let mut pieces = part.split(';');
            let Some(lang) = Lang::from_code(pieces.next().unwrap_or("")) else { continue };
            let quality = pieces
                .find_map(|p| p.trim().strip_prefix("q="))
                .and_then(|q| q.trim().parse::<f32>().ok())
                .unwrap_or(1.0);
            // Con la misma calidad gana el primero, que es el orden del usuario.
            if quality > 0.0 && best.map_or(true, |(_, q)| quality > q) {
                best = Some((lang, quality));
            }
        }
        best.map(|(lang, _)| lang)
    }

    /// La coma o el punto de los decimales ("4,5" / "4.5").
    pub fn decimal_separator(self) -> char {
        match self {
            Lang::Es => ',',
            Lang::En => '.',
        }
    }

    /// El texto de `key` en este idioma. Si falta, el español; si tampoco
    /// está, la propia clave (y un aviso en el log), que se vea en la página.
    /// Va en `Lang` (que es `Copy`) para que los átomos y los correos traduzcan
    /// sin crear un `I18n`.
    pub fn t(self, key: &str) -> String {
        self.lookup(key, None)
    }

    /// Como `t`, con variables: `t_args("search-results", &[("q", q.into())])`.
    pub fn t_args(self, key: &str, args: &[(&'static str, FluentValue)]) -> String {
        let args: HashMap<Cow<'static, str>, FluentValue> =
            args.iter().map(|(name, value)| (Cow::Borrowed(*name), value.clone())).collect();
        self.lookup(key, Some(&args))
    }

    fn lookup(self, key: &str, args: Option<&HashMap<Cow<'static, str>, FluentValue>>) -> String {
        LOCALES.try_lookup_complete(&self.langid(), key, args).unwrap_or_else(|| {
            println!("i18n: falta la clave \"{}\" ({})", key, self.code());
            key.to_string()
        })
    }

    fn langid(self) -> LanguageIdentifier {
        match self {
            Lang::Es => langid!("es"),
            Lang::En => langid!("en"),
        }
    }
}

/// Un número para `I18n::t_count`. askama pasa los campos por referencia
/// (`&i64`) y los literales como `i32`: así vale cualquiera.
pub trait Count {
    fn count(&self) -> i64;
}
impl Count for i64 { fn count(&self) -> i64 { *self } }
impl Count for i32 { fn count(&self) -> i64 { *self as i64 } }
impl Count for usize { fn count(&self) -> i64 { *self as i64 } }
impl<T: Count + ?Sized> Count for &T { fn count(&self) -> i64 { (**self).count() } }

/// Un enlace del selector de idioma.
#[derive(Clone, Debug)]
pub struct LangLink {
    pub code: &'static str,
    pub name: &'static str,
    pub url: String,
    pub current: bool,
}

/// El idioma de una petición y lo que hace falta para pintar en él: los
/// textos (`t`), los enlaces internos (`url`) y la misma página en los otros
/// idiomas (`hreflang` y selector). Lo saca actix de la petición (`FromRequest`).
#[derive(Clone, Debug)]
pub struct I18n {
    pub lang: Lang,
    /// La página sin el prefijo del idioma, con su query ("/search?q=pala").
    path: String,
    /// "https://molushop.es": `hreflang` pide URLs completas.
    origin: String,
}

impl I18n {
    pub fn new(lang: Lang, path: &str, origin: &str) -> Self {
        I18n { lang, path: path.to_string(), origin: origin.to_string() }
    }

    /// Uno en español para lo que se pinta sin petición (las rutas `spawn` de
    /// ejemplo, las tarjetas sueltas por id). Es estático porque los
    /// componentes guardan una referencia (`&'a I18n`), no una copia.
    pub fn default_ref() -> &'static I18n {
        static DEFAULT: OnceLock<I18n> = OnceLock::new();
        DEFAULT.get_or_init(I18n::default)
    }

    pub fn from_http_request(req: &HttpRequest) -> Self {
        let info = req.connection_info();
        let origin = format!("{}://{}", info.scheme(), info.host());
        let (lang, path) = Lang::split_path(req.path());
        // Lo de /components va en el idioma de la página que lo pide (y esa
        // es la página para hreflang, si se repinta entera: el checkout).
        if lang == Lang::DEFAULT && path.starts_with(COMPONENTS_PREFIX) {
            let headers = req.headers();
            let page = headers.get("HX-Current-URL").or_else(|| headers.get(actix_web::http::header::REFERER));
            if let Some(page) = page.and_then(|v| v.to_str().ok()) {
                let (page_lang, page_path) = Lang::split_path(path_of_url(page));
                return I18n::new(page_lang, page_path, &origin);
            }
        }
        let path = match req.query_string() {
            "" => path.to_string(),
            query => format!("{}?{}", path, query),
        };
        I18n::new(lang, &path, &origin)
    }

    /// El código del idioma ("es", "en"), para `<html lang>`.
    pub fn code(&self) -> &'static str {
        self.lang.code()
    }

    /// El texto de `key` en este idioma (`Lang::t`).
    pub fn t(&self, key: &str) -> String {
        self.lang.t(key)
    }

    /// Como `t`, con variables: `t_args("search-results", &[("q", q.into())])`.
    pub fn t_args(&self, key: &str, args: &[(&'static str, FluentValue)]) -> String {
        self.lang.t_args(key, args)
    }

    /// `t_args` con una variable de texto, para las plantillas:
    /// `{{ i18n.t1("search-title-query", "q", q) }}`.
    pub fn t1(&self, key: &str, name: &'static str, value: impl AsRef<str>) -> String {
        self.t_args(key, &[(name, FluentValue::from(value.as_ref()))])
    }

    /// Un texto con plural según `count`, para las plantillas:
    /// `{{ i18n.t_count("product-reviews-count", n) }}`.
    pub fn t_count(&self, key: &str, count: impl Count) -> String {
        self.t_count_as(key, "count", count)
    }

    /// Como `t_count`, con otro nombre de variable:
    /// `{{ i18n.t_count_as("password-forgot-sent", "minutes", reset_minutes) }}`.
    pub fn t_count_as(&self, key: &str, name: &'static str, count: impl Count) -> String {
        self.t_args(key, &[(name, FluentValue::from(count.count()))])
    }

    /// Con dos variables de texto.
    pub fn t2(&self, key: &str, name1: &'static str, value1: impl AsRef<str>, name2: &'static str, value2: impl AsRef<str>) -> String {
        self.t_args(key, &[(name1, FluentValue::from(value1.as_ref())), (name2, FluentValue::from(value2.as_ref()))])
    }

    /// Una ruta de la tienda en este idioma: "/cart" -> "/en/cart". Lo que no
    /// es una ruta interna ("javascript:void(0)", "https://...") sale igual.
    /// `impl AsRef<str>`: askama pasa los campos por referencia y lo que
    /// devuelve un método (`order.detail_url()`), por valor.
    pub fn url(&self, path: impl AsRef<str>) -> String {
        let path = path.as_ref();
        if path.starts_with('/') && !path.starts_with("//") {
            format!("{}{}", self.lang.prefix(), path)
        } else {
            path.to_string()
        }
    }

    /// Esta misma página en `lang`, con el dominio (para `hreflang`).
    pub fn page_url_in(&self, lang: Lang) -> String {
        format!("{}{}{}", self.origin, lang.prefix(), self.path)
    }

    /// Esta página en cada idioma, para las etiquetas `hreflang`.
    pub fn alternates(&self) -> Vec<(&'static str, String)> {
        Lang::ALL.iter().map(|lang| (lang.code(), self.page_url_in(*lang))).collect()
    }

    /// El selector de idioma: cada enlace pasa por `/lang/{code}`, que guarda
    /// la elección en la cookie y vuelve a esta misma página en ese idioma.
    pub fn languages(&self) -> Vec<LangLink> {
        Lang::ALL
            .iter()
            .map(|lang| LangLink {
                code: lang.code(),
                name: lang.native_name(),
                url: format!(
                    "/lang/{}?{}",
                    lang.code(),
                    serde_urlencoded::to_string([("next", self.path.as_str())]).unwrap_or_default()
                ),
                current: *lang == self.lang,
            })
            .collect()
    }
}

impl Default for I18n {
    fn default() -> Self {
        I18n::new(Lang::DEFAULT, "/", "")
    }
}

impl FromRequest for I18n {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(Ok(I18n::from_http_request(req)))
    }
}

/// "http://host:8080/en/search?q=x" -> "/en/search?q=x".
fn path_of_url(url: &str) -> &str {
    match url.split_once("://") {
        Some((_, rest)) => rest.find('/').map_or("/", |i| &rest[i..]),
        None => url,
    }
}

/// Una ruta interna segura a la que volver tras elegir idioma: empieza por "/"
/// pero no por "//" ni "/\" (el navegador las tomaría por otro dominio).
pub fn safe_next(next: &str) -> Option<&str> {
    let ok = next.starts_with('/') && !next.starts_with("//") && !next.starts_with("/\\");
    ok.then_some(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_path_only_takes_whole_segments() {
        assert_eq!(Lang::split_path("/en/search"), (Lang::En, "/search"));
        assert_eq!(Lang::split_path("/en"), (Lang::En, "/"));
        assert_eq!(Lang::split_path("/english"), (Lang::Es, "/english"));
        assert_eq!(Lang::split_path("/search"), (Lang::Es, "/search"));
    }

    #[test]
    fn accept_language_picks_the_best_supported() {
        assert_eq!(Lang::from_accept_language("en-GB,en;q=0.9,es;q=0.8"), Some(Lang::En));
        assert_eq!(Lang::from_accept_language("fr-FR,fr;q=0.9,es;q=0.5,en;q=0.4"), Some(Lang::Es));
        assert_eq!(Lang::from_accept_language("de,fr"), None);
        assert_eq!(Lang::from_accept_language("en;q=0,es"), Some(Lang::Es));
        assert_eq!(Lang::from_accept_language(""), None);
    }

    #[test]
    fn urls_keep_the_page_and_the_query() {
        let i18n = I18n::new(Lang::En, "/search?q=pala roja", "http://localhost:8080");
        assert_eq!(i18n.url("/cart"), "/en/cart");
        assert_eq!(i18n.url("javascript:void(0)"), "javascript:void(0)");
        assert_eq!(I18n::default_ref().url("/cart"), "/cart");
        assert_eq!(i18n.page_url_in(Lang::Es), "http://localhost:8080/search?q=pala roja");
        let en = i18n.languages().into_iter().find(|l| l.code == "en").unwrap();
        assert!(en.current);
        assert_eq!(en.url, "/lang/en?next=%2Fsearch%3Fq%3Dpala+roja");
    }

    #[test]
    fn next_must_stay_on_this_site() {
        assert_eq!(safe_next("/search?q=x"), Some("/search?q=x"));
        assert_eq!(safe_next("//evil.com"), None);
        assert_eq!(safe_next("/\\evil.com"), None);
        assert_eq!(safe_next("https://evil.com"), None);
    }

    #[test]
    fn texts_fall_back_to_spanish_and_then_to_the_key() {
        let en = I18n::new(Lang::En, "/", "");
        assert_eq!(en.t("lang-label"), "Language");
        assert_eq!(I18n::default().t("lang-label"), "Idioma");
        assert_eq!(en.t("no-such-key"), "no-such-key");
        assert_eq!(en.t_args("i18n-test-count", &[("count", 1.into())]), "1 product");
        assert_eq!(Lang::En.t("lang-label"), "Language");
        assert_eq!(I18n::default().t_args("i18n-test-count", &[("count", 3.into())]), "3 productos");
        assert_eq!(en.t1("search-title-query", "q", "pala"), "Results for “pala”");
        assert_eq!(en.t2("search-title-query-category", "q", "pala", "category", "Padel"), "“pala” in Padel");
    }

    #[test]
    fn every_key_exists_in_every_language() {
        // Una clave que solo está en un idioma saldría en español (o como clave) en el otro.
        let keys = |lang: &str| -> std::collections::BTreeSet<String> {
            let dir = format!("{}/locales/{}", env!("CARGO_MANIFEST_DIR"), lang);
            std::fs::read_dir(dir).unwrap()
                .flat_map(|f| std::fs::read_to_string(f.unwrap().path()).unwrap().lines().map(str::to_string).collect::<Vec<_>>())
                .filter(|l| l.chars().next().map_or(false, |c| c.is_ascii_lowercase()))
                .filter_map(|l| l.split_once(" =").map(|(k, _)| k.trim().to_string()))
                .collect()
        };
        assert_eq!(keys("es"), keys("en"));
    }

    #[test]
    fn components_follow_the_page_language() {
        use actix_web::test::TestRequest;
        let from = |req: TestRequest| I18n::from_http_request(&req.to_http_request());
        // htmx manda la página en HX-Current-URL...
        let i18n = from(TestRequest::get().uri("/components/search-suggestions?q=pa")
            .insert_header(("HX-Current-URL", "http://localhost:8080/en/search?q=pa")));
        assert_eq!((i18n.lang, i18n.path.as_str()), (Lang::En, "/search?q=pa"));
        // ...y fetch y los formularios, el navegador en Referer.
        let i18n = from(TestRequest::post().uri("/components/checkout/confirm")
            .insert_header(("Referer", "http://localhost:8080/en/checkout?lines=1")));
        assert_eq!((i18n.lang, i18n.path.as_str()), (Lang::En, "/checkout?lines=1"));
        // Fuera de /components manda la URL, venga de donde venga.
        let i18n = from(TestRequest::get().uri("/search").insert_header(("Referer", "http://localhost:8080/en/home")));
        assert_eq!(i18n.lang, Lang::Es);
        assert_eq!(from(TestRequest::get().uri("/components/x")).lang, Lang::Es);
    }

    #[test]
    fn fragments_follow_the_page_language() {
        assert_eq!(path_of_url("http://localhost:8080/en/search?q=x"), "/en/search?q=x");
        assert_eq!(path_of_url("https://molushop.es"), "/");
    }
}
