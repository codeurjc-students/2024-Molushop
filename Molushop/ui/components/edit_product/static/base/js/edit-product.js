const edit_product = {
    init_edit_product(element){
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
    // Tras guardar el General (HX-Trigger product_update): la cabecera toma
    // lo que se acaba de guardar, que está en el propio formulario.
    update_header(element, e){
        let form = element.querySelector(".general-form");
        let states = {
            0: ["Draft", "neutral"],
            1: ["Activo", "success"],
            2: ["Inactivo", "danger"],
        };
        let [label, variant] = states[e.detail.status] || states[2];
        let badge = element.querySelector(".product-main .product-status");
        if(badge){
            badge.className = `m-badge m-badge--${variant} product-status`;
            badge.textContent = label;
        }
        let title = element.querySelector(".product-main .product-title");
        if(title) title.textContent = form.elements["name"].value;
        let brand = form.elements["brand"].value.trim();
        let brandLine = element.querySelector(".product-main .product-store");
        if(brand && !brandLine){
            brandLine = document.createElement("p");
            brandLine.className = "product-store";
            title.after(brandLine);
        }
        if(brandLine){
            if(brand) brandLine.textContent = "Marca: " + brand;
            else brandLine.remove();
        }
    }
}
