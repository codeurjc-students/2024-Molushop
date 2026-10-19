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
    // Tras guardar estado, stock o precio (HX-Trigger variation_update): la
    // fila del explorador toma el dato nuevo y se recalculan filtros y matriz.
    // El estado de la fila sale como en VariationsExplorer::state_class.
    update_row(element,e){
        let row = document.querySelector("tr#"+CSS.escape("var-row-"+element.id));
        if(!row) return;
        let d = e.detail;
        if(d.status !== undefined) row.dataset.status = d.status;
        if(d.stock !== undefined){
            row.dataset.stock = d.stock;
            row.querySelector("td.num").textContent = d.stock;
        }
        if(d.price !== undefined){
            row.dataset.price = d.price;
            row.querySelector("td.price .m-price__amount").textContent = MONEY.format(d.price);
            row.querySelector("td.price .m-price__currency").hidden = false;
        }
        let status = Number(row.dataset.status);
        let stock = Number(row.dataset.stock);
        let state = status == 1 ? (stock > 0 ? "is-active" : "is-out")
            : status == 0 ? "is-draft" : "is-inactive";
        let [label, variant] = {1: ["Activo", "success"], 0: ["Draft", "neutral"]}[status] || ["Inactivo", "danger"];
        row.className = state;
        row.dataset.state = state;
        row.querySelector("td.state").innerHTML = `<span class="m-badge m-badge--${variant}">${label}</span>`;
        row.dispatchEvent(new CustomEvent("ve:refresh", {bubbles: true}));
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
