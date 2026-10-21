let nav1_menu = {
    // El menú y su botón se buscan cada vez: con hx-boost la cabecera llega
    // nueva con cada página.
    parts: function(){
        return {
            trigger: document.querySelector(".nav1 .user-icon"),
            menu: document.getElementById("profile-menu")
        };
    },

    set_open: function(open){
        const {trigger, menu} = nav1_menu.parts();
        if(!trigger || !menu){
            return
        }
        menu.hidden = !open;
        trigger.setAttribute("aria-expanded", open);
    },

    // Una vez por cabecera (COMPONENT.mount, base.js)
    init: function(nav){
        const trigger = nav.querySelector(".user-icon");
        const menu = nav.querySelector("#profile-menu");
        if(!trigger || !menu){
            return
        }

        trigger.addEventListener("click",()=>{
            nav1_menu.set_open(menu.hidden);
        });

        // Cualquier enlace o botón de dentro cierra el menú: los enlaces navegan y
        // "Identifícate"/"Registrarse" abren el modal, que ya tapa todo lo demás.
        menu.addEventListener("click",(event)=>{
            if(event.target.closest("a, button")){
                nav1_menu.set_open(false);
            }
        });
    }
}

// Sobre document, una sola vez: sirven para la cabecera que haya en cada momento.
document.addEventListener("click",(event)=>{
    const {menu} = nav1_menu.parts();
    if(menu && !menu.hidden && !event.target.closest(".nav1 .profile")){
        nav1_menu.set_open(false);
    }
});

document.addEventListener("keydown",(event)=>{
    const {trigger, menu} = nav1_menu.parts();
    if(event.key==="Escape" && menu && !menu.hidden){
        nav1_menu.set_open(false);
        trigger.focus();
    }
});

COMPONENT.mount(".nav1", nav1_menu.init);
