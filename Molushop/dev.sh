#!/bin/bash

# Inicializar la variable de rutas
SASS_PATHS=""

# Función para agregar rutas a la variable
add_path() {
    SASS_PATHS="$SASS_PATHS $1:$2"
}

# Agregar rutas
add_path "ui/components/create_product/static/styles" "target_css/components/create_product"
add_path "ui/pages/create_product/static/styles" "target_css/pages/create_product"

echo "Rutas acumuladas: $SASS_PATHS"

# Ejecutar el comando sass con las rutas acumuladas
sass.bat --watch $SASS_PATHS