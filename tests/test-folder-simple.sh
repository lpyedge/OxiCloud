#!/bin/bash

# Crear la carpeta directamente
echo "Creando carpeta de prueba directamente..."
mkdir -p /home/torrefacto/OxiCloud/storage/prueba123

# Verificar las carpetas
echo "Verificando carpetas existentes..."
ls -la /home/torrefacto/OxiCloud/storage/

# Reiniciar el servidor
echo "Reiniciando el servidor..."
pkill -9 -f "oxicloud"
sleep 2
cd /home/torrefacto/OxiCloud && cargo run > /tmp/oxicloud.log 2>&1 &
sleep 3

# Comprobación de interfaz web
echo "Reinicio completado. Intenta ahora en tu navegador crear una carpeta y ver si aparece."