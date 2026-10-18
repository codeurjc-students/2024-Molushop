const PRODUCT_DETAIL = {
    element: null,
    init: function() {
        let principal = document.querySelector(".product-detail-principal");
        if (principal == null) { return; }
        this.element = principal;

        let main_image_container = principal.querySelector(".main-image-container");
        let more_images_container = principal.querySelector(".all-images");

        more_images_container.style.height = `${main_image_container.offsetHeight}px`;

        window.addEventListener("resize", () => {
            more_images_container.style.height = `${main_image_container.offsetHeight}px`;
        });

        let main_image = main_image_container.querySelector(".image-principal");
        let mini_images = more_images_container.querySelectorAll('.image-s');
        mini_images.forEach((image_cont) => {
            image_cont.addEventListener("mouseenter", () => {
                let image_url = image_cont.querySelector(".image-mini").src;
                main_image.src = image_url;
            });
        });

        // El selector de cantidad (− +) lo mueve el átomo (QUANTITY, base.js);
        // aquí solo se lee su input al pulsar "Añadir al carrito".
        this.init_product_selection();
        this.refreshAvailability();
        this.element.addEventListener("m-qty-change", () => this.showError(null));

        document.addEventListener("auth-cancelled", () => this.clearPendingAdd());
        this.resumePendingAdd();
    },

    // Devuelve { attrName: value } de los radios actualmente marcados
    getSelectedAttrs: function() {
        let selected = {};
        this.element.querySelectorAll(".variation-input:checked").forEach(radio => {
            let attrName = radio.name.replace("variation-", "");
            selected[attrName] = radio.value;
        });
        return selected;
    },

    // Devuelve las variaciones de variantMap que contienen el atributo name=attrName con value=attrValue,
    // primero las que tienen stock: al saltar de combinación se elige una que se pueda comprar.
    getVariantsByAttr: function(attrName, attrValue) {
        return variantMap
            .filter(variant => variant.attributes.some(a => a.name === attrName && a.value === attrValue))
            .sort((a, b) => (b.stock > 0) - (a.stock > 0));
    },

    // Devuelve un Set con los valores de targetAttrName que, con el resto de lo
    // seleccionado, dan una variación CON stock (para tachar los que no).
    getAvailableValues: function(selectedAttrs, targetAttrName) {
        let available = new Set();
        variantMap.forEach(variant => {
            if (variant.stock <= 0) return;

            let matches = Object.entries(selectedAttrs).every(([name, value]) => {
                if (name === targetAttrName) return true;
                let attr = variant.attributes.find(a => a.name === name);
                return attr && attr.value === value;
            });

            if (matches) {
                let targetAttr = variant.attributes.find(a => a.name === targetAttrName);
                if (targetAttr) available.add(targetAttr.value);
            }
        });
        return available;
    },

    // Al elegir un valor: si la combinación existe y tiene stock, se queda; si
    // no, se salta a la primera con stock que lleve ese valor (o, si no hay
    // ninguna, a la primera que exista). Después se repintan precio, píldoras y botón.
    updateAvailableOptions: function(event) {
        let target = event.currentTarget;
        let selectedAttrs = this.getSelectedAttrs();
        let variation = target.name.replace("variation-", "");
        let variationValue = target.value;

        let availableValues = this.getVariantsByAttr(variation, variationValue);

        let current = this.findVariantByAttrs(selectedAttrs);

        if (current && current.stock > 0) {
            this.updatePrice(selectedAttrs);
        } else if (availableValues && availableValues.length > 0) {
            let firstVariant = availableValues[0];
            firstVariant.attributes.forEach(attr => {
                if (attr.name === variation) return;
                let input = this.element.querySelector(`input[name="variation-${attr.name}"][value="${attr.value}"]`);
                if (input) {
                    input.checked = true;
                    selectedAttrs[attr.name] = attr.value;
                }
            });
            this.updatePrice(selectedAttrs);
        }
        this.refreshAvailability();
    },

    // Tacha (sin apagar: se puede pulsar para cambiar de combinación) los
    // valores que, con lo demás elegido, solo llevan a variaciones agotadas.
    // Los que no tienen stock en ninguna combinación ya vienen `disabled` del servidor.
    refreshAvailability: function() {
        this.showError(null);
        let selectedAttrs = this.getSelectedAttrs();
        let byAttr = {};
        this.element.querySelectorAll(".variation-input").forEach(input => {
            let attrName = input.name.replace("variation-", "");
            if (!(attrName in byAttr)) {
                byAttr[attrName] = this.getAvailableValues(selectedAttrs, attrName);
            }
            let label = this.element.querySelector(`label[for="${CSS.escape(input.id)}"]`);
            if (label) { label.classList.toggle("is-out-of-stock", !byAttr[attrName].has(input.value)); }
        });
        this.refreshAddButton();
    },

    // "Agotado" y apagado si la combinación elegida no existe o no tiene stock.
    refreshAddButton: function() {
        let btn = this.element.querySelector(".add-to-cart-btn");
        if (btn == null) { return; }
        let variant = this.findVariantByAttrs(this.getSelectedAttrs());
        let soldOut = variant == null || variant.stock <= 0;
        btn.disabled = soldOut;
        let text = btn.querySelector("span");
        if (text) { text.textContent = soldOut ? "Agotado" : "Añadir al carrito"; }
    },
    // Sin atributos seleccionados solo encaja la variación única de un producto
    // sin variaciones (attributes = []); en uno con variaciones no encaja ninguna.
    findVariantByAttrs: function(attrsJson) {
      let attrs = (typeof attrsJson === "string") ? JSON.parse(attrsJson) : attrsJson;
      let attrNames = Object.keys(attrs);

      return variantMap.find(variant =>
          attrNames.length === variant.attributes.length &&
          attrNames.every(name => {
              let attr = variant.attributes.find(a => a.name === name);
              return attr && attr.value === attrs[name];
          })
      ) ?? null;
    },
    // Busca la variante exacta y actualiza el precio en pantalla
    updatePrice: function(selectedAttrs) {
        let attrNames = Object.keys(selectedAttrs);
        if (attrNames.length === 0) return;

        let match = variantMap.find(variant =>
            attrNames.every(name => {
                let attr = variant.attributes.find(a => a.name === name);
                return attr && attr.value === selectedAttrs[name];
            })
        );

        // Átomo m-price: se cambia el importe y se esconde la moneda si no hay.
        let priceEl = document.querySelector("#product-price .m-price__amount");
        let currencyEl = document.querySelector("#product-price .m-price__currency");
        if (priceEl && match) {
            // price llega null si la variación no tiene fila en prices: se
            // enseña como no disponible en vez de "null €".
            let hasPrice = match.price != null;
            priceEl.textContent = hasPrice ? MONEY.format(match.price) : "No disponible";
            if (currencyEl) { currencyEl.hidden = !hasPrice; }
        }
    },

    // El aviso bajo el botón (null lo esconde). Se limpia al cambiar de
    // variación o de cantidad: ya no se refiere a lo que hay elegido.
    showError: function(message) {
        let el = this.element.querySelector(".add-to-cart-error");
        if (el == null) { return; }
        el.textContent = message || "";
        el.hidden = !message;
    },

    addToCart: function() {
        this.showError(null);
        let selectedAttrs = this.getSelectedAttrs();
        let variant = this.findVariantByAttrs(selectedAttrs);

        if (!variant) {
            this.showError("Selecciona una variación válida.");
            return;
        }
        // El servidor la rechazaría igual ("No price found"); así el aviso se
        // entiende y no se hace la petición.
        if (variant.price == null) {
            this.showError("Esta variación no está disponible ahora mismo.");
            return;
        }

        let quantityInput = this.element.querySelector(".quantity-product .m-qty__input");
        let quantity = parseInt(quantityInput.value) || 1;

        if (quantity <= 0) {
            this.showError("La cantidad debe ser mayor que 0.");
            return;
        }

        let btn = this.element.querySelector(".add-to-cart-btn");
        let url = btn.dataset.addUrl;

        fetch(url, {
            method: "POST",
            headers: { "Content-Type": "application/json", "HX-Request": "true" },
            body: JSON.stringify({
                product_var_id: variant.id,
                quantity: quantity
            })
        })
        .then(response => {
            if (response.status === 401) {
                // Se guarda lo que quería añadir para reintentarlo cuando vuelva
                // identificado (el login recarga la página y se perdería).
                this.savePendingAdd(variant.id, quantity);
                // Lo recoge el login_modal y abre el login
                document.dispatchEvent(new CustomEvent("auth-required"));
                return null;
            }
            let cartCount = response.headers.get("X-Cart-Count");
            return response.text().then(html => ({ html, cartCount }));
        })
        .then(data => {
            if (data === null) return;
            document.body.insertAdjacentHTML('beforeend', data.html);
            let modal = document.getElementById('myModal');
            if (modal && typeof _hyperscript !== 'undefined') {
                _hyperscript.processNode(modal);
            }
            if (data.cartCount) {
                let badge = document.getElementById('cart-badge');
                if (badge) {
                    badge.textContent = data.cartCount;
                    badge.style.display = 'flex';
                }
            }
        })
        .catch(err => {
            console.error("Error al añadir al carrito:", err);
        });
    },

    // --- Añadido pendiente tras el login ---
    // Si "Añadir al carrito" recibe un 401 se guarda aquí qué quería añadir. El login
    // recarga la página y, al volver identificado, se reintenta solo. Se descarta si
    // cierra el login sin identificarse (auth-cancelled), si ha pasado demasiado tiempo
    // o si al volver está en otra ficha.
    PENDING_ADD_KEY: "pending_cart_add",
    PENDING_ADD_MAX_AGE_MS: 10 * 60 * 1000,

    savePendingAdd: function(productVarId, quantity) {
        try {
            sessionStorage.setItem(this.PENDING_ADD_KEY, JSON.stringify({
                path: location.pathname,
                product_var_id: productVarId,
                quantity: quantity,
                saved_at: Date.now()
            }));
        } catch (e) {
            // Sin sessionStorage (modo privado estricto): no hay reintento, como antes
        }
    },

    clearPendingAdd: function() {
        try { sessionStorage.removeItem(this.PENDING_ADD_KEY); } catch (e) {}
    },

    resumePendingAdd: function() {
        if (document.body.dataset.user_logged !== "true") { return; }

        let pending = null;
        try { pending = JSON.parse(sessionStorage.getItem(this.PENDING_ADD_KEY)); } catch (e) {}
        if (pending == null) { return; }
        // Se borra ANTES de reintentar: si el reintento falla no debe repetirse en cada recarga
        this.clearPendingAdd();

        if (pending.path !== location.pathname) { return; }
        if (Date.now() - pending.saved_at > this.PENDING_ADD_MAX_AGE_MS) { return; }

        let variant = variantMap.find(v => v.id === pending.product_var_id);
        if (!variant) { return; }

        // Se deja la ficha como estaba al pulsar, para que lo que se añade sea lo que se ve
        variant.attributes.forEach(attr => {
            let input = this.element.querySelector(`input[name="variation-${attr.name}"][value="${attr.value}"]`);
            if (input) { input.checked = true; }
        });
        this.updatePrice(this.getSelectedAttrs());
        let quantityInput = this.element.querySelector(".quantity-product .m-qty__input");
        if (quantityInput) {
            quantityInput.value = pending.quantity;
            QUANTITY.refresh(quantityInput);
        }

        // El mismo camino que el botón: si ya no hay stock, sale el modal de error del servidor
        this.addToCart();
    },

    init_product_selection: function() {
        this.element.querySelectorAll(".variation-input").forEach(input => {
            input.addEventListener("change", (event) => this.updateAvailableOptions(event));
        });

        // Botón añadir al carrito
        let addBtn = this.element.querySelector(".add-to-cart-btn");
        if (addBtn) {
            addBtn.addEventListener("click", () => this.addToCart());
        }
    }
};

window.addEventListener("load", () => {
    PRODUCT_DETAIL.init();
});
