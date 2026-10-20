// Componente variations_explorer: filtros, tabla y matriz de las variaciones
// de un producto. Los datos están en las filas de la tabla (data-a0…, data-state, data-stock,
// data-price): filtrar es esconder filas, y la matriz se monta con las que
// quedan visibles.
//
// Va dentro de una función para no dejar nombres globales (`el`…) en una
// página del panel que carga más scripts.
(() => {
    document.addEventListener('change', (event) => {
        const root = event.target.closest('.c-variations-explorer');
        if (root && event.target.matches('[data-filter-attr], [data-axis]')) refreshExplorer(root);
        if (root && event.target.matches('[data-bulk-action]')) showBulkField(root);
    });

    document.addEventListener('submit', (event) => {
        const form = event.target.closest('.ve-bulk');
        if (!form) return;
        event.preventDefault();
        submitBulk(form.closest('.c-variations-explorer'), form);
    });

    // Respuesta de la edición en bloque (HX-Trigger): salta en el explorador,
    // que es el `source` de la petición.
    document.addEventListener('variations_bulk_update', (event) => {
        const root = event.target.closest?.('.c-variations-explorer');
        if (!root) return;
        event.detail.rows.forEach((data) => {
            const row = root.querySelector('#' + CSS.escape('var-row-' + data.id));
            if (row) updateRow(row, data);
            // Un editor abierto de una de ellas ya no tiene los datos buenos
            if (row?.classList.contains('is-editing')) closeEditor(root);
        });
        refreshExplorer(root);
    });

    document.addEventListener('click', (event) => {
        const root = event.target.closest('.c-variations-explorer');
        if (!root) return;
        const state = event.target.closest('[data-filter-state]');
        const view = event.target.closest('[data-view]');

        if (state) {
            state.setAttribute('aria-pressed', String(state.getAttribute('aria-pressed') !== 'true'));
        } else if (event.target.closest('[data-filter-reset]')) {
            root.querySelectorAll('[data-filter-attr]').forEach((select) => { select.value = ''; });
            root.querySelectorAll('[data-filter-state]').forEach((b) => b.setAttribute('aria-pressed', 'false'));
        } else if (view) {
            root.querySelectorAll('[data-view]').forEach((b) => b.setAttribute('aria-pressed', String(b === view)));
            root.querySelectorAll('[data-panel]').forEach((panel) => {
                panel.hidden = panel.dataset.panel !== view.dataset.view;
            });
            return;
        } else {
            return;
        }
        refreshExplorer(root);
    });

    function refreshExplorer(root) {
        const attrs = [...root.querySelectorAll('[data-filter-attr]')]
            .filter((select) => select.value)
            .map((select) => [select.dataset.filterAttr, select.value]);
        const states = [...root.querySelectorAll('[data-filter-state][aria-pressed="true"]')]
            .map((b) => b.dataset.filterState);

        const rows = variationRows(root);
        let shown = 0;
        rows.forEach((row) => {
            const visible = attrs.every(([i, value]) => row.dataset['a' + i] === value)
                && (states.length === 0 || states.includes(row.dataset.state));
            row.hidden = !visible;
            if (visible) shown++;
        });
        // El editor abierto se esconde con su fila (sin cerrarlo: no se pierde lo escrito)
        const editorRow = root.querySelector('.ve-editor-row');
        if (editorRow) editorRow.hidden = editorRow.previousElementSibling.hidden;
        // Los atributos fijados por un filtro son iguales en todas las filas: en
        // las tarjetas de móvil se esconden (CSS), en la tabla se quedan.
        const fixed = new Set(attrs.map(([i]) => Number(i)));
        rows.forEach((row) => {
            row.querySelectorAll('td.attr').forEach((td, i) => td.classList.toggle('is-fixed', fixed.has(i)));
        });
        // El contador de arriba y el del botón de la edición en bloque
        root.querySelectorAll('[data-count]').forEach((count) => { count.textContent = shown; });
        const bulkButton = root.querySelector('.ve-bulk [type="submit"]');
        if (bulkButton) bulkButton.disabled = shown === 0;
        root.querySelector('.ve-empty').hidden = shown > 0;

        renderMatrix(root, rows);
    }

    // Si una celda junta varias variaciones (atributos sin fijar), su color es el
    // de la "mejor": basta una activa con stock para que se venda algo.
    const STATE_ORDER = ['is-active', 'is-out', 'is-draft', 'is-inactive'];

    function renderMatrix(root, rows) {
        const container = root.querySelector('[data-matrix]');
        if (!container) return;
        container.replaceChildren();

        const r = root.querySelector('[data-axis="rows"]').value;
        const c = root.querySelector('[data-axis="cols"]').value;
        if (r === c) {
            container.append(el('p', 'm-hint', 'Elige atributos distintos para filas y columnas.'));
            return;
        }

        // Los valores salen de los selectores de filtro: así respetan su orden y,
        // si un eje está filtrado, solo queda ese valor.
        const filterOf = (i) => root.querySelector(`[data-filter-attr="${i}"]`);
        const valuesOf = (i) => {
            const select = filterOf(i);
            return select.value ? [select.value] : [...select.options].slice(1).map((o) => o.value);
        };
        const titleOf = (i) => filterOf(i).closest('label').querySelector('.m-label').textContent;
        const usual = usualPrice(rows);

        const table = el('table', 've-matrix');
        const head = table.createTHead().insertRow();
        head.append(el('th', 've-matrix__corner', titleOf(r) + ' \\ ' + titleOf(c)));
        valuesOf(c).forEach((col) => head.append(el('th', null, col)));

        const body = table.createTBody();
        valuesOf(r).forEach((rowValue) => {
            const tr = body.insertRow();
            tr.append(el('th', null, rowValue));
            valuesOf(c).forEach((colValue) => {
                const match = (row) => row.dataset['a' + r] === rowValue && row.dataset['a' + c] === colValue;
                const cellRows = rows.filter((row) => !row.hidden && match(row));
                tr.append(matrixCell(cellRows, rows.some(match), usual));
            });
        });
        container.append(table);
    }

    function matrixCell(cellRows, exists, usual) {
        if (cellRows.length === 0) {
            // Existe pero los filtros la esconden, o no existe esa combinación.
            return exists
                ? el('td', 've-cell is-filtered', '', 'Oculta por los filtros')
                : el('td', 've-cell is-missing', '—', 'Esta combinación no existe');
        }
        const state = STATE_ORDER.find((s) => cellRows.some((row) => row.dataset.state === s));
        const stock = cellRows.reduce((sum, row) => sum + Number(row.dataset.stock), 0);
        const td = el('td', 've-cell ' + state, null, cellRows.length + ' variación(es) · ' + stock + ' uds');
        td.append(el('strong', null, String(stock)));
        if (cellRows.length > 1) {
            td.append(el('span', 've-cell__count', '×' + cellRows.length));
        } else if (cellRows[0].dataset.price !== usual) {
            const row = cellRows[0];
            const label = row.dataset.price === '' ? 'Sin precio' : MONEY.format(row.dataset.price) + ' ' + row.dataset.symbol;
            td.append(el('span', 've-cell__price', label));
        }
        return td;
    }

    // El precio más repetido del producto: en la matriz solo se escribe el de las
    // celdas que se salen de él.
    function usualPrice(rows) {
        const counts = new Map();
        rows.forEach((row) => counts.set(row.dataset.price, (counts.get(row.dataset.price) || 0) + 1));
        return [...counts].sort((a, b) => b[1] - a[1])[0]?.[0];
    }

    // Los valores vienen del vendedor: se escriben como texto, nunca como HTML.
    function el(tag, className, text, title) {
        const node = document.createElement(tag);
        if (className) node.className = className;
        if (text != null) node.textContent = text;
        if (title) node.title = title;
        return node;
    }

    // ---- Filas: lo que cambia al guardar ----

    // Copia en la fila lo guardado (status, stock, price y/o sku; el que no venga se
    // queda) y recalcula su estado como VariationsExplorer::state_class. No
    // refresca filtros ni matriz: quien llama hace refresh() al terminar.
    function updateRow(row, data) {
        if (data.status !== undefined) row.dataset.status = data.status;
        if (data.sku !== undefined) {
            row.dataset.sku = data.sku;
            renderSkuCell(row);
        }
        if (data.stock !== undefined) {
            row.dataset.stock = data.stock;
            row.querySelector('td.num').textContent = data.stock;
        }
        if (data.price !== undefined) {
            const hasPrice = data.price !== null && data.price !== '';
            row.dataset.price = hasPrice ? data.price : '';
            row.querySelector('td.price .m-price__amount').textContent = hasPrice ? MONEY.format(data.price) : 'No disponible';
            row.querySelector('td.price .m-price__currency').hidden = !hasPrice;
        }
        const status = Number(row.dataset.status);
        const stock = Number(row.dataset.stock);
        const state = status === 1 ? (stock > 0 ? 'is-active' : 'is-out')
            : status === 0 ? 'is-draft' : 'is-inactive';
        const [label, variant] = { 1: ['Activo', 'success'], 0: ['Draft', 'neutral'] }[status] || ['Inactivo', 'danger'];
        // Sin pisar is-editing
        row.classList.remove(...STATE_ORDER);
        row.classList.add(state);
        row.dataset.state = state;
        const badge = el('span', 'm-badge m-badge--' + variant, label);
        row.querySelector('td.state').replaceChildren(badge);
    }

    // Las filas de variación, sin la del editor abierto
    function variationRows(root) {
        return [...root.querySelectorAll('.ve-table tbody tr:not(.ve-editor-row)')];
    }

    // ---- Editor de una variación (solo en la página de edición) ----

    // "Editar": el editor se carga en una fila nueva justo debajo de la suya, que
    // queda resaltada. Solo hay uno abierto; pulsar otra vez en la misma lo cierra.
    function openEditor(button, url) {
        const row = button.closest('tr');
        const root = row.closest('.c-variations-explorer');
        const wasOpen = row.classList.contains('is-editing');
        closeEditor(root);
        if (wasOpen) return;

        const editorRow = el('tr', 've-editor-row');
        const cell = editorRow.insertCell();
        cell.colSpan = row.cells.length;
        row.after(editorRow);
        row.classList.add('is-editing');
        button.setAttribute('aria-expanded', 'true');
        htmx.ajax('GET', url, { target: cell, swap: 'innerHTML' });
    }

    // Vale cualquier elemento del explorador (también el botón "Cerrar" del editor)
    function closeEditor(element) {
        const root = element.closest('.c-variations-explorer');
        root.querySelector('.ve-editor-row')?.remove();
        root.querySelectorAll('tr.is-editing').forEach((row) => {
            row.classList.remove('is-editing');
            row.querySelector('[aria-expanded]')?.setAttribute('aria-expanded', 'false');
        });
    }

    // El editor de una variación (edit-product-variation.js) los usa al guardar y
    // al cerrar; el botón "Editar" de cada fila, openEditor.
    window.variationsExplorer = { updateRow, refresh: refreshExplorer, openEditor, closeEditor };

    // ---- SKU en la tabla (solo en la página de edición) ----
    // "Editar SKU" cambia la columna por campos; "Guardar SKU" manda los que
    // han cambiado (estén o no a la vista) y el servidor guarda todos o ninguno.

    document.addEventListener('click', (event) => {
        const bar = event.target.closest('.ve-sku-bar');
        if (!bar) return;
        const root = bar.closest('.c-variations-explorer');
        if (event.target.closest('[data-sku-edit]')) startSkuEdit(root);
        else if (event.target.closest('[data-sku-cancel]')) stopSkuEdit(root);
        else if (event.target.closest('[data-sku-save]')) saveSkus(root);
    });

    document.addEventListener('input', (event) => {
        if (!event.target.matches('.ve-sku-input')) return;
        event.target.removeAttribute('aria-invalid');
        event.target.removeAttribute('title');
        updateSkuCount(event.target.closest('.c-variations-explorer'));
    });

    // Intro en un campo guarda; Tab pasa a la fila siguiente (orden normal)
    document.addEventListener('keydown', (event) => {
        if (event.key !== 'Enter' || !event.target.matches('.ve-sku-input')) return;
        event.preventDefault();
        saveSkus(event.target.closest('.c-variations-explorer'));
    });

    // Respuestas del servidor (HX-Trigger): saltan en el explorador, el `source`.
    document.addEventListener('variations_skus_saved', (event) => {
        const root = event.target.closest?.('.c-variations-explorer');
        if (!root) return;
        event.detail.rows.forEach((data) => {
            const row = root.querySelector('#' + CSS.escape('var-row-' + data.id));
            if (row) row.dataset.sku = data.sku;
        });
        stopSkuEdit(root);
    });

    document.addEventListener('variations_skus_problems', (event) => {
        const root = event.target.closest?.('.c-variations-explorer');
        if (!root) return;
        showSkuProblems(root, event.detail.problems.map((problem) => ({
            row: root.querySelector('#' + CSS.escape('var-row-' + problem.id)),
            message: problem.message,
        })));
    });

    function renderSkuCell(row) {
        const cell = row.querySelector('td.sku');
        if (row.dataset.sku) cell.textContent = row.dataset.sku;
        else cell.replaceChildren(el('span', 've-none', '—'));
    }

    // "Verde · S · Tela": para los avisos y la etiqueta de cada campo
    function rowLabel(row) {
        return [...row.querySelectorAll('td.attr')].map((td) => td.textContent).join(' · ') || 'Variación única';
    }

    function startSkuEdit(root) {
        closeEditor(root);
        root.classList.add('is-editing-skus');
        variationRows(root).forEach((row) => {
            const input = el('input', 'm-input m-input--sm ve-sku-input');
            input.type = 'text';
            input.value = row.dataset.sku;
            input.maxLength = 64;
            input.autocomplete = 'off';
            input.spellcheck = false;
            input.setAttribute('aria-label', 'SKU de ' + rowLabel(row));
            row.querySelector('td.sku').replaceChildren(input);
        });
        setSkuBar(root, true);
        updateSkuCount(root);
        variationRows(root).find((row) => !row.hidden)?.querySelector('.ve-sku-input').focus();
    }

    // Sale sin guardar: cada celda vuelve a lo que tiene la fila (data-sku)
    function stopSkuEdit(root) {
        root.classList.remove('is-editing-skus');
        variationRows(root).forEach(renderSkuCell);
        setSkuBar(root, false);
    }

    function setSkuBar(root, editing) {
        const bar = root.querySelector('.ve-sku-bar');
        bar.querySelector('[data-sku-edit]').hidden = editing;
        bar.querySelectorAll('[data-sku-save], [data-sku-cancel], [data-sku-changed]').forEach((node) => { node.hidden = !editing; });
        bar.querySelector('[data-sku-error]').hidden = true;
    }

    function changedSkus(root) {
        return variationRows(root)
            .map((row) => ({ row, value: row.querySelector('.ve-sku-input')?.value.trim() }))
            .filter(({ row, value }) => value !== undefined && value !== row.dataset.sku);
    }

    function updateSkuCount(root) {
        const n = changedSkus(root).length;
        const bar = root.querySelector('.ve-sku-bar');
        bar.querySelector('[data-sku-changed]').textContent = n === 0 ? 'Sin cambios' : n === 1 ? '1 cambio' : n + ' cambios';
        bar.querySelector('[data-sku-save]').disabled = n === 0;
    }

    // Marca los campos con problema (sale el motivo al pasar por encima) y
    // los lista bajo la barra.
    function showSkuProblems(root, problems) {
        root.querySelectorAll('.ve-sku-input[aria-invalid]').forEach((input) => {
            input.removeAttribute('aria-invalid');
            input.removeAttribute('title');
        });
        const error = root.querySelector('[data-sku-error]');
        error.replaceChildren();
        problems.forEach(({ row, message }) => {
            const input = row?.querySelector('.ve-sku-input');
            if (input) {
                input.setAttribute('aria-invalid', 'true');
                input.title = message;
            }
            error.append(el('span', null, (row ? rowLabel(row) + ': ' : '') + message));
        });
        error.hidden = problems.length === 0;
    }

    // Lo mismo que comprueba el servidor (sku_service::check_sku_batch), para
    // avisar antes. Los repetidos se buscan en toda la tabla, no solo en los cambiados.
    function saveSkus(root) {
        const changes = changedSkus(root);
        if (changes.length === 0) return;

        const problems = [];
        changes.forEach(({ row, value }) => {
            if (!/^[A-Za-z0-9._-]*$/.test(value)) {
                problems.push({ row, message: 'Solo letras sin tilde, números, guiones (-), guiones bajos (_) y puntos (.)' });
            }
        });
        const byValue = new Map();
        variationRows(root).forEach((row) => {
            const value = row.querySelector('.ve-sku-input').value.trim();
            if (value) byValue.set(value, [...(byValue.get(value) || []), row]);
        });
        byValue.forEach((rows, value) => {
            if (rows.length > 1) rows.forEach((row) => problems.push({ row, message: value + ' está repetido en la tabla' }));
        });
        showSkuProblems(root, problems);
        if (problems.length > 0) return;

        htmx.ajax('POST', root.querySelector('.ve-sku-bar').dataset.skuUrl, {
            source: root,
            target: 'body',
            swap: 'beforeend',
            values: { changes: JSON.stringify(changes.map(({ row, value }) => ({ id: row.id.replace('var-row-', ''), sku: value }))) },
        });
    }

    // ---- Edición en bloque (solo en la página de edición) ----

    const BULK_STATUS = { 1: 'Activo', 2: 'Inactivo', 0: 'Draft' };

    function showBulkField(root) {
        const action = root.querySelector('[data-bulk-action]').value;
        root.querySelectorAll('[data-bulk-for]').forEach((field) => {
            field.hidden = field.dataset.bulkFor !== action;
        });
        root.querySelector('[data-bulk-error]').hidden = true;
    }

    function submitBulk(root, form) {
        const action = form.querySelector('[data-bulk-action]').value;
        const field = form.querySelector(`[data-bulk-for="${action}"]`);
        const input = field.querySelector('select, input');
        const value = input.value.trim();
        const rows = variationRows(root).filter((row) => !row.hidden);
        const error = form.querySelector('[data-bulk-error]');

        const problem = bulkProblem(action, value, input, rows.length);
        error.hidden = !problem;
        if (problem) {
            error.textContent = problem;
            input.focus();
            return;
        }

        const n = rows.length === 1 ? '1 variación' : rows.length + ' variaciones';
        const what = {
            status: `Se pondrá en estado «${BULK_STATUS[value]}» ${n === '1 variación' ? 'la variación filtrada' : 'a ' + n}.`,
            set_stock: `Se pondrán ${value} unidades de stock a ${n}.`,
            add_stock: Number(value) > 0
                ? `Se sumarán ${value} unidades al stock de ${n}.`
                : `Se restarán ${-Number(value)} unidades al stock de ${n} (sin bajar de 0).`,
            price: `Se pondrá el precio ${MONEY.format(value)} € a ${n}.`,
        }[action];

        confirmBulk(what, () => {
            htmx.ajax('POST', form.dataset.bulkUrl, {
                source: root,
                target: 'body',
                swap: 'beforeend',
                values: { action, value, ids: rows.map((row) => row.id.replace('var-row-', '')).join(',') },
            });
        });
    }

    // Lo mismo que valida el servidor (BulkAction::parse), para avisar antes.
    function bulkProblem(action, value, input, count) {
        if (count === 0) return 'Ninguna variación cumple los filtros.';
        if (action === 'status') return null;
        if (value === '' || !input.checkValidity()) {
            return {
                set_stock: 'Indica las unidades: un número entero, 0 o más.',
                add_stock: 'Indica cuántas unidades sumar (o restar, con un número negativo).',
                price: 'Indica el precio: un número, 0 o más, con 2 decimales como mucho.',
            }[action];
        }
        if (action === 'add_stock' && Number(value) === 0) return 'Sumar 0 unidades no cambia nada.';
        return null;
    }

    // Modal de confirmación con el átomo m-modal (la misma estructura que
    // ui/components/modal/templates/shell.html). Se monta aquí y no lo pinta el
    // servidor porque el texto depende de los filtros.
    function confirmBulk(text, onConfirm) {
        const modal = el('div', 'm-modal');
        modal.setAttribute('role', 'dialog');
        modal.setAttribute('aria-modal', 'true');
        const dialog = el('div', 'm-modal__dialog');
        const closeButton = el('button', 'm-modal__close', '×');
        closeButton.type = 'button';
        closeButton.setAttribute('aria-label', 'Cerrar');
        const body = el('div', 'm-modal__body');
        body.append(el('p', null, text));
        const actions = el('div', 'm-modal__actions');
        const no = el('button', 'm-btn m-btn--secondary', 'Cancelar');
        const yes = el('button', 'm-btn m-btn--primary', 'Sí, cambiar');
        no.type = yes.type = 'button';
        actions.append(no, yes);
        dialog.append(closeButton, body, actions);
        const backdrop = el('div', 'm-modal__backdrop');
        modal.append(backdrop, dialog);

        const onKey = (event) => { if (event.key === 'Escape') close(); };
        function close() {
            document.removeEventListener('keyup', onKey);
            modal.classList.add('closing');
            setTimeout(() => modal.remove(), 300);
        }
        [backdrop, closeButton, no].forEach((node) => node.addEventListener('click', close));
        yes.addEventListener('click', () => { close(); onConfirm(); });
        document.addEventListener('keyup', onKey);
        document.body.append(modal);
        no.focus();
    }

    // Al final: STATE_ORDER (const) tiene que existir ya cuando se monta la matriz.
    // htmx.onLoad corre con la página y con cada trozo que htmx mete o restaura
    // (volver atrás desde "Editar"): ahí los selectores vuelven a "Todos" pero las
    // filas guardan su `hidden`, así que se recalcula todo.
    htmx.onLoad((elt) => {
        const explorers = elt.matches?.('.c-variations-explorer') ? [elt] : elt.querySelectorAll('.c-variations-explorer');
        explorers.forEach(refreshExplorer);
    });
})();
