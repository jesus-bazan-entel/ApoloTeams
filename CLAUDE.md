# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ApoloTeams (also called Rust Teams) is a Microsoft Teams clone with a Rust backend and React frontend. Real-time messaging via WebSockets, JWT authentication, file sharing, and video/audio calls.

## Build & Run Commands

```bash
# Backend (Rust/Actix-web)
cargo run -p backend                    # Dev server at http://localhost:8080
cargo build --release -p backend        # Production build

# Frontend (React/Vite)
cd frontend-react
npm install                             # Install dependencies (first time)
npm run dev                             # Dev server at http://localhost:5173
npm run build                           # Production build (runs tsc first)
npm run lint                            # ESLint check
npm run type-check                      # TypeScript check only

# Full workspace
cargo test --workspace                  # Run all Rust tests
cargo test -p backend                   # Backend tests only
cargo fmt --all                         # Format Rust code
cargo clippy --all                      # Lint Rust code
```

## Architecture

```
┌─────────────────┐     ┌──────────────────────────────────────────┐
│  React Frontend │────▶│  Actix-web Backend                       │
│  (Vite + Zustand)     │  Handlers → Services → DB (Repository)   │
│                 │◀────│                                          │
└────────┬────────┘     └──────────────────────────────────────────┘
         │ WebSocket           │
         └─────────────────────┘
```

**Backend layers** (in `backend/src/`):
- `handlers/` - HTTP endpoint definitions, request validation
- `services/` - Business logic, orchestrates DB calls
- `db/` - Repository pattern, SQLx queries against PostgreSQL
- `websocket.rs` - Connection management with DashMap, broadcast to channels/users

**Frontend organization** (in `frontend-react/src/`):
- `pages/` - Route components (LoginPage, ChatPage, TeamPage, etc.)
- `store/index.ts` - Zustand store (auth, teams, channels, messages, UI state)
- `api/client.ts` - Axios with interceptors for token refresh on 401
- `websocket/client.ts` - WebSocket client with message handlers

**Shared crate** (`shared/src/`):
- `models.rs` - Domain types (User, Team, Channel, Message)
- `dto.rs` - Request/response DTOs
- `error.rs` - AppError enum used throughout

## Key Patterns

**Adding an endpoint**: Handler in `handlers/*.rs` → Service method in `services/*.rs` → DB query in `db/*.rs` → Register route in `main.rs`

**Authentication flow**: JWT access/refresh tokens → Frontend stores in localStorage → Axios interceptor retries on 401 → Middleware extracts user_id via `get_user_id_from_request()`

**Real-time updates**: API call triggers WebSocket broadcast → Frontend receives via `wsClient.on()` → Updates Zustand store

## Environment Variables

Copy `.env.example` to `.env`. Key variables:
- `DATABASE_URL` - PostgreSQL connection string
- `JWT_SECRET` - Secret for signing tokens
- `UPLOAD_DIR` - File upload directory
- `RUST_LOG` - Log level (e.g., `info,backend=debug`)

## API Base Path

All REST endpoints under `/api/v1/`. WebSocket at `/ws?token={jwt}`.

## Database

PostgreSQL with SQLx. Migrations in `backend/migrations/`. Run with `sqlx migrate run`.

## Testing API Manually

Use `api-tests.http` with VS Code REST Client extension.

## Deployment

See `DEPLOYMENT_DEBIAN12.md` for full production setup (systemd, Nginx, SSL).
