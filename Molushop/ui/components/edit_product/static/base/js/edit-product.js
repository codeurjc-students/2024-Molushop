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
    }
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