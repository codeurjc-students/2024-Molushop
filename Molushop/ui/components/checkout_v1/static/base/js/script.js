// Componente checkout. De momento no hace nada: el formulario es un POST normal
// y la validación de campos obligatorios la hace el propio navegador con
// `required`. El endpoint que lo recibe todavía no existe (parte 3 del paso 6).

// "Confirmar pedido" se envía una sola vez: al pulsar, el botón se apaga y
// avisa. El servidor ya impide comprar dos veces el mismo carrito (FOR UPDATE
// en checkout_confirm_service); esto evita que salga la segunda petición. El
// evento submit solo llega si el navegador ha dado por buenos los `required`.
document.addEventListener('submit', (event) => {
    const form = event.target.closest('.checkout-body');
    if (!form) return;
    if (form.dataset.sending === 'true') {
        event.preventDefault();
        return;
    }
    form.dataset.sending = 'true';
    const button = form.querySelector('.confirm-button');
    button.disabled = true;
    button.setAttribute('aria-busy', 'true');
    button.textContent = 'Confirmando pedido…';
});

// Al volver atrás con el navegador la página sale de la caché tal cual,
// con el botón apagado: se deja como al principio.
window.addEventListener('pageshow', (event) => {
    if (!event.persisted) return;
    const form = document.querySelector('.checkout-body');
    if (!form) return;
    delete form.dataset.sending;
    const button = form.querySelector('.confirm-button');
    button.disabled = false;
    button.removeAttribute('aria-busy');
    button.textContent = 'Confirmar pedido';
});
