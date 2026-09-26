// Opiniones de producto. La sección se pinta entera en el servidor y se
// sustituye entera en cada cambio (publicar, "ver más"), así que aquí no hay
// nada de interfaz: las estrellas son radios y el formulario lo envía htmx.
//
// Lo único que necesita JavaScript es no perder lo escrito cuando la sesión se
// ha caído mientras el usuario escribía: el POST devuelve 401, se abre el login
// (login_modal.js), y al identificarse la página se RECARGA. Sin guardar el
// borrador, el texto se iría con la recarga. Mismo apaño que el "añadir al
// carrito" pendiente de la ficha, con una diferencia: aquí no se reenvía solo.
// Publicar una opinión es algo que dice el usuario, no algo que se reintenta a
// su espalda; el borrador vuelve al formulario y él decide.
const PRODUCT_REVIEWS = {
    DRAFT_KEY: "pending_review",
    DRAFT_MAX_AGE_MS: 10 * 60 * 1000,

    init: function() {
        // Los oyentes van en document y no en el formulario: la sección se
        // sustituye entera y se los llevaría por delante.
        document.addEventListener("htmx:responseError", (event) => this.onResponseError(event));
        document.addEventListener("click", (event) => this.onClick(event));
        document.addEventListener("auth-cancelled", () => this.clearDraft());
        this.resumeDraft();
    },

    // Delegado en document a propósito: el botón de identificarse puede venir
    // en un fragmento pintado después de la carga (el "ver más" de un visitante
    // sin sesión), y un oyente puesto en el botón no habría existido entonces.
    onClick: function(event) {
        let button = event.target.closest ? event.target.closest(".review-login-btn") : null;
        if (button == null) { return; }
        // Lo recoge login_modal.js y abre el login, igual que hace con el 401.
        document.dispatchEvent(new CustomEvent("auth-required"));
    },

    onResponseError: function(event) {
        if (event.detail == null || event.detail.xhr == null) { return; }
        if (event.detail.xhr.status !== 401) { return; }

        let form = event.target.closest ? event.target.closest(".review-form") : null;
        if (form == null) { return; }

        this.saveDraft(form);
    },

    saveDraft: function(form) {
        let checked = form.querySelector('input[name="rating_value"]:checked');
        let textarea = form.querySelector('textarea[name="comment"]');
        try {
            sessionStorage.setItem(this.DRAFT_KEY, JSON.stringify({
                path: location.pathname,
                rating_value: checked ? checked.value : null,
                comment: textarea ? textarea.value : "",
                saved_at: Date.now()
            }));
        } catch (e) {
            // Sin sessionStorage (modo privado estricto) no hay borrador: se
            // pierde lo escrito, como pasaría sin todo esto.
        }
    },

    clearDraft: function() {
        try { sessionStorage.removeItem(this.DRAFT_KEY); } catch (e) {}
    },

    resumeDraft: function() {
        if (document.body.dataset.user_logged !== "true") { return; }

        let draft = null;
        try { draft = JSON.parse(sessionStorage.getItem(this.DRAFT_KEY)); } catch (e) {}
        if (draft == null) { return; }

        // Se borra siempre: un borrador viejo, de otra ficha o que no encuentre
        // formulario no debe quedarse esperando a la siguiente página.
        this.clearDraft();

        if (draft.path !== location.pathname) { return; }
        if (Date.now() - draft.saved_at > this.DRAFT_MAX_AGE_MS) { return; }

        let form = document.querySelector(".product-reviews-1 .review-form");
        if (form == null) { return; }

        if (draft.rating_value) {
            let radio = form.querySelector('input[name="rating_value"][value="' + draft.rating_value + '"]');
            if (radio) { radio.checked = true; }
        }
        let textarea = form.querySelector('textarea[name="comment"]');
        if (textarea) { textarea.value = draft.comment || ""; }
    }
};

window.addEventListener("load", () => {
    PRODUCT_REVIEWS.init();
});
