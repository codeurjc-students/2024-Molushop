// Buscador del listado de productos del panel: filtra las tarjetas en el
// navegador por nombre y por SKU de sus variaciones (data-search-name y
// data-search-skus). Sin tildes ni mayúsculas, y cada palabra tiene que
// aparecer en alguno de los dos (las palabras no tienen que ir seguidas).
// La búsqueda se guarda en la URL (?q=) para que siga al volver de "Editar".
(() => {
    document.addEventListener('input', (event) => {
        if (!event.target.matches('[data-product-search]')) return;
        const group = event.target.closest('.c-product-panel-group');
        filterProducts(group);
        const url = new URL(location.href);
        if (event.target.value.trim()) url.searchParams.set('q', event.target.value.trim());
        else url.searchParams.delete('q');
        history.replaceState(history.state, '', url);
    });

    function fold(text) {
        return text.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase();
    }

    function filterProducts(group) {
        const input = group.querySelector('[data-product-search]');
        const words = fold(input.value).split(/\s+/).filter(Boolean);
        const cards = [...group.querySelectorAll('.c-product-panel-element')];
        let shown = 0;
        cards.forEach((card) => {
            const name = fold(card.dataset.searchName);
            const skus = card.dataset.searchSkus.split(' ').filter(Boolean);
            const visible = words.every((word) => name.includes(word) || skus.some((sku) => fold(sku).includes(word)));
            card.hidden = !visible;
            if (visible) shown++;

            // Encaja por SKU (alguna palabra no está en el nombre): cuáles
            const byName = words.every((word) => name.includes(word));
            const matched = visible && !byName
                ? skus.filter((sku) => words.some((word) => fold(sku).includes(word)))
                : [];
            const hint = card.querySelector('[data-sku-match]');
            hint.hidden = matched.length === 0;
            hint.textContent = matched.length > 3
                ? 'SKU: ' + matched.slice(0, 3).join(', ') + ' y ' + (matched.length - 3) + ' más'
                : 'SKU: ' + matched.join(', ');
        });
        group.querySelector('[data-search-count]').textContent = words.length === 0
            ? ''
            : `${shown} de ${cards.length} productos`;
        group.querySelector('[data-search-empty]').hidden = shown > 0;
    }

    // Con la página y con lo que htmx mete o restaura (volver desde "Editar"):
    // la búsqueda sale de la URL.
    htmx.onLoad((elt) => {
        const groups = elt.matches?.('.c-product-panel-group') ? [elt] : elt.querySelectorAll('.c-product-panel-group');
        groups.forEach((group) => {
            const input = group.querySelector('[data-product-search]');
            if (!input) return;
            input.value = new URLSearchParams(location.search).get('q') || '';
            filterProducts(group);
        });
    });
})();
