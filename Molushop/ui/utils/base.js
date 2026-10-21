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

// hx-boost en la tienda (layout1). Con él, los enlaces piden la página con
// htmx y sustituyen el body, así que hay que cuidar lo que una recarga daba
// hecho:
// - Scripts: htmx ejecuta los <script> que trae lo que pinta, y los
//   componentes cargan su JS en el body. Un fichero ya cargado no se vuelve
//   a ejecutar (declararía otra vez sus const y duplicaría sus oyentes): se
//   quita de la respuesta, también de lo que restaura el historial. Y
//   head-support no quita los del <head>: lo que ya se ejecutó sigue vivo.
// - Lo que no es tienda (panel, admin) va recargando: tiene otro layout.
// - Si la página falla (404, 500...) o no hay red, también recargando, para
//   que se vea la página de error del servidor o la del navegador.
// - Transición suave entre páginas (View Transitions), solo al navegar.
// El historial de la tienda no guarda copias (hx-history="false", layout1):
// al volver atrás llega la página del servidor. El del panel sí, y también
// se le quitan los scripts ya cargados.
const BOOST = {
    loaded: new Set(),
    outside: /^\/(panel|admin)(\/|$)/,

    stripLoadedScripts: function(html) {
        document.querySelectorAll('script[src]').forEach((script) => this.loaded.add(script.src));
        return html.replace(/<script\b[^>]*\ssrc="([^"]+)"[^>]*>\s*<\/script>/gi, (tag, src) =>
            this.loaded.has(new URL(src, location.href).href) ? '' : tag);
    },

    reload: function(event) {
        event.preventDefault();
        location.assign(event.detail.requestConfig?.path ?? event.detail.pathInfo.requestPath);
    },
};

document.addEventListener('htmx:beforeRequest', (event) => {
    if (event.detail.boosted && BOOST.outside.test(new URL(event.detail.requestConfig.path, location.href).pathname)) {
        BOOST.reload(event);
    }
});

document.addEventListener('htmx:beforeSwap', (event) => {
    if (event.detail.boosted && event.detail.isError) {
        event.detail.shouldSwap = false;
        location.assign(event.detail.xhr.responseURL || event.detail.pathInfo.requestPath);
        return;
    }
    if (typeof event.detail.serverResponse === 'string') {
        event.detail.serverResponse = BOOST.stripLoadedScripts(event.detail.serverResponse);
    }
    // Con la pestaña oculta el navegador aborta la transición (y lo apunta como error)
    if (event.detail.boosted && !document.hidden) {
        event.detail.swapOverride = 'innerHTML transition:true';
    }
});

document.addEventListener('htmx:sendError', (event) => {
    if (event.detail.boosted) { location.assign(event.detail.pathInfo.requestPath); }
});

document.addEventListener('htmx:historyCacheHit', (event) => {
    event.detail.item.content = BOOST.stripLoadedScripts(event.detail.item.content);
});

document.addEventListener('htmx:historyCacheMissLoad', (event) => {
    event.detail.response = BOOST.stripLoadedScripts(event.detail.response);
});

document.addEventListener('htmx:removingHeadElement', (event) => {
    if (event.detail.headElement.tagName === 'SCRIPT') { event.preventDefault(); }
});

// Errores de htmx: htmx 2 no pinta las respuestas 4xx/5xx. Las que el
// servidor quiere enseñar (un aviso) llevan HX-Retarget (utils/hx.rs,
// error_modal): esas se pintan. Las demás, como antes, no.
document.addEventListener('htmx:beforeSwap', (event) => {
    if (event.detail.isError && !event.detail.boosted && event.detail.xhr.getResponseHeader('HX-Retarget')) {
        event.detail.shouldSwap = true;
        event.detail.isError = false;
    }
});

// Mientras dura una petición de htmx, el botón que la lanza (o los de envío
// de su formulario) se apaga y lleva spinner (aria-busy, button.css): sin
// esto, un doble clic manda dos veces. Para las que salen de htmx.ajax con un
// `source` que no es ni botón ni formulario (el explorador de variaciones),
// los botones marcados con data-busy-disable dentro de ese `source`.
const BUSY = {
    buttons(elt) {
        if (elt.matches('form')) return [...elt.querySelectorAll('[type="submit"]')];
        if (elt.matches('button')) return [elt];
        return [...elt.querySelectorAll('[data-busy-disable]')];
    },
};

document.addEventListener('htmx:beforeRequest', (event) => {
    BUSY.buttons(event.detail.elt).forEach((button) => {
        if (button.disabled) return;   // ya estaba apagado: no es nuestro
        button.disabled = true;
        button.setAttribute('aria-busy', 'true');
    });
});

document.addEventListener('htmx:afterRequest', (event) => {
    BUSY.buttons(event.detail.elt).forEach((button) => {
        if (button.getAttribute('aria-busy') !== 'true') return;
        button.disabled = false;
        button.removeAttribute('aria-busy');
    });
});

// Arranque de los componentes. No vale "load" ni "DOMContentLoaded": solo
// saltan con la carga normal, y con hx-boost la tienda cambia de página sin
// recargar (htmx sustituye el <body>). mount() llama a init(elemento) una vez
// por cada elemento que encaje con `selector`:
// - los de la carga normal y los que htmx meta después (hx-boost, volver
//   atrás con el historial, fragmentos), con htmx.onLoad;
// - los que ya estén en la página cuando llega el script: con hx-boost, el
//   <script src> de la página nueva puede cargarse después del cambio.
// Va por elemento y no por página: lo que restaura el historial son
// elementos nuevos, sin los oyentes de antes, y hay que arrancarlos otra vez.
// Lo que vaya sobre document se pone una sola vez, fuera de init.
const COMPONENT = {
    mount: function(selector, init) {
        let started = new WeakSet();
        let run = (root) => {
            let elements = root.matches?.(selector) ? [root] : root.querySelectorAll(selector);
            elements.forEach((element) => {
                if (started.has(element)) { return; }
                started.add(element);
                init(element);
            });
        };
        htmx.onLoad(run);
        // Mientras se lee el HTML la página está a medias: espera al htmx:load
        // que htmx lanza sobre el body al terminar.
        if (document.readyState !== "loading") { run(document.body); }
    }
};
