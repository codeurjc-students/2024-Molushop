// Pestaña Imágenes de "Editar producto": "Hacer principal". La principal es la
// de la tarjeta del listado, la primera de la ficha y la de la cabecera del editor.
(() => {
    document.addEventListener('click', (event) => {
        const button = event.target.closest('[data-make-main]');
        if (!button) return;
        const image = button.closest('.image-container');
        htmx.ajax('POST', image.closest('[data-main-image-url]').dataset.mainImageUrl, {
            source: image,
            target: 'body',
            swap: 'beforeend',
            values: { image_id: image.dataset.imageId },
        });
    });

    // Respuesta del servidor (HX-Trigger): salta en la imagen, el `source`.
    document.addEventListener('main_image_update', (event) => {
        const image = event.target.closest?.('.image-container');
        const gallery = image?.closest('.base-images');
        if (!gallery) return;
        gallery.querySelectorAll('.image-container[data-image-id]').forEach((container) => {
            const isMain = container === image;
            container.querySelector('[data-main-badge]').hidden = !isMain;
            container.querySelector('[data-make-main]').hidden = isMain;
        });
        // La principal pasa a la primera, como la ordena el servidor
        gallery.prepend(image);

        // Cabecera del editor: si no tenía imagen, se cambia el hueco por una
        const header = image.closest('.product-edit-container')?.querySelector('.product-main .product-image');
        if (!header) return;
        const src = image.querySelector('img').src;
        if (header.tagName === 'IMG') {
            header.src = src;
        } else {
            const img = document.createElement('img');
            img.className = 'product-image';
            img.src = src;
            img.alt = '';
            header.replaceWith(img);
        }
    });
})();
