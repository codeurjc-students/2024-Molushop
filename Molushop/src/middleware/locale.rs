use std::future::{ready, Ready};
use std::rc::Rc;

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;

/// Constantes de idiomas soportados
const DEFAULT_LOCALE: &str = "es";
const SUPPORTED_LOCALES: &[(&str, &str)] = &[
    ("es", ""),       // Español - sin prefijo
    ("en", "/en"),    // Inglés
    ("pt", "/pt"),    // Portugués
];

/// Datos de locale para pasar en extensiones
#[derive(Clone, Debug)]
pub struct LocaleData {
    pub locale: String,
    pub prefix: String,
}

impl Default for LocaleData {
    fn default() -> Self {
        Self {
            locale: DEFAULT_LOCALE.to_string(),
            prefix: String::new(),
        }
    }
}

/// Middleware para extraer el idioma de la ruta
pub struct Locale;

impl Locale {
    pub fn new() -> Self {
        Self {}
    }
}

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for Locale
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LocaleMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LocaleMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct LocaleMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for LocaleMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Extraer locale de la ruta
        let locale_data = extract_locale(req.path());
        
        // Almacenar en extensiones para acceso posterior
        req.extensions_mut().insert(locale_data);

        let service = self.service.clone();

        Box::pin(async move {
            let res = service.call(req).await?;
            Ok(res)
        })
    }
}

/// Extrae el idioma desde la ruta
fn extract_locale(path: &str) -> LocaleData {
    // Buscar prefijo en la ruta
    for (locale, prefix) in SUPPORTED_LOCALES.iter() {
        if !prefix.is_empty() && path.starts_with(prefix) {
            return LocaleData {
                locale: locale.to_string(),
                prefix: prefix.to_string(),
            };
        }
    }
    
    // Si no hay prefijo, usar idioma por defecto
    LocaleData::default()
}