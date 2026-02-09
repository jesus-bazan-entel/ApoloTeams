# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ApoloTeams is a Microsoft Teams clone with a Rust backend and React frontend. Real-time messaging via WebSockets, JWT authentication, file sharing, and video/audio calls.

## Build & Run Commands

```bash
# Backend (Rust/Actix-web) - binary is "rust-teams-server"
cargo run -p backend                    # Dev server at http://localhost:8080
cargo build --release -p backend        # Production build
cargo check -p backend                  # Type check without building

# Frontend (React/Vite) - in frontend-react/
cd frontend-react
npm install                             # Install dependencies (first time)
npm run dev                             # Dev server at http://localhost:5173
npm run build                           # Production build (runs tsc first)
npm run lint                            # ESLint check
npm run type-check                      # TypeScript check only

# Testing
cargo test --workspace                  # Run all Rust tests
cargo test -p backend                   # Backend tests only
cargo test -p backend test_name         # Run single test by name

# Code quality
cargo fmt --all                         # Format Rust code
cargo clippy --all                      # Lint Rust code
```

## Architecture

```
┌─────────────────┐     ┌──────────────────────────────────────────┐
│  React Frontend │────▶│  Actix-web Backend                       │
│  (Vite/Zustand/ │     │  Handlers → Services → DB (Repository)   │
│   Tailwind)     │◀────│  :8080                                   │
│  :5173          │     └──────────────────────────────────────────┘
└────────┬────────┘
         │ WebSocket /ws?token={jwt}
         └─────────────────────────────────────────────────────────┘
```

**Workspace** has 3 crates: `backend`, `frontend` (legacy Dioxus, unused), `shared`. The active frontend is `frontend-react/` (not a Rust crate).

**Backend layers** (`backend/src/`):
- `handlers/` - HTTP endpoint definitions, request validation
- `services/` - Business logic via `Services` container struct (all services injected as `Arc<Services>` via Actix `web::Data`)
- `db/` - Repository pattern, SQLx queries against PostgreSQL
- `websocket.rs` - Connection management with DashMap, broadcast to channels/users
- `middleware.rs` - JWT extraction helpers (`get_user_id_from_request()`)
- `config.rs` - `AppConfig` loaded from env vars

**Frontend** (`frontend-react/src/`):
- `pages/` - Route components (LoginPage, ChatPage, TeamPage, etc.)
- `store/index.ts` - Zustand store (auth, teams, channels, messages, UI state)
- `api/client.ts` - Axios with interceptors for token refresh on 401
- `websocket/client.ts` - WebSocket client with message handlers
- `types/index.ts` - TypeScript interfaces matching shared crate models
- Vite dev server proxies `/api` and `/ws` to backend at `127.0.0.1:8080`
- Production build: `frontend-react/dist/` → copy to `./static/` for backend to serve as SPA

**Shared crate** (`shared/src/`):
- `models.rs` - Domain types (User, Team, Channel, Message, Call, Notification)
- `dto.rs` - Request/response DTOs with `validator` derive macros
- `error.rs` - AppError enum used throughout
- `validation.rs` - Input validation utilities

## Key Patterns

**Adding an endpoint**: Handler in `handlers/*.rs` → Service method in `services/*.rs` → DB query in `db/*.rs` → Register route in `main.rs` under `/api/v1` scope

**Service injection**: All services live in `Services` struct (`services/mod.rs`), constructed with `PgPool` + `AppConfig`, wrapped in `Arc`, and registered as `web::Data`. Handlers access via `services: web::Data<Arc<Services>>`.

**Authentication flow**: JWT access/refresh tokens → Frontend stores in localStorage (`rust_teams_token`, `rust_teams_refresh_token`) → Axios interceptor retries on 401 → Backend middleware extracts user_id via `get_user_id_from_request()`

**Real-time updates**: API call triggers WebSocket broadcast → Frontend receives via `wsClient.on()` → Updates Zustand store

## Environment Variables

Copy `.env.example` to `.env`. Required:
- `DATABASE_URL` - PostgreSQL connection (e.g., `postgresql://postgres:postgres@localhost:5432/rust_teams`)
- `JWT_SECRET` - Secret for signing tokens
- `HOST` / `PORT` - Server bind address (default `127.0.0.1:8080`)

Optional:
- `UPLOAD_DIR` - File upload directory (default: `./uploads`)
- `MAX_FILE_SIZE_MB` - Max upload size (default: 50)
- `RUST_LOG` - Log level (e.g., `info,rust_teams_backend=debug`)
- `CORS_ALLOWED_ORIGINS` - Comma-separated allowed origins
- `WS_HEARTBEAT_INTERVAL_SECS` / `WS_CLIENT_TIMEOUT_SECS` - WebSocket tuning

## Database

PostgreSQL with SQLx. Migrations in `backend/migrations/` run automatically on server start. Manual: `sqlx migrate run` (requires `sqlx-cli`).

## Testing API Manually

Use `api-tests.http` with VS Code REST Client extension.

## Deployment

See `DEPLOYMENT_DEBIAN12.md` for production setup (systemd, Nginx, SSL).
