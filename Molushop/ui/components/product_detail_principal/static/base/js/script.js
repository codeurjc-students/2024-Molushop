let product_detail = {
    init:function(){
        //obtner los 
        console.log("holaaa");

        let principal = document.querySelector(".product-detail-principal");
        let main_image_container = principal.querySelector(".main-image-container"); 
        let more_images_container = principal.querySelector(".all-images");

        more_images_container.style.height = `${main_image_container.offsetHeight}px`;
        
        //hacer un evento para cuando se haga un rezise que se haha lo mismo
        window.addEventListener("resize",()=>{
            more_images_container.style.height = `${main_image_container.offsetHeight}px`;
        });

        let main_image = main_image_container.querySelector(".image-principal");
        let mini_images =  more_images_container.querySelectorAll('.image-s');
        //hacer un evento para cada una de las mini imagenes
        console.log(mini_images);
        mini_images.forEach((image_cont)=>{
            console.log(image_cont);
            image_cont.addEventListener("mouseenter",()=>{
                let image_url = image_cont.querySelector(".image-mini").src;
                main_image.src=image_url;
            });
        });
    }
}
/*
document.addEventListener("DOMContentLoaded",()=>{
    product_detail.init();
});*/
window.addEventListener("load",()=>{
    product_detail.init();
});

//cuando se haga un resize y cuando se cargue el hmtl, adaptar el tamaño 