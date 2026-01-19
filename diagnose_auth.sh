#!/bin/bash

# Script de diagnóstico completo para autenticación
# Verifica base de datos, registro y login

API_URL="http://127.0.0.1:8080/api/v1"
DB_PATH="/home/apolo/ApoloTeams/data/apolo_teams.db"

echo "=========================================="
echo "  Diagnóstico de Autenticación"
echo "=========================================="
echo ""

# 1. Verificar que el servidor está corriendo
echo "1. Verificando servidor..."
HEALTH=$(curl -s "${API_URL}/../health" 2>/dev/null)
if echo "$HEALTH" | grep -q "healthy"; then
    echo "   ✅ Servidor está corriendo"
else
    echo "   ❌ Servidor no responde"
    exit 1
fi
echo ""

# 2. Verificar base de datos
echo "2. Verificando base de datos..."
if [ -f "$DB_PATH" ]; then
    echo "   ✅ Base de datos existe"
    ls -la "$DB_PATH"
else
    echo "   ❌ Base de datos no existe"
    exit 1
fi
echo ""

# 3. Verificar permisos de la base de datos
echo "3. Verificando permisos..."
DB_PERMS=$(stat -c "%a" "$DB_PATH" 2>/dev/null || stat -f "%OLp" "$DB_PATH" 2>/dev/null)
DB_OWNER=$(stat -c "%U" "$DB_PATH" 2>/dev/null || stat -f "%Su" "$DB_PATH" 2>/dev/null)
echo "   Permisos: $DB_PERMS"
echo "   Propietario: $DB_OWNER"
if [ "$DB_OWNER" = "apolo" ]; then
    echo "   ✅ Permisos correctos"
else
    echo "   ❌ Permisos incorrectos - debería ser 'apolo'"
    echo "   Ejecuta: sudo chown apolo:apolo $DB_PATH"
fi
echo ""

# 4. Listar usuarios en la base de datos
echo "4. Listando usuarios en la base de datos..."
sqlite3 "$DB_PATH" "SELECT id, email, username, display_name, created_at FROM users;" 2>/dev/null || echo "   ❌ No se pudo leer la base de datos"
echo ""

# 5. Contar usuarios
echo "5. Contando usuarios..."
USER_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM users;" 2>/dev/null || echo "0")
echo "   Total de usuarios: $USER_COUNT"
echo ""

# 6. Probar registro con un usuario nuevo
TIMESTAMP=$(date +%s)
TEST_EMAIL="diag${TIMESTAMP}@example.com"
TEST_USERNAME="diaguser${TIMESTAMP}"
TEST_PASSWORD="password123"

echo "6. Probando registro..."
echo "   Email: $TEST_EMAIL"
echo "   Username: $TEST_USERNAME"

REGISTER_RESPONSE=$(curl -s -X POST "${API_URL}/auth/register" \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"${TEST_EMAIL}\",
    \"username\": \"${TEST_USERNAME}\",
    \"display_name\": \"Diag User\",
    \"password\": \"${TEST_PASSWORD}\"
  }")

echo "   Respuesta:"
echo "$REGISTER_RESPONSE" | jq . 2>/dev/null || echo "$REGISTER_RESPONSE"
echo ""

if echo "$REGISTER_RESPONSE" | grep -q "access_token"; then
    echo "   ✅ Registro exitoso"
    REGISTERED_EMAIL="$TEST_EMAIL"
    REGISTERED_PASSWORD="$TEST_PASSWORD"
else
    echo "   ❌ Registro falló"
    echo "   Intentando usar usuario existente..."
    REGISTERED_EMAIL="test@example.com"
    REGISTERED_PASSWORD="password123"
fi
echo ""

# 7. Probar login
echo "7. Probando login..."
echo "   Email: $REGISTERED_EMAIL"
echo "   Password: $REGISTERED_PASSWORD"

LOGIN_RESPONSE=$(curl -s -X POST "${API_URL}/auth/login" \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"${REGISTERED_EMAIL}\",
    \"password\": \"${REGISTERED_PASSWORD}\"
  }")

echo "   Respuesta:"
echo "$LOGIN_RESPONSE" | jq . 2>/dev/null || echo "$LOGIN_RESPONSE"
echo ""

if echo "$LOGIN_RESPONSE" | grep -q "access_token"; then
    echo "   ✅ Login exitoso"
    
    # Extraer token
    ACCESS_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.access_token' 2>/dev/null | tr -d '"')
    USER_ID=$(echo "$LOGIN_RESPONSE" | jq -r '.user.id' 2>/dev/null)
    
    echo ""
    echo "8. Probando endpoint protegido..."
    ME_RESPONSE=$(curl -s -X GET "${API_URL}/users/me" \
      -H "Authorization: Bearer ${ACCESS_TOKEN}")
    
    echo "   Respuesta:"
    echo "$ME_RESPONSE" | jq . 2>/dev/null || echo "$ME_RESPONSE"
    echo ""
    
    if echo "$ME_RESPONSE" | grep -q '"error"'; then
        echo "   ❌ Error en endpoint protegido"
    elif echo "$ME_RESPONSE" | grep -q "id"; then
        echo "   ✅ Endpoint protegido funciona"
    fi
else
    echo "   ❌ Login falló"
fi

echo ""
echo "=========================================="
echo "  Diagnóstico completado"
echo "=========================================="
