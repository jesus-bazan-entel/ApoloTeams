# Troubleshooting Guide for Rust Teams

## Table of Contents
1. [Frontend Not Loading](#frontend-not-loading)
2. [Backend Issues](#backend-issues)
3. [Build Errors](#build-errors)
4. [Database Issues](#database-issues)
5. [WebSocket Issues](#websocket-issues)

---

## Frontend Not Loading

### Symptoms
- Blank page when accessing the application
- 404 errors for static files
- JavaScript/WASM errors in browser console

### Solutions

#### 1. Check if frontend files exist
```bash
ls -la backend/static/
```
You should see:
- `index.html`
- `frontend-*.js`
- `frontend-*.wasm`
- Other static assets

#### 2. Rebuild and copy frontend
```bash
cd frontend
trunk build --release
cd ..
cp -r frontend/dist/* backend/static/
```

#### 3. Check browser console for errors
Open Developer Tools (F12) → Console tab

Common errors:
- **CORS errors**: Backend CORS is misconfigured
- **WASM loading errors**: WASM file not found or corrupted
- **API errors**: Backend not running or wrong URL

#### 4. Verify WASM MIME type
The server must serve `.wasm` files with `application/wasm` MIME type.

Check nginx config if using reverse proxy:
```nginx
types {
    application/wasm wasm;
}
```

---

## Backend Issues

### Backend won't start

#### 1. Check if port is in use
```bash
ss -tlnp | grep 8080
# or
lsof -i :8080
```

Kill the process if needed:
```bash
kill -9 <PID>
```

#### 2. Check logs
```bash
# If using systemd
journalctl -u rust-teams -f

# If running directly
RUST_LOG=debug ./target/release/backend
```

#### 3. Check configuration
```bash
cat backend/.env
```

Required environment variables:
```
DATABASE_URL=sqlite:./data/rust_teams.db
JWT_SECRET=your-secret-key-here
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
```

#### 4. Check database file permissions
```bash
ls -la backend/data/
chmod 644 backend/data/rust_teams.db
```

### API returns 500 errors

#### 1. Enable debug logging
```bash
RUST_LOG=debug,sqlx=debug ./target/release/backend
```

#### 2. Check database migrations
```bash
cd backend
cargo sqlx migrate run
```

---

## Build Errors

### Frontend build fails

#### 1. Install required tools
```bash
# Install trunk
cargo install trunk

# Install wasm target
rustup target add wasm32-unknown-unknown
```

#### 2. Clear cache and rebuild
```bash
cd frontend
rm -rf dist/
cargo clean
trunk build --release
```

#### 3. Check Dioxus version compatibility
Ensure all Dioxus dependencies use the same version (0.5.x).

### Backend build fails

#### 1. Check SQLx offline mode
```bash
cd backend
cargo sqlx prepare
```

#### 2. Install system dependencies (Debian/Ubuntu)
```bash
sudo apt-get install libssl-dev pkg-config
```

---

## Database Issues

### Database locked error
```
Error: database is locked
```

Solution:
```bash
# Stop all processes using the database
pkill -f backend

# Remove lock file if exists
rm -f backend/data/rust_teams.db-wal
rm -f backend/data/rust_teams.db-shm

# Restart backend
./target/release/backend
```

### Migration errors

#### 1. Reset database (WARNING: deletes all data)
```bash
rm -f backend/data/rust_teams.db
./target/release/backend  # Will recreate database
```

#### 2. Check migration files
```bash
ls -la backend/migrations/
```

---

## WebSocket Issues

### WebSocket connection fails

#### 1. Check WebSocket URL
The frontend should connect to:
- Development: `ws://localhost:8080/ws`
- Production: `wss://your-domain.com/ws`

#### 2. Check nginx proxy configuration (if using)
```nginx
location /ws {
    proxy_pass http://127.0.0.1:8080;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_set_header Host $host;
    proxy_read_timeout 86400;
}
```

#### 3. Check firewall
```bash
sudo ufw allow 8080/tcp
```

---

## Quick Health Check

Run these commands to verify the system is working:

```bash
# 1. Check backend health
curl http://localhost:8080/health

# 2. Check API is responding
curl http://localhost:8080/api/v1/auth/login -X POST \
  -H "Content-Type: application/json" \
  -d '{"email":"test@test.com","password":"test"}'

# 3. Check frontend files are served
curl -I http://localhost:8080/

# 4. Check WebSocket endpoint
curl -I http://localhost:8080/ws
```

---

## Getting Help

If you're still having issues:

1. Check the logs with debug level: `RUST_LOG=debug`
2. Open an issue on GitHub with:
   - Error message
   - Steps to reproduce
   - System information (OS, Rust version)
   - Relevant log output
