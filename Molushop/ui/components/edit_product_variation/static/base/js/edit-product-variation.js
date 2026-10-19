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
    change_status(element,e){
        // Misma etiqueta y clase de fila que pinta edit_product.html
        let states = {
            0: ["Draft", "neutral", "is-draft"],
            1: ["Activo", "success", "is-active"],
            2: ["Inactivo", "danger", "is-inactive"],
        };
        let [status_name, variant, row_class] = states[e.detail.status] || states[2];
        let tr_target = document.querySelector("tr#"+CSS.escape("var-row-"+element.id));
        if(!tr_target) return;
        tr_target.className = row_class;
        tr_target.querySelector("td.state").innerHTML =
            `<span class="m-badge m-badge--${variant}">${status_name}</span>`;
    },
    // Casilla "Habilitar oferta": abre o cierra el precio de oferta de su formulario
    toggle_sale(checkbox){
        let form = checkbox.closest("form");
        form.querySelector(".sale-active").value = checkbox.checked ? "true" : "false";
        form.querySelector(".sale-price").readOnly = !checkbox.checked;
    },
    update_identifiers(element,form){
        element.addEventListener('click',(event)=>{
            event.preventDefault();
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
