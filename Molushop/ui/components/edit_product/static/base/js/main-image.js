// Pestaña Imágenes de "Editar producto": "Hacer principal" y lo que cambia al
// borrar una imagen. La principal es la de la tarjeta del listado, la primera
// de la ficha y la de la cabecera del editor.
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

    // Deja `image` como la única principal y la pone la primera, como la
    // ordena el servidor. Sin `image` (no quedan imágenes), solo la cabecera.
    const showAsMain = (gallery, image) => {
        gallery.querySelectorAll('.image-container[data-image-id]').forEach((container) => {
            const isMain = container === image;
            container.querySelector('[data-main-badge]').hidden = !isMain;
            container.querySelector('[data-make-main]').hidden = isMain;
        });
        if (image) gallery.prepend(image);
        updateHeader(gallery, image?.querySelector('img').src);
    };

    // Cabecera del editor: la imagen, o el hueco "Sin imagen" si no hay
    const updateHeader = (gallery, src) => {
        const header = gallery.closest('.product-edit-container')?.querySelector('.product-main .product-image');
        if (!header) return;
        if (src && header.tagName === 'IMG') {
            header.src = src;
            return;
        }
        let replacement;
        if (src) {
            replacement = document.createElement('img');
            replacement.src = src;
            replacement.alt = '';
        } else {
            replacement = document.createElement('div');
            replacement.className = 'product-image--empty';
            replacement.setAttribute('aria-hidden', 'true');
            replacement.textContent = 'Sin imagen';
        }
        replacement.classList.add('product-image');
        header.replaceWith(replacement);
    };

    // Respuesta del servidor (HX-Trigger): salta en la imagen, el `source`.
    document.addEventListener('main_image_update', (event) => {
        const image = event.target.closest?.('.image-container');
        const gallery = image?.closest('.base-images');
        if (!gallery) return;
        showAsMain(gallery, image);
    });

    // Borrar imagen: salta en el "Sí" del modal, fuera de la galería, así que
    // la imagen se busca por su id. `new_main`: la que el servidor ha hecho
    // principal si la borrada lo era.
    document.addEventListener('image_deleted', (event) => {
        const { image_id, new_main } = event.detail;
        const image = document.querySelector(`.base-images .image-container[data-image-id="${image_id}"]`);
        if (!image) return;
        const gallery = image.closest('.base-images');
        const wasMain = !image.querySelector('[data-main-badge]').hidden;
        image.remove();

        if (!gallery.querySelector('.image-container')) {
            const empty = document.createElement('p');
            empty.className = 'no-images';
            empty.textContent = 'No hay imágenes disponibles';
            gallery.append(empty);
        }
        if (new_main) {
            showAsMain(gallery, gallery.querySelector(`.image-container[data-image-id="${new_main}"]`));
        } else if (wasMain) {
            // Era la principal y no quedan otras guardadas
            showAsMain(gallery, null);
        }
    });
})();
