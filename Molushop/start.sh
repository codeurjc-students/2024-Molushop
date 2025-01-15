#!/bin/bash

# Inicializar la variable de rutas
SASS_PATHS1="ui/components/create_product/static/styles:target_css/components/create_product"
SASS_PATHS2="ui/pages/create_product/static/styles:target_css/pages/create_product"

# Ejecutar el primer comando sass --watch en una nueva terminal
#start cmd /k "sass.bat --watch $SASS_PATHS1"

# Ejecutar el segundo comando sass --watch en otra nueva terminal
#start cmd /k "sass.bat --watch $SASS_PATHS2"

#start "" "C:\Program Files\Git\bin\bash.exe" --login

start "" "C:\Program Files\Git\bin\bash.exe" -c "sass.bat --watch $SASS_PATHS1; exec bash"
start "" "C:\Program Files\Git\bin\bash.exe" -c "sass.bat --watch $SASS_PATHS2; exec bash"