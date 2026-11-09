// Pestaña Imágenes de "Editar producto": en qué variaciones sale cada imagen.
// Se elige un valor por atributo ("Cualquiera" si da igual): Color = Red sale
// en todas las rojas, y con todo en "Cualquiera" la imagen es del producto
// entero. Al servidor van los ids de las variaciones que encajan.
//
// Las variaciones (ids y valores) se leen de las filas del explorador de la
// pestaña Variaciones, que está en la misma página (variations_explorer).
(() => {
    document.addEventListener('click', (event) => {
        const button = event.target.closest('[data-image-variations]');
        if (button) openDialog(button.closest('.image-container'));
    });

    // Respuesta del servidor (HX-Trigger): salta en la imagen, el `source`.
    document.addEventListener('image_variations_update', (event) => {
        const image = event.target.closest?.('.image-container');
        if (!image) return;
        image.dataset.variations = event.detail.ids.join(',');
        renderLabel(image);
    });

    htmx.onLoad((elt) => {
        const images = elt.matches?.('[data-image-id]') ? [elt] : elt.querySelectorAll('[data-image-id]');
        images.forEach(renderLabel);
    });

    // Atributos (título y valores, en el orden de los filtros) y variaciones
    // (id y valor de cada atributo) del explorador del producto.
    function variationsOf(image) {
        const explorer = image.closest('.product-edit-container')?.querySelector('.c-variations-explorer');
        if (!explorer) return null;
        const attrs = [...explorer.querySelectorAll('[data-filter-attr]')].map((select) => ({
            index: select.dataset.filterAttr,
            title: select.closest('label').querySelector('.m-label').textContent,
            values: [...select.options].slice(1).map((option) => option.value),
        }));
        const rows = [...explorer.querySelectorAll('.ve-table tbody tr[id^="var-row-"]')].map((row) => ({
            id: row.id.slice('var-row-'.length),
            values: attrs.map((attr) => row.dataset['a' + attr.index]),
        }));
        return { attrs, rows };
    }

    function linkedIds(image) {
        return image.dataset.variations ? image.dataset.variations.split(',') : [];
    }

    // Las variaciones que encajan con un valor por atributo ('' = cualquiera)
    function matching(data, chosen) {
        return data.rows.filter((row) => chosen.every((value, i) => !value || row.values[i] === value));
    }

    // Cómo se lee lo guardado: si los enlaces son justo "las de Color = Red",
    // se describe así y el diálogo abre con esos valores; si son sueltas
    // (no salen de un valor por atributo), solo cuántas.
    function describe(data, ids) {
        const none = data.attrs.map(() => '');
        if (ids.length === 0) return { label: 'Todas', chosen: none };
        const set = new Set(ids);
        const linked = data.rows.filter((row) => set.has(row.id));
        const chosen = data.attrs.map((_, i) => {
            const values = new Set(linked.map((row) => row.values[i]));
            return values.size === 1 ? [...values][0] : '';
        });
        if (linked.length === ids.length && matching(data, chosen).length === linked.length) {
            const parts = data.attrs.map((attr, i) => chosen[i] && attr.title + ': ' + chosen[i]).filter(Boolean);
            return { label: parts.join(' · ') || 'Todas', chosen };
        }
        return { label: ids.length === 1 ? '1 variación' : ids.length + ' variaciones', chosen: none, loose: true };
    }

    function renderLabel(image) {
        const button = image.querySelector('[data-image-variations]');
        const data = variationsOf(image);
        if (!button || !data) return;
        const { label } = describe(data, linkedIds(image));
        button.textContent = label;
        button.classList.toggle('is-linked', label !== 'Todas');
    }

    function openDialog(image) {
        const data = variationsOf(image);
        if (!data) return;
        const current = describe(data, linkedIds(image));

        const body = el('div', 'm-modal__body image-variations-dialog');
        body.append(el('h3', 'image-variations-dialog__title', 'En qué variaciones sale esta imagen'));
        const preview = el('img', 'image-variations-dialog__img');
        preview.src = image.querySelector('img').src;
        preview.alt = '';
        body.append(preview);
        if (current.loose) {
            body.append(el('p', 'm-hint', `Ahora sale en ${current.label} sueltas. Al guardar se sustituyen por lo que elijas aquí.`));
        }

        const selects = data.attrs.map((attr, i) => {
            const field = el('label', 'm-field');
            field.append(el('span', 'm-label', attr.title));
            const select = el('select', 'm-input');
            select.append(new Option('Cualquiera', ''));
            attr.values.forEach((value) => select.append(new Option(value, value, false, value === current.chosen[i])));
            field.append(select);
            body.append(field);
            return select;
        });
        const summary = el('p', 'image-variations-dialog__summary');
        summary.setAttribute('aria-live', 'polite');
        body.append(summary);

        const chosen = () => selects.map((select) => select.value);
        const refresh = () => {
            const n = matching(data, chosen()).length;
            summary.textContent = chosen().every((value) => !value)
                ? 'Imagen del producto: sale en todas las variaciones.'
                : n === 0
                    ? 'Ninguna variación tiene esa combinación.'
                    : `Sale en ${n === 1 ? '1 variación' : n + ' variaciones'} de ${data.rows.length}.`;
            save.disabled = n === 0;
        };

        const actions = el('div', 'm-modal__actions');
        const cancel = el('button', 'm-btn m-btn--secondary', 'Cancelar');
        const save = el('button', 'm-btn m-btn--primary', 'Guardar');
        cancel.type = save.type = 'button';
        actions.append(cancel, save);

        const close = openModal(body, actions, cancel);
        selects.forEach((select) => select.addEventListener('change', refresh));
        refresh();

        save.addEventListener('click', () => {
            // Todo en "Cualquiera": sin enlaces (imagen del producto entero)
            const ids = chosen().every((value) => !value) ? [] : matching(data, chosen()).map((row) => row.id);
            close();
            htmx.ajax('POST', image.closest('[data-image-variations-url]').dataset.imageVariationsUrl, {
                source: image,
                target: 'body',
                swap: 'beforeend',
                values: { image_id: image.dataset.imageId, ids: ids.join(',') },
            });
        });
    }

    // Átomo m-modal, con la estructura de ui/components/modal/templates/shell.html.
    // Devuelve la función que lo cierra.
    function openModal(body, actions, focus) {
        const modal = el('div', 'm-modal');
        modal.setAttribute('role', 'dialog');
        modal.setAttribute('aria-modal', 'true');
        const dialog = el('div', 'm-modal__dialog');
        const closeButton = el('button', 'm-modal__close', '×');
        closeButton.type = 'button';
        closeButton.setAttribute('aria-label', 'Cerrar');
        dialog.append(closeButton, body, actions);
        const backdrop = el('div', 'm-modal__backdrop');
        modal.append(backdrop, dialog);

        const onKey = (event) => { if (event.key === 'Escape') close(); };
        function close() {
            document.removeEventListener('keyup', onKey);
            modal.classList.add('closing');
            setTimeout(() => modal.remove(), 300);
        }
        [backdrop, closeButton, focus].forEach((node) => node.addEventListener('click', close));
        document.addEventListener('keyup', onKey);
        document.body.append(modal);
        focus.focus();
        return close;
    }

    // Los valores vienen del vendedor: se escriben como texto, nunca como HTML.
    function el(tag, className, text) {
        const node = document.createElement(tag);
        if (className) node.className = className;
        if (text != null) node.textContent = text;
        return node;
    }
})();
