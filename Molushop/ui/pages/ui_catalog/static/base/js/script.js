// Catálogo de UI: pinta cada muestra leyendo la variable de tokens.css que
// indica su atributo data-*, para no repetir aquí ningún valor.
const UI_CATALOG = {
    init: function() {
        let root = document.querySelector(".ui-catalog");
        if (root == null) { return; }
        this.styles = getComputedStyle(document.documentElement);

        root.querySelectorAll("[data-token]").forEach(el => this.initSwatch(el));
        root.querySelectorAll("[data-font-size]").forEach(el => {
            el.style.fontSize = `var(${el.dataset.fontSize})`;
            this.prependLabel(el, el.dataset.fontSize);
        });
        root.querySelectorAll("[data-font-weight]").forEach(el => {
            el.style.fontWeight = `var(${el.dataset.fontWeight})`;
            this.prependLabel(el, el.dataset.fontWeight);
        });
        root.querySelectorAll("[data-space]").forEach(el => {
            let bar = document.createElement("div");
            bar.className = "ui-space-bar";
            bar.style.width = `var(${el.dataset.space})`;
            el.appendChild(bar);
            this.prependLabel(el, el.dataset.space);
        });
        root.querySelectorAll("[data-radius]").forEach(el => {
            el.style.borderRadius = `var(${el.dataset.radius})`;
            this.prependLabel(el, el.dataset.radius);
        });
        root.querySelectorAll("[data-shadow]").forEach(el => {
            el.style.boxShadow = `var(${el.dataset.shadow})`;
            el.style.borderRadius = "var(--radius-md)";
            this.prependLabel(el, el.dataset.shadow, false);
        });
    },

    // Valor de la variable tal como está escrito en tokens.css
    valueOf: function(token) {
        return this.styles.getPropertyValue(token).trim();
    },

    initSwatch: function(el) {
        let token = el.dataset.token;

        let color = document.createElement("div");
        color.className = "ui-swatch-color";
        color.style.backgroundColor = `var(${token})`;

        let name = document.createElement("span");
        name.className = "ui-swatch-name";
        name.textContent = token;

        // El color ya resuelto (rgb), que es lo que de verdad se pinta
        let value = document.createElement("span");
        value.className = "ui-swatch-value";

        el.append(color, name, value);
        value.textContent = this.toHex(getComputedStyle(color).backgroundColor);
    },

    prependLabel: function(el, token, withValue = true) {
        let label = document.createElement("span");
        label.className = "ui-label";
        label.textContent = withValue ? `${token}: ${this.valueOf(token)}` : token;
        el.prepend(label);
    },

    // "rgb(76, 230, 147)" -> "#4ce693". Si no encaja, se deja como venga.
    toHex: function(rgb) {
        let parts = rgb.match(/^rgb\((\d+),\s*(\d+),\s*(\d+)\)$/);
        if (parts == null) { return rgb; }
        return "#" + parts.slice(1).map(n => Number(n).toString(16).padStart(2, "0")).join("");
    }
};

window.addEventListener("load", () => {
    UI_CATALOG.init();
});
