// Componente cesta. Los handlers van delegados sobre document porque cada
// modificación reemplaza el nodo #cart-total-1 entero con el HTML que
// devuelve el servidor: si se bindease sobre los botones, se perderían.
const CART_TOTAL_V1 = {
    // Evita que dos peticiones se pisen mientras el usuario aporrea el "+"
    pending: false,
    debounce: null,

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

        // "Seleccionar todos": de momento sólo estado visual, la selección
        // todavía no significa nada en el servidor.
        document.addEventListener("change", (event) => {
            let root = this.root();
            if (root == null || !root.contains(event.target)) { return; }
            if (!event.target.classList.contains("select-all-input")) { return; }

            let checked = event.target.checked;
            root.querySelectorAll(".line-check").forEach(check => {
                check.checked = checked;
            });
        });
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
                    window.location.href = "/master/es/login";
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
