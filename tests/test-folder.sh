#!/bin/bash

# Crear una carpeta nueva
echo "Creando carpeta nueva..."
curl -v -X POST -H "Content-Type: application/json" -d '{"name":"test_folder_script","parent_id":null}' http://localhost:8085/api/folders

# Listar las carpetas
echo -e "\nListando carpetas..."
curl -v http://localhost:8085/api/folders