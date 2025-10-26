let login_base = {
    init: function(){

    },
    do_after_login:function(event){   
        let detail = event.detail; //datos del evento htmx
        if(detail.xhr.status>=200&&detail.xhr.status<300){
            console.log("todo okey");
            //recargar la pagina
            location.reload(); 
        }else{
            console.log("Algo ha ido mal!");
        }
    }
    //podemos poner aqui el evento para que se actualice la pagina o directamente con htmx con hx-on
}