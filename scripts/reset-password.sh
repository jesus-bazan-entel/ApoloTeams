#!/bin/bash
# Reset password for an ApoloTeams user
# Usage: ./scripts/reset-password.sh <email_or_username>

set -e

DB_URL="${DATABASE_URL:-postgresql://rust_user:ApoloNext.2026@localhost:5432/rust_teams}"

if [ -z "$1" ]; then
    echo "Uso: $0 <email_o_username>"
    echo ""
    echo "Ejemplo:"
    echo "  $0 admin@example.com"
    echo "  $0 admin"
    echo ""
    echo "Usuarios registrados:"
    psql "$DB_URL" -t -A -c "SELECT username || ' (' || email || ')' FROM users ORDER BY username;"
    exit 1
fi

IDENTIFIER="$1"

# Check if user exists
USER_INFO=$(psql "$DB_URL" -t -A -c "SELECT id, username, email FROM users WHERE email = '$IDENTIFIER' OR username = '$IDENTIFIER' LIMIT 1;" 2>/dev/null)

if [ -z "$USER_INFO" ]; then
    echo "Error: No se encontro usuario con email/username '$IDENTIFIER'"
    echo ""
    echo "Usuarios disponibles:"
    psql "$DB_URL" -t -A -c "SELECT username || ' (' || email || ')' FROM users ORDER BY username;"
    exit 1
fi

USER_ID=$(echo "$USER_INFO" | cut -d'|' -f1)
USERNAME=$(echo "$USER_INFO" | cut -d'|' -f2)
EMAIL=$(echo "$USER_INFO" | cut -d'|' -f3)

echo "Usuario encontrado:"
echo "  ID:       $USER_ID"
echo "  Username: $USERNAME"
echo "  Email:    $EMAIL"
echo ""

# Prompt for new password
read -s -p "Nueva contraseña: " NEW_PASSWORD
echo ""
read -s -p "Confirmar contraseña: " CONFIRM_PASSWORD
echo ""

if [ "$NEW_PASSWORD" != "$CONFIRM_PASSWORD" ]; then
    echo "Error: Las contraseñas no coinciden."
    exit 1
fi

if [ ${#NEW_PASSWORD} -lt 6 ]; then
    echo "Error: La contraseña debe tener al menos 6 caracteres."
    exit 1
fi

# Generate Argon2id hash using Python (compatible with Rust argon2 crate defaults)
HASH=$(python3 -c "
import sys
from argon2 import PasswordHasher
ph = PasswordHasher()
print(ph.hash(sys.stdin.read().rstrip('\n')))
" <<< "$NEW_PASSWORD" 2>/dev/null)

if [ -z "$HASH" ]; then
    echo "Error: No se pudo generar el hash. Instala argon2-cffi:"
    echo "  pip3 install argon2-cffi"
    exit 1
fi

# Update password in database (use dollar-quoting to avoid $argon2id$ being interpreted)
psql "$DB_URL" -c "UPDATE users SET password_hash = \$\$${HASH}\$\$, updated_at = NOW() WHERE id = '${USER_ID}';" > /dev/null

echo ""
echo "Contraseña actualizada exitosamente para '$USERNAME'."
