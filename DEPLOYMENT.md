# Guía de Despliegue en Debian 12

Esta guía te llevará paso a paso para desplegar ApoloTeams (Rust Teams) en un servidor Debian 12.

## Requisitos Previos

- Servidor Debian 12 con acceso root o sudo
- Mínimo 1GB RAM, 10GB disco
- Puerto 80/443 abierto para tráfico web
- Dominio configurado (opcional pero recomendado)

---

## Paso 1: Actualizar el Sistema

```bash
# Conectarse al servidor
ssh root@tu-servidor-ip

# Actualizar paquetes
apt update && apt upgrade -y

# Instalar dependencias básicas
apt install -y curl wget git build-essential pkg-config libssl-dev sqlite3 nginx certbot python3-certbot-nginx
```

---

## Paso 2: Instalar Rust

```bash
# Instalar Rust usando rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Seleccionar opción 1 (instalación por defecto)

# Cargar las variables de entorno
source $HOME/.cargo/env

# Verificar instalación
rustc --version
cargo --version
```

---

## Paso 3: Crear Usuario de Aplicación

```bash
# Crear usuario para la aplicación (más seguro que usar root)
useradd -m -s /bin/bash apolo
usermod -aG sudo apolo

# Cambiar a usuario apolo
su - apolo
```

---

## Paso 4: Clonar y Compilar el Proyecto

```bash
# Clonar el repositorio
cd /home/apolo
git clone https://github.com/jesus-bazan-entel/ApoloTeams.git
cd ApoloTeams

# Compilar en modo release (optimizado para producción)
cargo build --release -p rust-teams-backend

# El binario estará en: target/release/rust-teams-backend
```

---

## Paso 5: Configurar la Aplicación

```bash
# Crear directorios necesarios
mkdir -p /home/apolo/ApoloTeams/data
mkdir -p /home/apolo/ApoloTeams/uploads
mkdir -p /home/apolo/ApoloTeams/logs

# Crear archivo de configuración
cp .env.example .env
nano .env
```

**Editar `.env` con los siguientes valores:**

```env
# Configuración del Servidor
HOST=127.0.0.1
PORT=8080

# Base de Datos
DATABASE_URL=sqlite:/home/apolo/ApoloTeams/data/apolo_teams.db

# JWT - IMPORTANTE: Cambiar esta clave en producción
JWT_SECRET=tu-clave-secreta-muy-larga-y-segura-cambiar-esto
JWT_EXPIRATION_HOURS=24

# Archivos
UPLOAD_DIR=/home/apolo/ApoloTeams/uploads
MAX_FILE_SIZE_MB=50

# WebSocket
WS_HEARTBEAT_INTERVAL_SECS=30
WS_CLIENT_TIMEOUT_SECS=60

# CORS - Cambiar por tu dominio
CORS_ALLOWED_ORIGINS=https://tu-dominio.com,https://www.tu-dominio.com

# Logging
RUST_LOG=info,rust_teams_backend=info
```

---

## Paso 6: Crear Base de Datos

```bash
# Crear la base de datos SQLite
sqlite3 /home/apolo/ApoloTeams/data/apolo_teams.db < backend/migrations/20240101000001_initial_schema.sql

# Verificar que se creó correctamente
sqlite3 /home/apolo/ApoloTeams/data/apolo_teams.db ".tables"
```

---

## Paso 7: Crear Servicio Systemd

```bash
# Volver a root
exit

# Crear archivo de servicio
nano /etc/systemd/system/apolo-teams.service
```

**Contenido del archivo:**

```ini
[Unit]
Description=ApoloTeams - Rust Teams Clone
After=network.target

[Service]
Type=simple
User=apolo
Group=apolo
WorkingDirectory=/home/apolo/ApoloTeams
Environment="RUST_LOG=info"
ExecStart=/home/apolo/ApoloTeams/target/release/rust-teams-backend
Restart=always
RestartSec=5
StandardOutput=append:/home/apolo/ApoloTeams/logs/apolo-teams.log
StandardError=append:/home/apolo/ApoloTeams/logs/apolo-teams-error.log

# Seguridad
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/home/apolo/ApoloTeams/data /home/apolo/ApoloTeams/uploads /home/apolo/ApoloTeams/logs

[Install]
WantedBy=multi-user.target
```

```bash
# Recargar systemd
systemctl daemon-reload

# Habilitar servicio para inicio automático
systemctl enable apolo-teams

# Iniciar servicio
systemctl start apolo-teams

# Verificar estado
systemctl status apolo-teams
```

---

## Paso 8: Configurar Nginx como Proxy Reverso

```bash
# Crear configuración de Nginx
nano /etc/nginx/sites-available/apolo-teams
```

**Contenido del archivo:**

```nginx
# Configuración para ApoloTeams
upstream apolo_backend {
    server 127.0.0.1:8080;
    keepalive 32;
}

server {
    listen 80;
    server_name tu-dominio.com www.tu-dominio.com;
    
    # Redirigir HTTP a HTTPS (descomentar después de configurar SSL)
    # return 301 https://$server_name$request_uri;

    # Configuración temporal sin SSL
    location / {
        proxy_pass http://apolo_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        
        # Timeouts para WebSocket
        proxy_read_timeout 86400;
        proxy_send_timeout 86400;
    }

    # Archivos estáticos (frontend)
    location /static/ {
        alias /home/apolo/ApoloTeams/frontend/dist/;
        expires 30d;
        add_header Cache-Control "public, immutable";
    }

    # Uploads
    location /uploads/ {
        alias /home/apolo/ApoloTeams/uploads/;
        expires 7d;
    }

    # Límite de tamaño de archivo
    client_max_body_size 50M;
}

# Configuración HTTPS (descomentar después de obtener certificado SSL)
# server {
#     listen 443 ssl http2;
#     server_name tu-dominio.com www.tu-dominio.com;
#
#     ssl_certificate /etc/letsencrypt/live/tu-dominio.com/fullchain.pem;
#     ssl_certificate_key /etc/letsencrypt/live/tu-dominio.com/privkey.pem;
#     ssl_protocols TLSv1.2 TLSv1.3;
#     ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
#     ssl_prefer_server_ciphers off;
#
#     location / {
#         proxy_pass http://apolo_backend;
#         proxy_http_version 1.1;
#         proxy_set_header Upgrade $http_upgrade;
#         proxy_set_header Connection "upgrade";
#         proxy_set_header Host $host;
#         proxy_set_header X-Real-IP $remote_addr;
#         proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
#         proxy_set_header X-Forwarded-Proto $scheme;
#         proxy_cache_bypass $http_upgrade;
#         proxy_read_timeout 86400;
#         proxy_send_timeout 86400;
#     }
#
#     location /static/ {
#         alias /home/apolo/ApoloTeams/frontend/dist/;
#         expires 30d;
#         add_header Cache-Control "public, immutable";
#     }
#
#     location /uploads/ {
#         alias /home/apolo/ApoloTeams/uploads/;
#         expires 7d;
#     }
#
#     client_max_body_size 50M;
# }
```

```bash
# Habilitar sitio
ln -s /etc/nginx/sites-available/apolo-teams /etc/nginx/sites-enabled/

# Eliminar configuración por defecto
rm /etc/nginx/sites-enabled/default

# Verificar configuración
nginx -t

# Reiniciar Nginx
systemctl restart nginx
```

---

## Paso 9: Configurar SSL con Let's Encrypt (Recomendado)

```bash
# Obtener certificado SSL
certbot --nginx -d tu-dominio.com -d www.tu-dominio.com

# Seguir las instrucciones interactivas
# Seleccionar opción para redirigir HTTP a HTTPS

# Verificar renovación automática
certbot renew --dry-run
```

---

## Paso 10: Configurar Firewall

```bash
# Instalar UFW si no está instalado
apt install -y ufw

# Configurar reglas
ufw default deny incoming
ufw default allow outgoing
ufw allow ssh
ufw allow 80/tcp
ufw allow 443/tcp

# Habilitar firewall
ufw enable

# Verificar estado
ufw status
```

---

## Paso 11: Compilar y Desplegar Frontend

```bash
# Cambiar a usuario apolo
su - apolo
cd /home/apolo/ApoloTeams

# Instalar trunk (herramienta de build para Dioxus)
cargo install trunk

# Instalar target wasm
rustup target add wasm32-unknown-unknown

# Compilar frontend
cd frontend
trunk build --release

# Los archivos estarán en frontend/dist/
```

---

## Comandos Útiles de Administración

### Ver logs en tiempo real
```bash
journalctl -u apolo-teams -f
# o
tail -f /home/apolo/ApoloTeams/logs/apolo-teams.log
```

### Reiniciar servicio
```bash
systemctl restart apolo-teams
```

### Ver estado del servicio
```bash
systemctl status apolo-teams
```

### Actualizar aplicación
```bash
su - apolo
cd /home/apolo/ApoloTeams
git pull origin main
cargo build --release -p rust-teams-backend
exit
systemctl restart apolo-teams
```

### Backup de base de datos
```bash
sqlite3 /home/apolo/ApoloTeams/data/apolo_teams.db ".backup '/home/apolo/backups/apolo_teams_$(date +%Y%m%d).db'"
```

---

## Monitoreo y Mantenimiento

### Configurar logrotate
```bash
nano /etc/logrotate.d/apolo-teams
```

```
/home/apolo/ApoloTeams/logs/*.log {
    daily
    missingok
    rotate 14
    compress
    delaycompress
    notifempty
    create 0640 apolo apolo
    sharedscripts
    postrotate
        systemctl reload apolo-teams > /dev/null 2>&1 || true
    endscript
}
```

### Script de backup automático
```bash
nano /home/apolo/backup.sh
```

```bash
#!/bin/bash
BACKUP_DIR="/home/apolo/backups"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR

# Backup base de datos
sqlite3 /home/apolo/ApoloTeams/data/apolo_teams.db ".backup '$BACKUP_DIR/db_$DATE.db'"

# Backup uploads
tar -czf $BACKUP_DIR/uploads_$DATE.tar.gz /home/apolo/ApoloTeams/uploads/

# Eliminar backups antiguos (más de 7 días)
find $BACKUP_DIR -type f -mtime +7 -delete

echo "Backup completado: $DATE"
```

```bash
chmod +x /home/apolo/backup.sh

# Agregar a crontab (ejecutar diariamente a las 3am)
crontab -e
# Agregar línea:
0 3 * * * /home/apolo/backup.sh >> /home/apolo/ApoloTeams/logs/backup.log 2>&1
```

---

## Solución de Problemas

### El servicio no inicia
```bash
# Ver logs detallados
journalctl -u apolo-teams -n 100 --no-pager

# Verificar permisos
ls -la /home/apolo/ApoloTeams/
chown -R apolo:apolo /home/apolo/ApoloTeams/
```

### Error de conexión a base de datos
```bash
# Verificar que existe el archivo
ls -la /home/apolo/ApoloTeams/data/

# Verificar permisos
chmod 664 /home/apolo/ApoloTeams/data/apolo_teams.db
```

### WebSocket no conecta
```bash
# Verificar configuración de Nginx
nginx -t

# Verificar que el backend está escuchando
ss -tlnp | grep 8080
```

### Error 502 Bad Gateway
```bash
# Verificar que el backend está corriendo
systemctl status apolo-teams

# Reiniciar servicios
systemctl restart apolo-teams
systemctl restart nginx
```

---

## Resumen de Puertos

| Puerto | Servicio | Descripción |
|--------|----------|-------------|
| 22 | SSH | Acceso remoto |
| 80 | HTTP | Tráfico web (redirige a HTTPS) |
| 443 | HTTPS | Tráfico web seguro |
| 8080 | Backend | Solo localhost (proxy por Nginx) |

---

## Checklist Final

- [ ] Sistema actualizado
- [ ] Rust instalado
- [ ] Usuario apolo creado
- [ ] Proyecto clonado y compilado
- [ ] Archivo .env configurado
- [ ] Base de datos creada
- [ ] Servicio systemd configurado y activo
- [ ] Nginx configurado como proxy
- [ ] SSL configurado con Let's Encrypt
- [ ] Firewall configurado
- [ ] Frontend compilado
- [ ] Backups automáticos configurados
- [ ] Logrotate configurado

---

¡Tu aplicación ApoloTeams está lista para producción! 🚀
