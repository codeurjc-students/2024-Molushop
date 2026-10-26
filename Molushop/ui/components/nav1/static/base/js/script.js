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

// Sugerencias del buscador. htmx las pide al escribir (2 letras o más) y las
// pinta en #nav1-suggestions; aquí solo se abre y cierra la caja y se mueve la
// opción activa con las flechas. El foco se queda en el input
// (aria-activedescendant): Enter sobre una opción la abre y, sin opción, busca.
let nav1_search = {
    parts: function(){
        return {
            input: document.querySelector(".nav1 .search-bar input[type=search]"),
            box: document.getElementById("nav1-suggestions")
        };
    },

    options: function(box){
        return [...box.querySelectorAll("[role=option]")];
    },

    set_open: function(open){
        const {input, box} = nav1_search.parts();
        if(!input || !box){
            return
        }
        open = open && box.childElementCount > 0;
        box.hidden = !open;
        input.setAttribute("aria-expanded", open);
        if(!open){
            nav1_search.set_active(-1);
        }
    },

    set_active: function(index){
        const {input, box} = nav1_search.parts();
        const options = nav1_search.options(box);
        options.forEach((option, i) => option.setAttribute("aria-selected", i === index));
        if(index >= 0){
            input.setAttribute("aria-activedescendant", options[index].id);
            options[index].scrollIntoView({block: "nearest"});
        }else{
            input.removeAttribute("aria-activedescendant");
        }
    },

    // Una vez por cabecera (COMPONENT.mount, base.js)
    init: function(nav){
        const input = nav.querySelector(".search-bar input[type=search]");
        const box = nav.querySelector("#nav1-suggestions");
        if(!input || !box){
            return
        }

        // Llegan sugerencias nuevas: se enseñan si el usuario sigue en el input.
        box.addEventListener("htmx:afterSwap",()=>{
            nav1_search.set_open(document.activeElement === input);
        });

        input.addEventListener("input",()=>{
            // Por debajo de 2 letras htmx no pide nada: fuera las de antes.
            if(input.value.trim().length < 2){
                box.replaceChildren();
                nav1_search.set_open(false);
            }
        });

        input.addEventListener("focus",()=>{
            nav1_search.set_open(input.value.trim().length >= 2);
        });

        input.addEventListener("keydown",(event)=>{
            const options = nav1_search.options(box);
            const current = options.findIndex(option => option.getAttribute("aria-selected") === "true");
            if(event.key === "ArrowDown" || event.key === "ArrowUp"){
                if(options.length === 0){
                    return
                }
                event.preventDefault();
                if(box.hidden){
                    nav1_search.set_open(true);
                }
                // Pasado el final (o antes del principio) vuelve al input.
                let next = current + (event.key === "ArrowDown" ? 1 : -1);
                if(current === -1 && event.key === "ArrowUp"){
                    next = options.length - 1;
                }else if(next >= options.length || next < 0){
                    next = -1;
                }
                nav1_search.set_active(next);
            }else if(event.key === "Enter" && !box.hidden && current >= 0){
                event.preventDefault();
                nav1_search.set_open(false);
                options[current].click();
            }else if(event.key === "Escape" && !box.hidden){
                event.preventDefault();
                nav1_search.set_open(false);
            }
        });

        // Al buscar (Enter o la lupa) o al elegir una opción, se cierra.
        input.form.addEventListener("submit",()=>nav1_search.set_open(false));
        box.addEventListener("click",(event)=>{
            if(event.target.closest("a")){
                nav1_search.set_open(false);
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
    const {box} = nav1_search.parts();
    if(box && !box.hidden && !event.target.closest(".nav1 .search-bar")){
        nav1_search.set_open(false);
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
COMPONENT.mount(".nav1", nav1_search.init);
