// Abre el editor de una variación encima del editor de producto; si ya hay
// uno abierto, lo sustituye.
const edit_product_spawn_var = {
    handleSwapClick(element,path){
        let product_edit_container = element.closest(".product-edit-container");
        let brotherContainer = product_edit_container.previousElementSibling;
        if(brotherContainer && brotherContainer.classList.contains("variation-edit-container")){
            htmx_ajax_1("GET",path,brotherContainer,"outerHTML");
        }else{
            htmx_ajax_1("GET",path,product_edit_container,"beforebegin");
        }
    }
}

function htmx_ajax_1(metodo,endpoint,target,swap){
    htmx.ajax(metodo,endpoint,{
        target:target,
        swap:swap
    });
}
