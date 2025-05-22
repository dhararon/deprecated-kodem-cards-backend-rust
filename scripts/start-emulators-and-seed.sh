#!/bin/bash

# Definir variables
PROJECT_ID="kodemcards"
EMULATOR_HOST="0.0.0.0"
MAX_RETRIES=30
RETRY_INTERVAL=2

# Exportar variable DOCKER_CONTAINER para que el script de seeding sepa que está en Docker
export DOCKER_CONTAINER=true

# Iniciar los emuladores de Firebase en segundo plano
echo "🔥 Iniciando emuladores de Firebase..."
firebase emulators:start --project "$PROJECT_ID" &
EMULATOR_PID=$!

# Función para verificar si los emuladores están listos
function check_emulators() {
    # Verificar emulador Auth (puerto 9099)
    local auth_response=$(curl -s "http://$EMULATOR_HOST:9099/emulator/v1/projects/$PROJECT_ID/status" || echo "")
    # Verificar emulador Firestore (puerto 8080)
    local firestore_response=$(curl -s "http://$EMULATOR_HOST:8080/emulator/v1/projects/$PROJECT_ID/status" || echo "")
    # Verificar UI del emulador (puerto 4000)
    local ui_response=$(curl -s "http://$EMULATOR_HOST:4000" -o /dev/null -w '%{http_code}' || echo "")
    
    echo "Auth response: $auth_response"
    echo "Firestore response: $firestore_response"
    echo "UI response: $ui_response"
    
    # Verificar que todos los servicios estén respondiendo correctamente
    if [[ -n "$auth_response" && -n "$firestore_response" && "$ui_response" == "200" ]]; then
        return 0
    else
        return 1
    fi
}

# Esperar a que los emuladores estén listos
echo "⏳ Esperando a que los emuladores estén listos..."
sleep 10  # Espera inicial para permitir que los emuladores se inicien
RETRY_COUNT=0
while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if check_emulators; then
        echo "✅ Emuladores listos y funcionando"
        
        # Esperar un tiempo adicional para asegurar la estabilidad
        echo "⏳ Esperando 5 segundos adicionales para asegurar la estabilidad..."
        sleep 5
        break
    fi
    RETRY_COUNT=$((RETRY_COUNT+1))
    echo "⏳ Esperando a que los emuladores estén listos... intento $RETRY_COUNT de $MAX_RETRIES"
    sleep $RETRY_INTERVAL
done

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
    echo "❌ Los emuladores no pudieron iniciarse correctamente después de $MAX_RETRIES intentos"
    kill $EMULATOR_PID
    exit 1
fi

# Ejecutar prueba básica de conectividad
echo "🔍 Realizando prueba de conectividad a los emuladores..."
curl -s "http://$EMULATOR_HOST:9099/identitytoolkit.googleapis.com/v1/projects/$PROJECT_ID/accounts" || echo "Error en Auth API"
curl -s "http://$EMULATOR_HOST:8080/v1/projects/$PROJECT_ID/databases/(default)/documents" || echo "Error en Firestore API"
echo "✅ Prueba de conectividad completada"

# Configurar el PATH para acceder a bun
export PATH=$PATH:~/.bun/bin

# Ejecutar el script de seeding de usuarios
echo "🌱 Ejecutando script de seeding de usuarios..."
bun run scripts/seed-firebase-users.ts

# Si el script de seeding de usuarios se ejecutó correctamente
if [ $? -eq 0 ]; then
    echo "✅ Script de seeding de usuarios ejecutado con éxito"
else
    echo "❌ Error al ejecutar el script de seeding de usuarios"
    # Intentar ejecutar nuevamente después de un tiempo
    echo "🔄 Intentando ejecutar el script de usuarios nuevamente después de 10 segundos..."
    sleep 10
    echo "🌱 Segundo intento de ejecución del script de seeding de usuarios..."
    bun run scripts/seed-firebase-users.ts
fi

# Esperar un momento antes de continuar con el siguiente script
sleep 5

# Ejecutar el script de seeding de cartas
echo "🌱 Ejecutando script de seeding de cartas..."
bun run scripts/seed-firebase-cards.ts

# Si el script de seeding de cartas se ejecutó correctamente
if [ $? -eq 0 ]; then
    echo "✅ Script de seeding de cartas ejecutado con éxito"
else
    echo "❌ Error al ejecutar el script de seeding de cartas"
    # Intentar ejecutar nuevamente después de un tiempo
    echo "🔄 Intentando ejecutar el script de cartas nuevamente después de 10 segundos..."
    sleep 10
    echo "🌱 Segundo intento de ejecución del script de seeding de cartas..."
    bun run scripts/seed-firebase-cards.ts
fi

# Mantener los emuladores en ejecución (no finalizar el script)
echo "✅ Seeding completo. Emuladores listos con datos iniciales."
echo "🔄 Emuladores en ejecución. Presiona Ctrl+C para detener."
wait $EMULATOR_PID 