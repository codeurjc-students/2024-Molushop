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
                login_modal.element.showModal();
            })
        });
        
        login_modal.element.addEventListener("click",(event)=>{
            if(event.target===login_modal.element){
                login_modal.element.close();
            }
        });
    }
}

document.addEventListener("DOMContentLoaded",login_modal.init());