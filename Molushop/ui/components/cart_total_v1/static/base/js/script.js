// Componente cesta. Los handlers van delegados sobre document porque cada
// modificación reemplaza el nodo #cart-total-1 entero con el HTML que
// devuelve el servidor: si se bindease sobre los botones, se perderían.
const CART_TOTAL_V1 = {
    // Evita que dos peticiones se pisen mientras el usuario aporrea el "+"
    pending: false,
    debounce: null,

    // Ids de línea DESmarcados. Se guarda lo desmarcado y no lo marcado para que
    // una línea nueva entre seleccionada sin que haya que tocar nada.
    // Vive sólo en el navegador: el servidor recibe la lista de ids en el
    // ?lines= del checkout y recalcula los totales por su cuenta.
    unselected: new Set(),

    root: function() {
        return document.getElementById("cart-total-1");
    },

    init: function() {
        if (this.root() == null) { return; }

        document.addEventListener("click", (event) => {
            let root = this.root();
            if (root == null || !root.contains(event.target)) { return; }

            let less = event.target.closest(".minus-cont");
            let more = event.target.closest(".plus-cont");
            let trash = event.target.closest(".trash");

            if (less)  { this.step(less.closest(".card"), -1); }
            if (more)  { this.step(more.closest(".card"), 1); }
            if (trash) { this.remove(trash.closest(".card")); }
        });

        // Cantidad escrita a mano: se espera a que el usuario deje de teclear
        document.addEventListener("input", (event) => {
            let root = this.root();
            if (root == null || !root.contains(event.target)) { return; }
            if (!event.target.classList.contains("quantity-input")) { return; }

            let card = event.target.closest(".card");
            clearTimeout(this.debounce);
            this.debounce = setTimeout(() => this.applyQuantity(card), 500);
        });

        document.addEventListener("change", (event) => {
            let root = this.root();
            if (root == null || !root.contains(event.target)) { return; }

            let all = event.target.classList.contains("select-all-input");
            let line = event.target.classList.contains("line-check");
            if (!all && !line) { return; }

            if (all) {
                let checked = event.target.checked;
                root.querySelectorAll(".card").forEach(card => {
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

        return Array.from(root.querySelectorAll(".card"))
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
        root.querySelectorAll(".card").forEach(card => {
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

        root.querySelectorAll(".card").forEach(card => {
            let id = card.dataset.lineId;
            if (!id || this.unselected.has(id)) { return; }

            let input = card.querySelector(".quantity-input");
            let quantity = parseInt(input != null ? input.value : "0") || 0;
            let price = parseFloat(card.dataset.price) || 0;

            units += quantity;
            cents += Math.round(price * 100) * quantity;
        });

        let resume = root.querySelector(".data-resume");
        let discount = parseFloat(resume != null ? resume.dataset.discount : "0") || 0;
        let total = cents / 100;

        this.setText(root, ".summary-units", units);
        this.setText(root, ".summary-total", total.toFixed(2));
        this.setText(root, ".summary-final-total", (total - discount).toFixed(2));
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

        let link = root.querySelector(".button-continue .button-1-a");
        let button = root.querySelector(".button-continue .button-1");
        if (link == null) { return; }

        let base = root.dataset.checkoutUrl || "/checkout";
        let ids = this.selectedIds();
        let total = root.querySelectorAll(".card").length;

        link.href = (ids.length === total) ? base : `${base}?lines=${ids.join(",")}`;

        // Sin líneas no hay nada que comprar. No se usa `disabled`, que en un <a>
        // no existe; se marca con la clase y se corta el clic.
        let off = ids.length === 0;
        link.classList.toggle("is-disabled", off);
        if (button != null) { button.classList.toggle("is-disabled", off); }
        if (off) { link.removeAttribute("href"); }
    },

    // Suma delta a la cantidad de la línea y la envía
    step: function(card, delta) {
        if (card == null) { return; }
        let input = card.querySelector(".quantity-input");
        let current = parseInt(input.value) || 0;
        input.value = current + delta;
        clearTimeout(this.debounce);
        this.applyQuantity(card);
    },

    applyQuantity: function(card) {
        if (card == null) { return; }
        let input = card.querySelector(".quantity-input");
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

        this.send(this.root().dataset.updateUrl, {
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
        this.send(url, { method: "DELETE" });
    },

    send: function(url, options) {
        if (this.pending) { return; }
        this.pending = true;

        fetch(url, options)
            .then(response => {
                if (response.status === 401) {
                    // Lo recoge el login_modal y abre el login
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

                if (data.ok) {
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

window.addEventListener("load", () => {
    CART_TOTAL_V1.init();
});
