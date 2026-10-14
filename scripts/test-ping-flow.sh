#!/bin/bash
# este test verifica que el servidor backend esté vivo
# pegándole al endpoint /ping y revisando que responda HTTP 200
# es una prueba rápida para correr en local o en CI

echo "Iniciando prueba del Ping Flow..."
echo "Verificando servidor en http://127.0.0.1:8000/ping ..."

RESPONSE=$(curl -s -w "\n%{http_code}" http://127.0.0.1:8000/ping)

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)

BODY=$(echo "$RESPONSE" | sed '$d')

# si el código es 200, todo bien. Si no, fallamos con exit 1
# para que CI detecte el error
if [ "$HTTP_CODE" -eq 200 ]; then
    echo "El Servidor respondió con HTTP 200."
    echo "Respuesta: $BODY"
else
    echo "Error: Código HTTP $HTTP_CODE"
    exit 1
fi

echo "Prueba del Ping Flow completada:)"
