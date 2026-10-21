let login_modal = {
    // El <dialog> de la página actual: con hx-boost llega uno nuevo con cada página
    element: null,
    // true si el modal se abrió porque la sesión con la que se pintó la página ya no vale
    session_ended: false,
    // Una vez por <dialog> (COMPONENT.mount, base.js).
    // Se prepara SIEMPRE, también si la página se pintó con sesión: la sesión puede caducar
    // (o cerrarse en otra pestaña) con la página abierta, y entonces un 401 tiene que poder
    // abrir el modal. Los botones "Identifícate"/"Registrarse" ya solo se pintan sin sesión.
    init: function(dialog){
        login_modal.element = dialog;

        // Páginas que no tienen sentido sin sesión (p. ej. /orders) marcan un elemento con
        // data-login-required, y el login se abre solo al entrar.
        if(document.body.dataset.user_logged !== "true" && document.querySelector("[data-login-required]")){
            login_modal.open("login");
        }

        dialog.addEventListener("click",(event)=>{
            if(event.target===dialog){
                dialog.close();
            }
        });

        // Cerrar sin identificarse cuando la sesión había terminado: se recarga para que el nav
        // y la página dejen de enseñar datos de una sesión que ya no existe. Si el usuario se
        // identifica no se llega aquí: login_base.do_after_login recarga antes.
        // "close" salta con cualquier forma de cerrar: click fuera, Escape o close().
        dialog.addEventListener("close",()=>{
            // Quien tuviera algo esperando a este login (p. ej. un "Añadir al carrito") lo descarta
            document.dispatchEvent(new CustomEvent("auth-cancelled"));
            if(login_modal.session_ended){
                location.reload();
            }
        });
    },
    // mode: "login" | "register". Se fija en cada apertura porque los botones .change del
    // login_base alternan con toggle, y el modal se abriría en el último modo que se vio.
    open: function(mode, session_ended = false){
        if(!login_modal.element || login_modal.element.open){
            return
        }
        login_modal.session_ended = session_ended;
        login_modal.element.querySelector(".login_modal-notice").hidden = !session_ended;
        login_modal.element.querySelector(".container-ext.login")?.classList.toggle("disabled", mode!=="login");
        login_modal.element.querySelector(".container-ext.register")?.classList.toggle("disabled", mode!=="register");
        login_modal.element.showModal();
    }
}

// Sobre document, una sola vez. Los botones van delegados: así valen también los que
// lleguen después de la carga (fragmentos de htmx, otra página con hx-boost).
document.addEventListener("click",(event)=>{
    if(event.target.closest(".open-login-modal")){
        login_modal.open("login");
    }else if(event.target.closest(".open-register-modal")){
        login_modal.open("register");
    }
});

// Cualquier componente que reciba un 401 lanza "auth-required" (mismo nombre que el
// HX-Trigger que ya manda el backend) y aquí se decide qué hacer: abrir el login.
// Si la página se pintó con sesión, es que se ha cerrado o ha caducado: se avisa.
document.addEventListener("auth-required",()=>{
    let session_ended = document.body.dataset.user_logged === "true";
    login_modal.open("login", session_ended);
});

COMPONENT.mount("dialog.login_modal", login_modal.init);
