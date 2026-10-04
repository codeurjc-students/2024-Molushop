/// Formulario de `/panel/login`.
///
/// No reutiliza el `login_base` del modal a propósito: aquel pinta el registro
/// y abre modales de htmx, y en el panel solo hace falta entrar. Lo que sí se
/// reutiliza es el ENDPOINT de login (misma cookie, misma sesión).
#[derive(Clone, Debug)]
pub struct PanelLoginData {
    /// POST de login del modal (`login_base_controller::ROUTES.login`).
    pub login_url: String,
    /// A donde volver al entrar. Ya saneado: siempre una ruta de `/panel` o `/admin`.
    pub next: String,
    /// Hay sesión pero el usuario no es ni admin ni vendedor.
    pub logged_without_access: bool,
    /// Acaba de registrarse como vendedor: aviso de "cuenta creada, entra".
    pub registered: bool,
}
