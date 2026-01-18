# Guía de Despliegue en Debian 12

Esta guía detalla los pasos para desplegar ApoloTeams en un servidor Debian 12.

## Requisitos Previos

- Servidor Debian 12 con acceso root o sudo
- Mínimo 2GB RAM, 20GB disco
- Dominio configurado (opcional, para HTTPS)

## 1. Actualizar el Sistema

```bash
sudo apt update && sudo apt upgrade -y
```

## 2. Instalar Dependencias del Sistema

```bash
# Herramientas básicas
sudo apt install -y curl wget git build-essential pkg-config libssl-dev

# PostgreSQL
sudo apt install -y postgresql postgresql-contrib

# Node.js 20 LTS
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Nginx (para reverse proxy)
sudo apt install -y nginx

# Certbot para SSL (opcional)
sudo apt install -y certbot python3-certbot-nginx
```

## 3. Instalar Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verificar instalación
rustc --version
cargo --version
```

## 4. Configurar PostgreSQL

```bash
# Iniciar PostgreSQL
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Crear usuario y base de datos
sudo -u postgres psql << EOF
CREATE USER rust_teams WITH PASSWORD 'tu_password_seguro';
CREATE DATABASE rust_teams OWNER rust_teams;
GRANT ALL PRIVILEGES ON DATABASE rust_teams TO rust_teams;
\q
EOF
```

## 5. Clonar el Repositorio

```bash
cd /opt
sudo git clone https://github.com/jesus-bazan-entel/ApoloTeams.git
sudo chown -R $USER:$USER /opt/ApoloTeams
cd /opt/ApoloTeams
```

## 6. Configurar Variables de Entorno

```bash
# Crear archivo .env
cat > .env << EOF
# Server
HOST=127.0.0.1
PORT=8080

# Database PostgreSQL
DATABASE_URL=postgresql://rust_teams:tu_password_seguro@localhost:5432/rust_teams

# JWT
JWT_SECRET=$(openssl rand -base64 32)
JWT_EXPIRATION=86400

# CORS
CORS_ORIGINS=https://tu-dominio.com

# Logging
RUST_LOG=info
EOF
```

## 7. Ejecutar Migraciones de Base de Datos

```bash
# Instalar sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Ejecutar migraciones
cd backend
sqlx migrate run
cd ..
```

## 8. Compilar el Backend

```bash
cd backend
cargo build --release
cd ..
```

## 9. Compilar el Frontend React

```bash
cd frontend-react

# Instalar dependencias
npm install

# Crear archivo de configuración de producción
cat > .env.production << EOF
VITE_API_URL=https://tu-dominio.com/api
VITE_WS_URL=wss://tu-dominio.com/ws
EOF

# Compilar para producción
npm run build
cd ..
```

## 10. Configurar Nginx

```bash
sudo nano /etc/nginx/sites-available/apolo-teams
```

Contenido del archivo:

```nginx
server {
    listen 80;
    server_name tu-dominio.com;

    # Frontend React (archivos estáticos)
    location / {
        root /opt/ApoloTeams/frontend-react/dist;
        index index.html;
        try_files $uri $uri/ /index.html;
    }

    # API Backend
    location /api {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }

    # WebSocket
    location /ws {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_read_timeout 86400;
    }
}
```

Activar el sitio:

```bash
sudo ln -s /etc/nginx/sites-available/apolo-teams /etc/nginx/sites-enabled/
sudo rm /etc/nginx/sites-enabled/default
sudo nginx -t
sudo systemctl restart nginx
```

## 11. Configurar SSL con Certbot (Opcional pero Recomendado)

```bash
sudo certbot --nginx -d tu-dominio.com
```

## 12. Crear Servicio Systemd para el Backend

```bash
sudo nano /etc/systemd/system/apolo-teams.service
```

Contenido:

```ini
[Unit]
Description=ApoloTeams Backend Service
After=network.target postgresql.service

[Service]
Type=simple
User=www-data
Group=www-data
WorkingDirectory=/opt/ApoloTeams
Environment="RUST_LOG=info"
EnvironmentFile=/opt/ApoloTeams/.env
ExecStart=/opt/ApoloTeams/backend/target/release/backend
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Activar el servicio:

```bash
# Dar permisos
sudo chown -R www-data:www-data /opt/ApoloTeams

# Recargar systemd
sudo systemctl daemon-reload

# Iniciar y habilitar el servicio
sudo systemctl start apolo-teams
sudo systemctl enable apolo-teams

# Verificar estado
sudo systemctl status apolo-teams
```

## 13. Configurar Firewall (UFW)

```bash
sudo apt install -y ufw
sudo ufw allow ssh
sudo ufw allow 'Nginx Full'
sudo ufw enable
```

## 14. Verificar el Despliegue

```bash
# Verificar que el backend está corriendo
curl http://localhost:8080/api/health

# Verificar logs
sudo journalctl -u apolo-teams -f
```

## Comandos Útiles

### Reiniciar servicios
```bash
sudo systemctl restart apolo-teams
sudo systemctl restart nginx
sudo systemctl restart postgresql
```

### Ver logs
```bash
# Backend
sudo journalctl -u apolo-teams -f

# Nginx
sudo tail -f /var/log/nginx/error.log
sudo tail -f /var/log/nginx/access.log

# PostgreSQL
sudo tail -f /var/log/postgresql/postgresql-15-main.log
```

### Actualizar la aplicación
```bash
cd /opt/ApoloTeams
git pull origin main

# Recompilar backend
cd backend
cargo build --release
cd ..

# Recompilar frontend
cd frontend-react
npm install
npm run build
cd ..

# Reiniciar servicio
sudo systemctl restart apolo-teams
```

## Solución de Problemas

### El backend no inicia
1. Verificar variables de entorno: `cat /opt/ApoloTeams/.env`
2. Verificar conexión a PostgreSQL: `psql -U rust_teams -d rust_teams -h localhost`
3. Ver logs: `sudo journalctl -u apolo-teams -n 50`

### Error de conexión a la base de datos
1. Verificar que PostgreSQL está corriendo: `sudo systemctl status postgresql`
2. Verificar credenciales en `.env`
3. Verificar que el usuario tiene permisos: `sudo -u postgres psql -c "\du"`

### Frontend no carga
1. Verificar que los archivos existen: `ls -la /opt/ApoloTeams/frontend-react/dist`
2. Verificar configuración de Nginx: `sudo nginx -t`
3. Ver logs de Nginx: `sudo tail -f /var/log/nginx/error.log`

### WebSocket no conecta
1. Verificar configuración de Nginx para `/ws`
2. Verificar que el backend está escuchando WebSocket
3. Verificar CORS en `.env`

## Seguridad Adicional

### Configurar fail2ban
```bash
sudo apt install -y fail2ban
sudo systemctl enable fail2ban
sudo systemctl start fail2ban
```

### Configurar actualizaciones automáticas
```bash
sudo apt install -y unattended-upgrades
sudo dpkg-reconfigure -plow unattended-upgrades
```

---

## Resumen de Puertos

| Servicio | Puerto | Descripción |
|----------|--------|-------------|
| Nginx HTTP | 80 | Redirige a HTTPS |
| Nginx HTTPS | 443 | Frontend y API |
| Backend | 8080 | Solo localhost |
| PostgreSQL | 5432 | Solo localhost |

## Estructura de Archivos en el Servidor

```
/opt/ApoloTeams/
├── .env                          # Variables de entorno
├── backend/
│   └── target/release/backend    # Binario compilado
├── frontend-react/
│   └── dist/                     # Frontend compilado
└── backend/migrations/           # Migraciones SQL
```
