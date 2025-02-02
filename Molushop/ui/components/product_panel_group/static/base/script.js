//esto es un ejemplo
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

document.addEventListener('DOMContentLoaded', function() {
    cargarBotones();
    currentPath = window.location.pathname;
});
//content