const PRODUCT_DETAIL = {
    // La ficha que se ve y sus variaciones (JSON de la plantilla): con hx-boost
    // se pasa de una ficha a otra sin recargar.
    element: null,
    variantMap: [],

    // Una vez por ficha (COMPONENT.mount, base.js)
    init: function(principal) {
        this.element = principal;
        this.variantMap = JSON.parse(principal.querySelector(".variant-map").textContent);

        let main_image_container = principal.querySelector(".main-image-container");
        let more_images_container = principal.querySelector(".all-images");

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
        this.resumePendingAdd();
    },

    // Un texto de la ficha en el idioma de la página (data-text-*, true.html);
    // si faltara, el español.
    text: function(name, fallback) {
        return this.element.dataset["text" + name] || fallback;
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
    // de la más parecida a lo elegido (selectedAttrs) a la menos: al saltar de
    // combinación se cambia lo mínimo. Por orden:
    // 1. cuántos valores comparten con lo elegido (sin contar el que se acaba de pulsar);
    // 2. las que tienen stock;
    // 3. las que conservan los primeros atributos de la ficha (Color antes que Altura...).
    getVariantsByAttr: function(attrName, attrValue, selectedAttrs = {}) {
        let order = [...new Set([...this.element.querySelectorAll(".variation-input")]
            .map(input => input.name.replace("variation-", "")))]
            .filter(name => name !== attrName);
        let keeps = (variant, name) => variant.attributes.some(a => a.name === name && a.value === selectedAttrs[name]);
        let shared = (variant) => order.filter(name => keeps(variant, name)).length;

        return this.variantMap
            .filter(variant => variant.attributes.some(a => a.name === attrName && a.value === attrValue))
            .sort((a, b) => {
                let byShared = shared(b) - shared(a);
                if (byShared !== 0) { return byShared; }
                let byStock = (b.stock > 0) - (a.stock > 0);
                if (byStock !== 0) { return byStock; }
                for (let name of order) {
                    let byName = keeps(b, name) - keeps(a, name);
                    if (byName !== 0) { return byName; }
                }
                return 0;
            });
    },

    // Al elegir un valor: si la combinación existe, se queda aunque esté agotada
    // (se ve su precio y el botón dice "Agotado"); si no existe, se salta a la
    // primera con stock que lleve ese valor (o, si no hay ninguna, a la primera
    // que exista), la más parecida a lo elegido (getVariantsByAttr). Después se
    // repintan precio, píldoras y botón.
    updateAvailableOptions: function(event) {
        let target = event.currentTarget;
        let selectedAttrs = this.getSelectedAttrs();
        let variation = target.name.replace("variation-", "");
        let variationValue = target.value;

        let availableValues = this.getVariantsByAttr(variation, variationValue, selectedAttrs);

        let current = this.findVariantByAttrs(selectedAttrs);

        if (current) {
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
        this.refreshGallery(this.findVariantByAttrs(this.getSelectedAttrs()));
    },

    // La galería de la variación elegida, con la misma regla que
    // ProductDetailPrincipalData::gallery_for: primero sus imágenes, luego las
    // del producto entero (data-variations vacío) y, escondidas, las de otras
    // variaciones; sin propias ni generales, todas. La grande pasa a la primera.
    // Si la combinación no existe, se queda como estaba.
    refreshGallery: function(variant) {
        if (variant == null) { return; }
        let gallery = this.element.querySelector(".all-images");
        let thumbs = [...gallery.querySelectorAll(".image-s")];
        if (thumbs.length === 0) { return; }

        let linkedTo = (thumb) => thumb.dataset.variations ? thumb.dataset.variations.split(",") : [];
        let own = thumbs.filter(thumb => linkedTo(thumb).includes(variant.id));
        let general = thumbs.filter(thumb => linkedTo(thumb).length === 0);
        let showAll = own.length === 0 && general.length === 0;
        let others = thumbs.filter(thumb => !own.includes(thumb) && !general.includes(thumb));

        let order = showAll ? thumbs : [...own, ...general, ...others];
        order.forEach(thumb => {
            thumb.hidden = !showAll && others.includes(thumb);
            gallery.appendChild(thumb);
        });
        gallery.scrollTop = 0;
        gallery.scrollLeft = 0;
        let first = order.find(thumb => !thumb.hidden);
        this.element.querySelector(".image-principal").src = first.querySelector(".image-mini").src;
    },

    // Tacha (sin apagar: se puede elegir y ver agotada) los valores que, con lo
    // demás elegido, dan una variación que existe y está agotada. Si la
    // combinación no existe no se tacha: al pulsarlo se salta a una que lleve
    // ese valor (updateAvailableOptions). Los que no lleva ninguna variación
    // activa ya vienen `disabled` del servidor.
    refreshAvailability: function() {
        this.showError(null);
        let selectedAttrs = this.getSelectedAttrs();
        this.element.querySelectorAll(".variation-input").forEach(input => {
            let attrName = input.name.replace("variation-", "");
            let variant = this.findVariantByAttrs({ ...selectedAttrs, [attrName]: input.value });
            let soldOut = variant != null && variant.stock <= 0;
            let label = this.element.querySelector(`label[for="${CSS.escape(input.id)}"]`);
            if (label) { label.classList.toggle("is-out-of-stock", soldOut); }
        });
        this.refreshAddButton();
    },

    // "Agotado" y apagado si la combinación elegida no existe o no tiene stock;
    // si existe y está agotada, además el aviso bajo el botón.
    refreshAddButton: function() {
        let btn = this.element.querySelector(".add-to-cart-btn");
        if (btn == null) { return; }
        let variant = this.findVariantByAttrs(this.getSelectedAttrs());
        let soldOut = variant == null || variant.stock <= 0;
        btn.disabled = soldOut;
        let text = btn.querySelector("span");
        if (text) { text.textContent = soldOut ? this.text("SoldOut", "Agotado") : this.text("Add", "Añadir al carrito"); }
        let msg = this.element.querySelector(".sold-out-msg");
        if (msg) { msg.hidden = !(variant != null && variant.stock <= 0); }
    },
    // Sin atributos seleccionados solo encaja la variación única de un producto
    // sin variaciones (attributes = []); en uno con variaciones no encaja ninguna.
    findVariantByAttrs: function(attrsJson) {
      let attrs = (typeof attrsJson === "string") ? JSON.parse(attrsJson) : attrsJson;
      let attrNames = Object.keys(attrs);

      return this.variantMap.find(variant =>
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

        let match = this.variantMap.find(variant =>
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
            priceEl.textContent = hasPrice ? MONEY.format(match.price) : this.text("Unavailable", "No disponible");
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
            this.showError(this.text("PickVariation", "Selecciona una variación válida."));
            return;
        }
        // El servidor la rechazaría igual ("No price found"); así el aviso se
        // entiende y no se hace la petición.
        if (variant.price == null) {
            this.showError(this.text("VariationUnavailable", "Esta variación no está disponible ahora mismo."));
            return;
        }

        let quantityInput = this.element.querySelector(".quantity-product .m-qty__input");
        let quantity = parseInt(quantityInput.value) || 1;

        if (quantity <= 0) {
            this.showError(this.text("QuantityPositive", "La cantidad debe ser mayor que 0."));
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
            return response.text().then(html => ({ html, cartCount, ok: response.ok }));
        })
        .then(data => {
            if (data === null) return;
            if (data.ok) {
                // Añadido: un toast (átomo m-toast) en #m-toasts. htmx.swap procesa
                // su hyperscript (se quita solo).
                htmx.swap(document.getElementById('m-toasts'), data.html, { swapStyle: 'beforeend' });
            } else {
                // Error (sin stock, variación que ya no existe...): el aviso, un modal
                document.body.insertAdjacentHTML('beforeend', data.html);
                let modal = document.getElementById('myModal');
                if (modal && typeof _hyperscript !== 'undefined') {
                    _hyperscript.processNode(modal);
                }
            }
            if (data.cartCount) {
                let badge = document.getElementById('cart-badge');
                if (badge) {
                    badge.textContent = data.cartCount;
                    badge.style.display = 'flex';
                    // "Bote" del número (nav1/styles.css): se quita la clase y se
                    // fuerza un reflow para que la animación vuelva a empezar
                    badge.classList.remove('is-bumped');
                    void badge.offsetWidth;
                    badge.classList.add('is-bumped');
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

        let variant = this.variantMap.find(v => v.id === pending.product_var_id);
        if (!variant) { return; }

        // Se deja la ficha como estaba al pulsar, para que lo que se añade sea lo que se ve
        variant.attributes.forEach(attr => {
            let input = this.element.querySelector(`input[name="variation-${attr.name}"][value="${attr.value}"]`);
            if (input) { input.checked = true; }
        });
        this.updatePrice(this.getSelectedAttrs());
        this.refreshGallery(variant);
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

// Sobre document, una sola vez: vale para la ficha que se vea.
document.addEventListener("auth-cancelled", () => PRODUCT_DETAIL.clearPendingAdd());

COMPONENT.mount(".product-detail-principal", (principal) => PRODUCT_DETAIL.init(principal));
