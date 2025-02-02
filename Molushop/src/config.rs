use actix_files as fs;
use actix_web::web;
//use std::fs;
use std::path::{Path, PathBuf};


pub fn static_config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(fs::Files::new("/css", "./target_css").show_files_listing())
    ;
    //Se agregan las rutas estáticas de los componentes y páginas, se añaden automáticamente si siguen la estrucutra:
    // ui/components/<component_name>/static/base
    // ui/pages/<page_name>/static/base
    // dentro de la carpeta "base" se coloca el contenido estático
    let path_components = Path::new("./ui/components");
    let path_pages = Path::new("./ui/pages");

    // Configurar rutas estáticas para "components"
    configure_static_routes(cfg,path_components ).expect("Error configurando rutas estáticas para components");

    // Configurar rutas estáticas para "pages"
    configure_static_routes(cfg, path_pages).expect("Error configurando rutas estáticas para pages");
}

fn configure_static_routes(cfg: &mut web::ServiceConfig, path: &Path) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let static_path = path.join("static");
                let base_path = static_path.join("base");

                if base_path.exists() {
                    let xtra = path.strip_prefix("./ui").unwrap().to_str().unwrap().replace("\\", "/");
                    let route = format!("/static/{}", xtra);
                    //println!("Configurando ruta estática: {}", route);
                    cfg.service(fs::Files::new(&route, &base_path).show_files_listing());
                }
            }
        }
    }
    Ok(())
}