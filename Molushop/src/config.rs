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
    configure_static_routes_recursive(cfg, path_pages).expect("Error configurando rutas estáticas para pages");
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


//hacer un método que te recorra recursivamente las subcarpetas que no tengan el nombre "static" si tiene más carpetas 
fn configure_static_routes_recursive(cfg: &mut web::ServiceConfig, path: &Path) -> std::io::Result<()> {
    if !path.is_dir() {
        return Ok(());
    }

    // Procesar el directorio actual para encontrar carpetas "static/base"
    let static_path = path.join("static");
    let base_path = static_path.join("base");

    if base_path.exists() {
        // Si existe la estructura static/base, configurar esta ruta
        if let Ok(relative_path) = path.strip_prefix("./ui") {
            let xtra = relative_path.to_str().unwrap_or("").replace("\\", "/");
            let route = format!("/static/{}", xtra);
            //println!("Configurando ruta estática: {}", route);
            cfg.service(fs::Files::new(&route, &base_path).show_files_listing());
        }
    }

    // Explorar recursivamente los subdirectorios que no sean "static"
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        
        if entry_path.is_dir() {
            // Verificar que no sea una carpeta llamada "static"
            if let Some(dir_name) = entry_path.file_name() {
                if dir_name != "static" && dir_name != "templates" {
                    //println!("AAAAAAAAA {}",&entry_path.to_str().unwrap());
                    configure_static_routes_recursive(cfg, &entry_path)?;
                }
            }
        }
    }

    Ok(())
}