//esto es un ejemplo
const product_panel_group = {
    cargarBotones(){
        document.querySelectorAll('.toggle-variations').forEach(button => {
            button.addEventListener('click', function() {
                console.log('click');
                const variations = this.nextElementSibling;
                variations.classList.toggle('hidden');
            });
        });
    }
}

function cargarBotones(){
    document.querySelectorAll('.toggle-variations').forEach(button => {
        button.addEventListener('click', function() {
            console.log('click');
            const variations = this.nextElementSibling;
            variations.classList.toggle('hidden');
        });
    });
}

let currentPath;

/*
document.addEventListener('DOMContentLoaded', function() {
    product_panel_group.cargarBotones();
    //currentPath = window.location.pathname;
    console.log('DOMContentLoaded');
});
//content

window.addEventListener('popstate', () => {
    console.log('popstate');
    product_panel_group.cargarBotones()});

window.addEventListener('pageshow', () => {
    console.log('pageshow');
    product_panel_group.cargarBotones()});
*/