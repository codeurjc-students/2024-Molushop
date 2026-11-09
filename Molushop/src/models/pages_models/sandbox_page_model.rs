use askama::Template;

/// `/sandbox`: índice de las pruebas de UI. Cada prueba es una subpágina y se
/// borra cuando la idea pasa a su componente real; lo ya decidido está en `/ui`.
#[derive(Template)]
#[template(path = "pages/sandbox/sandbox.html")]
pub struct SandboxPage {
    pub user_logged: bool,
    pub page_name: String,
}
