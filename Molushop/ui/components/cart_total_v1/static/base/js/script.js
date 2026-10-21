// Componente cesta. Los handlers van delegados sobre document porque cada
// modificación reemplaza el nodo #cart-total-1 entero con el HTML que
// devuelve el servidor: si se bindease sobre los botones, se perderían.
const CART_TOTAL_V1 = {
    // Una petición cada vez, como hx-sync="queue last" de htmx (el carrito va
    // con fetch, no con htmx): lo que llega mientras hay una en curso espera en
    // `queue`, que guarda lo último de cada línea (un borrado le gana a un
    // cambio de cantidad de la misma). Antes se descartaba y se perdían clics.
    pending: false,
    queue: new Map(),
    // Un temporizador por línea: con uno solo, cambiar dos líneas seguidas
    // dejaba la primera sin mandar.
    timers: new Map(),

    // Ids de línea DESmarcados. Se guarda lo desmarcado y no lo marcado para que
    // una línea nueva entre seleccionada sin que haya que tocar nada.
    // Vive sólo en el navegador: el servidor recibe la lista de ids en el
    // ?lines= del checkout y recalcula los totales por su cuenta.
    unselected: new Set(),

    root: function() {
        return document.getElementById("cart-total-1");
    },

    // Las líneas que se pueden comprar. Las que ya no se venden (.is-unavailable)
    // no tienen casilla ni cuentan en el resumen; para el enlace de "Continuar"
    // sí cuentan, y así su presencia obliga a mandar `?lines=`.
    cards: function(root) {
        return root.querySelectorAll(".card:not(.is-unavailable)");
    },

    // Los oyentes, una sola vez al cargar el fichero: buscan el carrito en
    // cada evento, así que valen para el que haya (con hx-boost, uno nuevo
    // cada vez que se entra).
    init: function() {
        document.addEventListener("click", (event) => {
            let root = this.root();
            if (root == null || !root.contains(event.target)) { return; }

            let trash = event.target.closest(".trash");
            if (trash) { this.remove(trash.closest(".card")); }
        });

        // − y + del selector (átomo QUANTITY, base.js): ya ha cambiado el
        // input sin salirse de 0..stock. Varios clics seguidos van en una sola
        // petición con el valor final.
        document.addEventListener("m-qty-change", (event) => {
            let root = this.root();
            if (root == null || !root.contains(event.target)) { return; }
            this.later(event.target.closest(".card"), 300);
        });

        // Cantidad escrita a mano: se espera a que el usuario deje de teclear
        document.addEventListener("input", (event) => {
            let root = this.root();
            if (root == null || !root.contains(event.target)) { return; }
            if (!event.target.classList.contains("m-qty__input")) { return; }
            this.later(event.target.closest(".card"), 500);
        });

        document.addEventListener("change", (event) => {
            let root = this.root();
            if (root == null || !root.contains(event.target)) { return; }

            let all = event.target.classList.contains("select-all-input");
            let line = event.target.classList.contains("line-check");
            if (!all && !line) { return; }

            if (all) {
                let checked = event.target.checked;
                this.cards(root).forEach(card => {
                    let check = card.querySelector(".line-check");
                    if (check != null) { check.checked = checked; }
                    this.mark(card.dataset.lineId, checked);
                });
            } else {
                let card = event.target.closest(".card");
                if (card != null) { this.mark(card.dataset.lineId, event.target.checked); }
            }

            this.refreshSelection();
        });
    },

    // Una vez por carrito (COMPONENT.mount, base.js). El servidor lo pinta
    // todo marcado: se olvida lo desmarcado en una visita anterior.
    mount: function() {
        this.unselected.clear();
        this.refreshSelection();
    },

    // Apunta (o desapunta) una línea como desmarcada
    mark: function(lineId, checked) {
        if (!lineId) { return; }
        if (checked) { this.unselected.delete(lineId); }
        else { this.unselected.add(lineId); }
    },

    // Devuelve los ids de las líneas marcadas, en el orden en que se ven
    selectedIds: function() {
        let root = this.root();
        if (root == null) { return []; }

        return Array.from(this.cards(root))
            .map(card => card.dataset.lineId)
            .filter(id => id && !this.unselected.has(id));
    },

    // Vuelve a poner los checkboxes como los dejó el usuario. Hay que llamarla
    // después de cada refresco parcial: send() reemplaza #cart-total-1 entero
    // con el HTML del servidor, que los devuelve todos marcados.
    applySelection: function() {
        let root = this.root();
        if (root == null) { return; }

        let ids = new Set();
        this.cards(root).forEach(card => {
            let id = card.dataset.lineId;
            if (!id) { return; }
            ids.add(id);

            let check = card.querySelector(".line-check");
            if (check != null) { check.checked = !this.unselected.has(id); }
        });

        // Una línea borrada no tiene por qué seguir ocupando sitio en el Set
        this.unselected.forEach(id => {
            if (!ids.has(id)) { this.unselected.delete(id); }
        });

        let all = root.querySelector(".select-all-input");
        if (all != null) {
            all.checked = ids.size > 0 && this.unselected.size === 0;
        }
    },

    // Reaplica la selección, repinta el resumen y sincroniza el botón
    refreshSelection: function() {
        this.applySelection();
        this.updateSummary();
        this.updateContinue();
    },

    // Recalcula unidades y totales con lo marcado. Es cosmético: los totales
    // que valen son los que recalcula el servidor en el checkout.
    updateSummary: function() {
        let root = this.root();
        if (root == null) { return; }

        let units = 0;
        // En céntimos, para no arrastrar los errores del coma flotante
        let cents = 0;

        this.cards(root).forEach(card => {
            let id = card.dataset.lineId;
            if (!id || this.unselected.has(id)) { return; }

            let input = card.querySelector(".m-qty__input");
            let quantity = parseInt(input != null ? input.value : "0") || 0;
            let price = parseFloat(card.dataset.price) || 0;

            units += quantity;
            cents += Math.round(price * 100) * quantity;
        });

        let resume = root.querySelector(".data-resume");
        let discount = parseFloat(resume != null ? resume.dataset.discount : "0") || 0;
        let total = cents / 100;

        this.setText(root, ".summary-units", units);
        // Importe del átomo m-price, en el mismo formato que lo pinta el servidor
        this.setText(root, ".summary-total .m-price__amount", MONEY.format(total));
        this.setText(root, ".summary-final-total .m-price__amount", MONEY.format(total - discount));
    },

    setText: function(root, selector, value) {
        let node = root.querySelector(selector);
        if (node != null) { node.textContent = value; }
    },

    // El enlace arrastra sólo lo marcado. Con todo marcado va a /checkout a
    // secas, que es lo mismo que hace el enlace cuando no hay JS.
    updateContinue: function() {
        let root = this.root();
        if (root == null) { return; }

        let link = root.querySelector(".button-continue .m-btn");
        if (link == null) { return; }

        let base = root.dataset.checkoutUrl || "/checkout";
        let ids = this.selectedIds();
        let total = root.querySelectorAll(".card").length;

        link.href = (ids.length === total) ? base : `${base}?lines=${ids.join(",")}`;

        // Sin líneas no hay nada que comprar. No se usa `disabled`, que en un <a>
        // no existe; se marca con la clase y se corta el clic.
        let off = ids.length === 0;
        link.classList.toggle("is-disabled", off);
        link.setAttribute("aria-disabled", off);
        if (off) { link.removeAttribute("href"); }
    },

    // Manda la cantidad de la línea cuando lleve `ms` sin cambiar
    later: function(card, ms) {
        if (card == null) { return; }
        let id = card.dataset.lineId;
        clearTimeout(this.timers.get(id));
        this.timers.set(id, setTimeout(() => {
            this.timers.delete(id);
            this.applyQuantity(card);
        }, ms));
    },

    applyQuantity: function(card) {
        if (card == null) { return; }
        let input = card.querySelector(".m-qty__input");
        let stock = parseInt(card.dataset.stock) || 0;
        let quantity = parseInt(input.value);

        if (isNaN(quantity)) { return; }
        // El servidor vuelve a validar; esto sólo evita peticiones inútiles
        if (quantity > stock) {
            input.value = stock;
            quantity = stock;
        }
        if (quantity < 0) {
            input.value = 0;
            quantity = 0;
        }

        this.send(card.dataset.lineId, this.root().dataset.updateUrl, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                cart_product_id: card.dataset.lineId,
                quantity: quantity
            })
        });
    },

    remove: function(card) {
        if (card == null) { return; }
        let url = `${this.root().dataset.removeUrl}/${card.dataset.lineId}`;
        clearTimeout(this.timers.get(card.dataset.lineId));
        this.timers.delete(card.dataset.lineId);
        this.send(card.dataset.lineId, url, { method: "DELETE" });
    },

    // `key`: la línea. Si hay una petición en curso, espera en la cola.
    send: function(key, url, options) {
        if (this.pending) {
            this.queue.set(key, { url, options });
            return;
        }
        this.pending = true;

        fetch(url, options)
            .then(response => {
                if (response.status === 401) {
                    // Lo recoge el login_modal y abre el login. Lo que esperaba
                    // en la cola también daría 401: se descarta.
                    this.queue.clear();
                    document.dispatchEvent(new CustomEvent("auth-required"));
                    return null;
                }
                let cartCount = response.headers.get("X-Cart-Count");
                return response.text().then(html => ({
                    html,
                    cartCount,
                    ok: response.ok
                }));
            })
            .then(data => {
                if (data === null) { return; }

                // Con algo en cola, esta respuesta ya está vieja: repintar
                // devolvería el número de antes de los últimos clics.
                if (data.ok && this.queue.size > 0) {
                    this.updateBadge(data.cartCount);
                } else if (data.ok) {
                    this.root().outerHTML = data.html;
                    this.updateBadge(data.cartCount);
                    // El HTML nuevo viene con todo marcado: hay que devolverle
                    // al usuario la selección que tenía
                    this.refreshSelection();
                } else {
                    // El cuerpo del error es un modal ya renderizado
                    this.showModal(data.html);
                }
            })
            .catch(err => {
                console.error("Error al actualizar el carrito:", err);
            })
            .finally(() => {
                this.pending = false;
                let next = this.queue.entries().next().value;
                if (next) {
                    let [key, request] = next;
                    this.queue.delete(key);
                    this.send(key, request.url, request.options);
                }
            });
    },

    updateBadge: function(cartCount) {
        if (cartCount === null) { return; }
        let badge = document.getElementById("cart-badge");
        if (badge == null) { return; }

        badge.textContent = cartCount;
        badge.style.display = parseInt(cartCount) > 0 ? "flex" : "none";
    },

    showModal: function(html) {
        document.body.insertAdjacentHTML("beforeend", html);
        let modal = document.getElementById("myModal");
        if (modal && typeof _hyperscript !== "undefined") {
            _hyperscript.processNode(modal);
        }
    }
};

CART_TOTAL_V1.init();
COMPONENT.mount("#cart-total-1", () => CART_TOTAL_V1.mount());
