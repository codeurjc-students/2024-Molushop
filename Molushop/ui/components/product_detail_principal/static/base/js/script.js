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

        this.init_product_selection();
        this.init_quantity();

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

    // Devuelve las variaciones de variantMap que contienen el atributo name=attrName con value=attrValue
    getVariantsByAttr: function(attrName, attrValue) {
        return variantMap.filter(variant =>
            variant.attributes.some(a => a.name === attrName && a.value === attrValue)
        );
    },

    // Devuelve un Set con los valores válidos de targetAttrName,
    // filtrando variantMap por todos los atributos seleccionados EXCEPTO targetAttrName
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

    // Selecciona automáticamente los atributos del primer variant que coincida con la variación elegida.
    updateAvailableOptions: function(event) {
        console.log(event);
        let target = event.currentTarget;
        console.log(target);
        let selectedAttrs = this.getSelectedAttrs();
        let variation = target.name.replace("variation-", "");
        let variationValue = target.value;

        let availableValues = this.getVariantsByAttr(variation, variationValue);

        let newAttribute = this.findVariantByAttrs(selectedAttrs);
        console.log(newAttribute);

        if(newAttribute){
            this.updatePrice(selectedAttrs);
        }else if (availableValues && availableValues.length > 0) {
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
        
    },
    findVariantByAttrs: function(attrsJson) {                                                                                                                                                        let attrs = (typeof attrsJson === "string") ? JSON.parse(attrsJson) : attrsJson;
      let attrNames = Object.keys(attrs);                                                                                                                                                          if (attrNames.length === 0) return null;                                                                                                                                               

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

        let priceEl = document.getElementById("product-price");
        if (priceEl && match) {
            priceEl.textContent = match.price;
        }
    },

    addToCart: function() {
        let selectedAttrs = this.getSelectedAttrs();
        let variant = this.findVariantByAttrs(selectedAttrs);

        if (!variant) {
            alert("Selecciona una variación válida");
            return;
        }

        let quantityInput = this.element.querySelector(".quantity-product input[type='number']");
        let quantity = parseInt(quantityInput.value) || 1;

        if (quantity <= 0) {
            alert("La cantidad debe ser mayor a 0");
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
        let quantityInput = this.element.querySelector(".quantity-product input[type='number']");
        if (quantityInput) { quantityInput.value = pending.quantity; }

        // El mismo camino que el botón: si ya no hay stock, sale el modal de error del servidor
        this.addToCart();
    },

    // +/- del selector de cantidad. Sólo tocan el input: la petición al
    // carrito la sigue lanzando el botón "Añadir al carrito".
    stepQuantity: function(delta) {
        let input = this.element.querySelector(".quantity-product input[type='number']");
        if (input == null) { return; }

        // Los límites se leen del propio input para que manden los atributos
        // min/max de la plantilla y no queden duplicados aquí.
        let min = parseInt(input.min);
        let max = parseInt(input.max);
        let quantity = (parseInt(input.value) || 0) + delta;

        if (!isNaN(min) && quantity < min) { quantity = min; }
        if (!isNaN(max) && quantity > max) { quantity = max; }

        input.value = quantity;
    },

    init_quantity: function() {
        let minus = this.element.querySelector(".quantity-product .minus-cont");
        let plus = this.element.querySelector(".quantity-product .plus-cont");

        if (minus) { minus.addEventListener("click", () => this.stepQuantity(-1)); }
        if (plus) { plus.addEventListener("click", () => this.stepQuantity(1)); }
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
