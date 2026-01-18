#!/bin/bash

# Script de prueba de autenticación para Rust Teams
# Prueba registro y login automáticamente

API_URL="http://127.0.0.1:8080/api/v1"

echo "=========================================="
echo "  Prueba de Autenticación - Rust Teams"
echo "=========================================="
echo ""

# Generar email único con timestamp
TIMESTAMP=$(date +%s)
EMAIL="test${TIMESTAMP}@example.com"
USERNAME="testuser${TIMESTAMP}"
PASSWORD="password123"
DISPLAY_NAME="Test User ${TIMESTAMP}"

echo "1. Probando registro..."
echo "   Email: $EMAIL"
echo "   Username: $USERNAME"
echo ""

REGISTER_RESPONSE=$(curl -s -X POST "${API_URL}/auth/register" \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"${EMAIL}\",
    \"username\": \"${USERNAME}\",
    \"display_name\": \"${DISPLAY_NAME}\",
    \"password\": \"${PASSWORD}\"
  }")

echo "Respuesta del registro:"
echo "$REGISTER_RESPONSE" | jq . 2>/dev/null || echo "$REGISTER_RESPONSE"
echo ""

# Verificar si el registro fue exitoso
if echo "$REGISTER_RESPONSE" | grep -q "access_token"; then
    echo "✅ Registro exitoso!"
    echo ""

    # Extraer el token
    ACCESS_TOKEN=$(echo "$REGISTER_RESPONSE" | jq -r '.access_token' 2>/dev/null)
    USER_ID=$(echo "$REGISTER_RESPONSE" | jq -r '.user.id' 2>/dev/null)

    echo "2. Probando login con las mismas credenciales..."
    LOGIN_RESPONSE=$(curl -s -X POST "${API_URL}/auth/login" \
      -H "Content-Type: application/json" \
      -d "{
        \"email\": \"${EMAIL}\",
        \"password\": \"${PASSWORD}\"
      }")

    echo "Respuesta del login:"
    echo "$LOGIN_RESPONSE" | jq . 2>/dev/null || echo "$LOGIN_RESPONSE"
    echo ""

    if echo "$LOGIN_RESPONSE" | grep -q "access_token"; then
        echo "✅ Login exitoso!"
        echo ""

        # Extraer el token de login
        LOGIN_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.access_token' 2>/dev/null)

        echo "3. Probando endpoint protegido (/users/me)..."
        ME_RESPONSE=$(curl -s -X GET "${API_URL}/users/me" \
          -H "Authorization: Bearer ${LOGIN_TOKEN}")

        echo "Respuesta de /users/me:"
        echo "$ME_RESPONSE" | jq . 2>/dev/null || echo "$ME_RESPONSE"
        echo ""

        if echo "$ME_RESPONSE" | grep -q "id"; then
            echo "✅ Endpoint protegido funciona correctamente!"
            echo ""
            echo "=========================================="
            echo "  TODAS LAS PRUEBAS PASARON ✅"
            echo "=========================================="
            echo ""
            echo "Credenciales de prueba:"
            echo "  Email: ${EMAIL}"
            echo "  Password: ${PASSWORD}"
            echo "  User ID: ${USER_ID}"
            echo ""
        else
            echo "❌ Error en endpoint protegido"
        fi
    else
        echo "❌ Login falló"
    fi
else
    echo "❌ Registro falló"
fi

echo ""
echo "=========================================="
