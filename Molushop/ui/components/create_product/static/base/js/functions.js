// Mensaje propio en el aviso nativo del navegador (el del `pattern` no dice
// qué formato se espera). Se borra al escribir: si no, el campo seguiría
// marcado como inválido aunque ya estuviera bien.
function handleInvalid(event) {
    let input = event.target;
    if (input.validity.valueMissing) {
        input.setCustomValidity('Escribe el precio.');
    } else {
        input.setCustomValidity('Precio con punto y hasta dos decimales, por ejemplo 12.50');
    }
    input.addEventListener('input', () => input.setCustomValidity(''), { once: true });
}