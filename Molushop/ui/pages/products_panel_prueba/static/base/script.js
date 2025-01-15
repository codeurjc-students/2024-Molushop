//esto es un ejemplo
document.querySelectorAll('.toggle-variations').forEach(button => {
    button.addEventListener('click', function() {
        const variations = this.nextElementSibling;
        if (variations.style.display === 'none') {
            variations.style.display = 'block';
            this.textContent = this.textContent.replace('Mostrar', 'Ocultar');
        } else {
            variations.style.display = 'none';
            this.textContent = this.textContent.replace('Ocultar', 'Mostrar');
        }
    });
});