// Utilidades comunes; layout1 carga este fichero en todas las páginas.

// Importes en el formato de la tienda ("9,00", "12.345,67"): el mismo texto que
// pinta el servidor (constants::money::format_amount y el átomo Price). Úsalo
// siempre que el JS escriba un precio, en vez de toFixed().
const MONEY = {
    formatter: new Intl.NumberFormat("es-ES", {
        minimumFractionDigits: 2,
        maximumFractionDigits: 2
    }),

    // Acepta número o texto ("150.99", que es como llega de la BD).
    format: function(value) {
        let number = Number(value);
        if (!Number.isFinite(number)) { return String(value); }
        // Sin "-0,00" para un negativo que redondea a cero, igual que en Rust.
        if (Math.abs(number) < 0.005) { number = 0; }
        return this.formatter.format(number);
    }
};

// Átomo de cantidad (Quantity, ui/styles/atoms/quantity.css). Va delegado sobre
// document: vale para los selectores que pinte el servidor y para los que
// lleguen después por htmx o fetch, sin engancharlos uno a uno.
//
// - Los botones suman o restan sin salirse del min/max del input, apagan el
//   botón del límite y lanzan `m-qty-change` (burbujea, con detail.value) desde
//   el input. Es la señal para el componente: la ficha no hace nada hasta
//   "Añadir al carrito"; el carrito la manda al servidor.
// - Lo escrito a mano no lanza `m-qty-change` (el componente escucha `input` si
//   lo quiere): solo se ajusta a los límites al salir del campo.
const QUANTITY = {
    limits: function(input) {
        return { min: parseInt(input.min), max: parseInt(input.max) };
    },

    clamp: function(input, value) {
        let { min, max } = this.limits(input);
        if (!isNaN(min) && value < min) { value = min; }
        if (!isNaN(max) && value > max) { value = max; }
        return value;
    },

    // Apaga − en el mínimo y + en el máximo
    refresh: function(input) {
        let box = input.closest(".m-qty");
        if (box == null) { return; }
        let { min, max } = this.limits(input);
        let value = parseInt(input.value);
        box.querySelector('.m-qty__btn[data-step="-1"]').disabled = !isNaN(min) && value <= min;
        box.querySelector('.m-qty__btn[data-step="1"]').disabled = !isNaN(max) && value >= max;
    },

    step: function(button) {
        let input = button.closest(".m-qty").querySelector(".m-qty__input");
        let current = parseInt(input.value) || 0;
        let value = this.clamp(input, current + parseInt(button.dataset.step));
        if (value === current) { return; }

        input.value = value;
        this.refresh(input);
        input.dispatchEvent(new CustomEvent("m-qty-change", { bubbles: true, detail: { value } }));
    },

    init: function() {
        document.addEventListener("click", (event) => {
            let button = event.target.closest(".m-qty__btn");
            if (button != null && !button.disabled) { this.step(button); }
        });
        document.addEventListener("input", (event) => {
            if (event.target.classList.contains("m-qty__input")) { this.refresh(event.target); }
        });
        document.addEventListener("change", (event) => {
            let input = event.target;
            if (!input.classList.contains("m-qty__input")) { return; }
            let { min } = this.limits(input);
            let value = parseInt(input.value);
            input.value = this.clamp(input, isNaN(value) ? (isNaN(min) ? 0 : min) : value);
            this.refresh(input);
        });
    }
};
QUANTITY.init();

// Errores de htmx: htmx 2 no pinta las respuestas 4xx/5xx. Las que el
// servidor quiere enseñar (un aviso) llevan HX-Retarget (utils/hx.rs,
// error_modal): esas se pintan. Las demás, como antes, no.
document.addEventListener('htmx:beforeSwap', (event) => {
    if (event.detail.isError && event.detail.xhr.getResponseHeader('HX-Retarget')) {
        event.detail.shouldSwap = true;
        event.detail.isError = false;
    }
});
