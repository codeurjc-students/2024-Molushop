
function incrementarCarrito(){
    console.log('Incrementando contador');
        
    // Obtiene el div con el ID 'numCarrito'
    var contadorDiv = document.getElementById('numCarrito');
    
    // Convierte el contenido del div a un número entero
    var contador = parseInt(contadorDiv.textContent, 10) || 0;
    
    // Incrementa el contador
    contador += 1;
    
    // Actualiza el contenido del div con el nuevo valor del contador
    contadorDiv.textContent = contador;

    console.log('Contador incrementado');
}

function decrementarCarrito(){
    console.log('Decrementando contador');
    // Obtiene el div con el ID 'numCarrito'
    var contadorDiv = document.getElementById('numCarrito');
    
    // Convierte el contenido del div a un número entero
    var contador = parseInt(contadorDiv.textContent, 10) || 0;
    
    // Incrementa el contador
    contador -= 1;
    
    // Actualiza el contenido del div con el nuevo valor del contador
    contadorDiv.textContent = contador;

    console.log('Contador decrementado');
}
    
    // Añade un event listener al botón para escuchar los clics
