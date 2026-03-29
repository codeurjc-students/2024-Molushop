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

    init_product_selection: function() {
        this.element.querySelectorAll(".variation-input").forEach(input => {
            input.addEventListener("change", (event) => this.updateAvailableOptions(event));
        });
        // Estado inicial al cargar la página
        
        //this.updateAvailableOptions();
    }
};

window.addEventListener("load", () => {
    PRODUCT_DETAIL.init();
});
