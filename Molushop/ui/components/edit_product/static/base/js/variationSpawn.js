//NO SE USA
const edit_product_spawn_var = {
    handleSwap(element,path){
        console.log("registrando evento");
        element.addEventListener('click',()=>{
            //verificar que el elemento "nombre_conenedor, existe"
            let product_edit_container = element.closest(".product-edit-container");
            console.log(product_edit_container);
            let brotherContainer = product_edit_container.previousElementSibling;
            console.log(brotherContainer);
            if(brotherContainer){
                if(brotherContainer.classList.contains("variation-edit-container")){
                    htmx_ajax_1("GET",path,brotherContainer,"outerHTML");
                }else{//si no lo tiene
                    htmx_ajax_1("GET",path,product_edit_container,"beforebegin");
                }
            }else{
                //si el hermano no existe (nulo)
                htmx_ajax_1("GET",path,product_edit_container,"beforebegin");
            }
        })//swap manual o eliminar 
    },
    handleSwapClick(element,path){
        //verificar que el elemento "nombre_conenedor, existe"
        let product_edit_container = element.closest(".product-edit-container");
        console.log(product_edit_container);
        let brotherContainer = product_edit_container.previousElementSibling;
        console.log(brotherContainer);
        if(brotherContainer){
            if(brotherContainer.classList.contains("variation-edit-container")){
                htmx_ajax_1("GET",path,brotherContainer,"outerHTML");
            }else{//si no lo tiene
                htmx_ajax_1("GET",path,product_edit_container,"beforebegin");
            }
        }else{
            //si el hermano no existe (nulo)
            htmx_ajax_1("GET",path,product_edit_container,"beforebegin");
        }
    }
    ,
    handleSwap2(element,id){
        let elementContainer = element.closest(".product-edit-container");
        let brotherContainer = elementContainer.previousSibiling;
        if(brotherContainer){
            if(!brotherContainer.classList.contains("variation-edit-container")){
                //si no tiene un hermano
                e.detail.shouldSwap=false;
            }
        }else{
            //si el hermano no existe (nulo)
            htmx.ajax('GET', '/components/edit-product-variation/spawn/'+id, {target:'closest .product-edit-container', swap:'outerHTML'})
        }
    },
    handleSwap3(element){
        element.addEventListener("htmx:beforeRequest",(e)=>{
            console.log("Before Request");
            let elementContainer = element.closest(".product-edit-container");
            let brotherContainer = elementContainer.previousSibiling;
            if(brotherContainer){
                if(brotherContainer.classList.contains("variation-edit-container")){
                    //si tiene un hermano modificar para
                    
                }
            }else{
                //si el hermano no existe (nulo)
                
            }
        });
    }
}

//ahora si!
function pruebaXX(elemento,id){
    let urlFinal = '/components/edit-product-variation/spawn/'+id; //a7b51bc2-b5dd-4bda-b30a-f14c2ef9cd7c
    let clos = elemento.closest('.product-edit-container');
    console.log("HOla");
    htmx.ajax("GET",urlFinal,{
        target: clos, // Encuentra el ancestro más cercano
        swap:'beforebegin'
        });
}

function htmx_ajax_1(metodo,endpoint,target,swap){
    htmx.ajax(metodo,endpoint,{
        target:target,
        swap:swap
    });
}