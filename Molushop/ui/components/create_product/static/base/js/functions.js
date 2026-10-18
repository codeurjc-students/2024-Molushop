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
// Variaciones del alta de producto (`templates/base-producto.html`). El
// formulario llega por htmx después de cargar este script, así que los botones
// se escuchan desde `document`.
document.addEventListener('click', (event) => {
    const button = event.target.closest('[data-variations]');
    if (!button) return;
    const root = button.closest('.variations');

    switch (button.dataset.variations) {
        case 'add-variation': {
            const variation = cloneTemplate(root, '.tpl-variation');
            variation.querySelector('.variation-values').append(cloneTemplate(root, '.tpl-value'));
            root.querySelector('.variations-list').append(variation);
            variation.querySelector('input').focus();
            break;
        }
        case 'add-value': {
            const value = cloneTemplate(root, '.tpl-value');
            button.closest('.variation').querySelector('.variation-values').append(value);
            value.querySelector('input').focus();
            break;
        }
        case 'remove-variation':
            button.closest('.variation').remove();
            break;
        case 'remove-value': {
            // Una variación sin valores no genera combinaciones: se quita entera.
            const values = button.closest('.variation-values');
            if (values.children.length > 1) button.closest('.variation-value').remove();
            else button.closest('.variation').remove();
            break;
        }
    }
    renumberVariations(root);
});

function cloneTemplate(root, selector) {
    return root.querySelector(selector).content.firstElementChild.cloneNode(true);
}

// Pone los `name` que espera json-enc-custom según la posición de cada campo,
// y los `id` para que cada etiqueta apunte a su campo.
function renumberVariations(root) {
    root.querySelectorAll('.variations-list > .variation').forEach((variation, i) => {
        const name = variation.querySelector('[data-field="name"]');
        name.name = `variations[${i}][name]`;
        name.id = `variation-${i}-name`;
        variation.querySelector('label').htmlFor = name.id;

        variation.querySelectorAll('[data-field="value"]').forEach((value, j) => {
            value.name = `variations[${i}][values][${j}][value]`;
            value.setAttribute('aria-label', `Valor ${j + 1}`);
        });
    });
}
