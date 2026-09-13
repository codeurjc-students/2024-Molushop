let nav1_menu = {
    init: function(){
        const trigger = document.querySelector(".nav1 .user-icon");
        const menu = document.getElementById("profile-menu");
        if(!trigger || !menu){
            return
        }

        const set_open = (open)=>{
            menu.hidden = !open;
            trigger.setAttribute("aria-expanded", open);
        };

        trigger.addEventListener("click",()=>{
            set_open(menu.hidden);
        });

        // Cualquier enlace o botón de dentro cierra el menú: los enlaces navegan y
        // "Identifícate"/"Registrarse" abren el modal, que ya tapa todo lo demás.
        menu.addEventListener("click",(event)=>{
            if(event.target.closest("a, button")){
                set_open(false);
            }
        });

        document.addEventListener("click",(event)=>{
            if(!menu.hidden && !event.target.closest(".nav1 .profile")){
                set_open(false);
            }
        });

        document.addEventListener("keydown",(event)=>{
            if(event.key==="Escape" && !menu.hidden){
                set_open(false);
                trigger.focus();
            }
        });
    }
}

// Este script se carga en el <head>: hay que esperar a que exista el nav.
// Se pasa la función, no se llama.
document.addEventListener("DOMContentLoaded",nav1_menu.init);
