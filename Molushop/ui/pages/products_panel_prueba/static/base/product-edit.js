document.addEventListener('DOMContentLoaded', function() {
    // Manejador de clicks en las pestañas
    const navItems = document.querySelectorAll('.nav-item');
    
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
            const tabId = this.getAttribute('data-tab');
            document.getElementById(tabId).classList.add('active');
        });
    });
});