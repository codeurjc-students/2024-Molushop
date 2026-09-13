let login_modal = {
    element: null,
    init: function(){
        if(document.body.dataset.user_logged==="true"){ // si ya está loggeado no salta el modal de login
            return
        }

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

        login_modal.element.addEventListener("click",(event)=>{
            if(event.target===login_modal.element){
                login_modal.element.close();
            }
        });
    },
    // mode: "login" | "register". Se fija en cada apertura porque los botones .change del
    // login_base alternan con toggle, y el modal se abriría en el último modo que se vio.
    open: function(mode){
        login_modal.element.querySelector(".container-ext.login")?.classList.toggle("disabled", mode!=="login");
        login_modal.element.querySelector(".container-ext.register")?.classList.toggle("disabled", mode!=="register");
        login_modal.element.showModal();
    }
}

document.addEventListener("DOMContentLoaded",login_modal.init);
