#!/bin/bash

# Define JSON data (respetando el formato exacto)
JSON_DATA='{"name":"test_api_folder_new","parent_id":null}'
CONTENT_LENGTH=$(echo -n "$JSON_DATA" | wc -c)

# Crear carpeta mediante API
echo "Creando carpeta 'test_api_folder_new'..."
(echo -e "POST /api/folders HTTP/1.1\r
Host: localhost\r
Content-Type: application/json\r
Content-Length: $CONTENT_LENGTH\r
Connection: close\r
\r
$JSON_DATA" | nc localhost 8085) > /tmp/folder_response.txt

cat /tmp/folder_response.txt
echo ""

# Verificar si la carpeta se creó
sleep 1
echo "Verificando directorio..."
ls -la /home/torrefacto/OxiCloud/storage/