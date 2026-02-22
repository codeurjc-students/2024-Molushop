use askama::Template;

use crate::models::components::nav1_model::*;
use crate::controllers::components::nav1_controller::ROUTES;
//si le damos los datos ya?
//tenemos que obtener las rutas del controller, o los pongo directamente aquí?

//TODO falta poner el numero del carrito
pub fn get_nav1_object(name:String)->Nav1Data{
    let data = Nav1Data{
        routes: &ROUTES,
        name
    };
    data
}

pub fn get_nav1_render(name:String)->String{
    let objeto =get_nav1_object(name);

    let render = Nav1{
        nav1:objeto
    }.render().unwrap();
    render
}