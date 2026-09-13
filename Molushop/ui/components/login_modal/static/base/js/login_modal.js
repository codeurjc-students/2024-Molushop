let login_modal = {
    element: null,
    // true si el modal se abrió porque la sesión con la que se pintó la página ya no vale
    session_ended: false,
    init: function(){
        // Se prepara SIEMPRE, también si la página se pintó con sesión: la sesión puede caducar
        // (o cerrarse en otra pestaña) con la página abierta, y entonces un 401 tiene que poder
        // abrir el modal. Los botones "Identifícate"/"Registrarse" ya solo se pintan sin sesión.
        login_modal.element = document.querySelector("dialog.login_modal");
        if(!login_modal.element){
            return
        }
        let botones_activar = document.querySelectorAll(".open-login-modal");
        botones_activar.forEach((activate)=>{
            activate.addEventListener("click",()=>{
                login_modal.open("login");
            })
        });
        let botones_registro = document.querySelectorAll(".open-register-modal");
        botones_registro.forEach((activate)=>{
            activate.addEventListener("click",()=>{
                login_modal.open("register");
            })
        });

        // Cualquier componente que reciba un 401 lanza "auth-required" (mismo nombre que el
        // HX-Trigger que ya manda el backend) y aquí se decide qué hacer: abrir el login.
        // Si la página se pintó con sesión, es que se ha cerrado o ha caducado: se avisa.
        document.addEventListener("auth-required",()=>{
            let session_ended = document.body.dataset.user_logged === "true";
            login_modal.open("login", session_ended);
        });

        login_modal.element.addEventListener("click",(event)=>{
            if(event.target===login_modal.element){
                login_modal.element.close();
            }
        });

        // Cerrar sin identificarse cuando la sesión había terminado: se recarga para que el nav
        // y la página dejen de enseñar datos de una sesión que ya no existe. Si el usuario se
        // identifica no se llega aquí: login_base.do_after_login recarga antes.
        // "close" salta con cualquier forma de cerrar: click fuera, Escape o close().
        login_modal.element.addEventListener("close",()=>{
            if(login_modal.session_ended){
                location.reload();
            }
        });
    },
    // mode: "login" | "register". Se fija en cada apertura porque los botones .change del
    // login_base alternan con toggle, y el modal se abriría en el último modo que se vio.
    open: function(mode, session_ended = false){
        if(login_modal.element.open){
            return
        }
        login_modal.session_ended = session_ended;
        login_modal.element.querySelector(".login_modal-notice").hidden = !session_ended;
        login_modal.element.querySelector(".container-ext.login")?.classList.toggle("disabled", mode!=="login");
        login_modal.element.querySelector(".container-ext.register")?.classList.toggle("disabled", mode!=="register");
        login_modal.element.showModal();
    }
}

document.addEventListener("DOMContentLoaded",login_modal.init);
