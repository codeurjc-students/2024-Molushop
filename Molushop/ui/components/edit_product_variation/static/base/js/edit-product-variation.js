const edit_variation = {
    init_edit_variation(element){
        let navItems = element.querySelectorAll('.nav-item');
    
        navItems.forEach(item => {
            item.addEventListener('click', function() {
                // Remover clase active de todas las pestañas
                navItems.forEach(nav => nav.classList.remove('active'));
                
                // Remover clase active de todos los contenidos
                element.querySelectorAll('.tab-pane').forEach(pane => {
                    pane.classList.remove('active');
                });
                
                // Agregar clase active a la pestaña seleccionada
                this.classList.add('active');
                
                // Mostrar el contenido correspondiente
                let tabId = this.getAttribute('data-tab');
                element.querySelector("#"+tabId).classList.add('active');
            });
        }); 
    },
    add_identifier(element,endpoint){
        element.addEventListener('click',()=>{
            //verificar todos los identifier-row que hay --> luego alomejor hay que actualizar si se borra uno
        });
    },
    update_identifiers(element,form){
        element.addEventListener('click',(event)=>{
            console.log("AAAA");
            event.preventDefault();
            console.log("bbbbb");
            let identifiers_containers = form.querySelectorAll(".identifier-row");
            let identifiers={
                identifiers:[]
            };
            if(identifiers_containers.length>0){
                identifiers_containers.forEach((container)=>{
                    let nameInput = container.querySelector(".identifier-name");
                    let valueInput = container.querySelector(".identifier-value");
                    if(nameInput && valueInput){
                        identifiers.identifiers.push({
                            name: nameInput.value,
                            value: valueInput.value,
                        });
                    }
                });
                let endpoint = form.getAttribute("hx-post");
                let target = form.getAttribute("hx-target");

                htmx_ajax_json("POST",endpoint,target,"innerHTML",identifiers)
            }
        });
    },
    editNamesIdentifiers(form){
        let identifiers_containers = form.querySelectorAll(".identifier-row");
        if(identifiers_containers.length>0){
            identifiers_containers.forEach((container, index)=>{
                container.id =`row-${index}`;
                let nameInput = container.querySelector("input.identifier-name");
                let valueInput = container.querySelector("input.identifier-value");
                
                if(nameInput&&valueInput){
                    nameInput.name=`identifiers[${index}][name]`;
                    valueInput.name=`identifiers[${index}][value]`;
                }
                //cambiar por 
            });
        }
    },
    setupDeleteIdentifier(event,endpoint){
        let contenedor = event.currentTarget.closest(".identifier-row");

        return {
            method:"hx-delete",
            message:"Estas seguro de querer eliminar el Identificador???",
            endpoint:endpoint,
            target:`#${contenedor.id}`,
            swap:"outerHTML swap:1s",
            hyperscript_action:`set filaRemove to the first <#${contenedor.id}/> remove filaRemove  set myForm to the first <.identifiers-form/> js(myForm) edit_variation.editNamesIdentifiers(myForm) end`,
            htmx_active:false
        }
    }
}

function htmx_ajax_json(metodo,endpoint,target,swap,values ){
    htmx.ajax(metodo,endpoint,{
        target:target,
        swap:swap,
        headers:{
            'content-type': 'application/json',
            'Accept': 'application/json'
        },
        values: {
            value:JSON.stringify(values)
        }
    });
}


function init_edit_product(){
    let navItems = document.querySelectorAll('.nav-item');
    
    navItems.forEach(item => {
        item.addEventListener('click', function() {
            // Remover clase active de todas las pestañas
            navItems.forEach(nav => nav.classList.remove('active'));
            
            // Remover clase active de todos los contenidos
            document.querySelectorAll('.tab-pane').forEach(pane => {
                pane.classList.remove('active');
            });
            
            // Agregar clase active a la pestaña seleccionada
            this.classList.add('active');
            
            // Mostrar el contenido correspondiente
            let tabId = this.getAttribute('data-tab');
            document.getElementById(tabId).classList.add('active');
        });
    }); 
}
/*
document.addEventListener('DOMContentLoaded', function() {
    edit_product.init_edit_product();
});

window.addEventListener('popstate', function(evt) { 
    edit_product.init_edit_product();
});

document.addEventListener('htmx:afterSwap', () => edit_product.init_edit_product());
*/