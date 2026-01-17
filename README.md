# Rust Teams 🦀

A blazingly fast, secure, and modern Microsoft Teams clone built entirely in Rust. This project demonstrates a full-stack Rust application with real-time messaging, video/audio calls, file sharing, and team collaboration features.

## 🌟 Features

### Core Features
- **Real-time Messaging** - Instant messaging with WebSocket support
- **Teams & Channels** - Organize conversations into teams and channels
- **Direct Messages** - Private one-on-one conversations
- **File Sharing** - Upload and share files with your team
- **User Presence** - See who's online, away, or busy
- **Notifications** - Real-time notifications for messages and mentions

### Advanced Features
- **Video & Audio Calls** - WebRTC-powered video conferencing
- **Message Reactions** - React to messages with emojis
- **Message Threading** - Reply to specific messages
- **Search** - Full-text search across messages, files, and users
- **User Profiles** - Customizable user profiles with avatars

### Security
- **JWT Authentication** - Secure token-based authentication
- **Password Hashing** - Argon2 password hashing
- **CORS Protection** - Configurable CORS policies
- **Input Validation** - Comprehensive input validation

## 🏗️ Architecture

This project uses a Cargo workspace with three crates:

```
rust-teams/
├── backend/          # Actix-web REST API server
├── frontend/         # Dioxus web application
├── shared/           # Shared types and utilities
├── Cargo.toml        # Workspace configuration
└── README.md
```

### Technology Stack

| Layer | Technology |
|-------|------------|
| Backend Framework | [Actix-web](https://actix.rs/) |
| Frontend Framework | [Dioxus](https://dioxuslabs.com/) |
| Database | SQLite with [SQLx](https://github.com/launchbadge/sqlx) |
| Authentication | JWT with [jsonwebtoken](https://github.com/Keats/jsonwebtoken) |
| Password Hashing | [Argon2](https://github.com/RustCrypto/password-hashes) |
| Real-time | WebSockets (native Actix-web) |
| Video Calls | WebRTC signaling |
| Serialization | [Serde](https://serde.rs/) |

## 🚀 Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (1.70 or later)
- [Trunk](https://trunkrs.dev/) (for frontend development)
- SQLite

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/rust-teams.git
   cd rust-teams
   ```

2. **Set up environment variables**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Create the database**
   ```bash
   # The database will be created automatically on first run
   # Or manually create it:
   mkdir -p data
   touch data/rust_teams.db
   ```

4. **Run database migrations**
   ```bash
   cd backend
   sqlx database create
   sqlx migrate run
   cd ..
   ```

### Running the Application

#### Development Mode

**Backend:**
```bash
cd backend
cargo run
```
The backend server will start at `http://localhost:8080`

**Frontend:**
```bash
cd frontend
trunk serve
```
The frontend will be available at `http://localhost:8081`

#### Production Build

**Backend:**
```bash
cd backend
cargo build --release
./target/release/rust-teams-backend
```

**Frontend:**
```bash
cd frontend
trunk build --release
# Serve the dist/ directory with your preferred web server
```

## 📚 API Documentation

### Authentication

#### Register
```http
POST /api/auth/register
Content-Type: application/json

{
  "username": "johndoe",
  "email": "john@example.com",
  "password": "securepassword123",
  "display_name": "John Doe"
}
```

#### Login
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "john@example.com",
  "password": "securepassword123"
}
```

### Teams

#### Create Team
```http
POST /api/teams
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Engineering",
  "description": "Engineering team discussions"
}
```

#### Get Teams
```http
GET /api/teams
Authorization: Bearer <token>
```

### Channels

#### Create Channel
```http
POST /api/teams/{team_id}/channels
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "general",
  "description": "General discussions",
  "is_private": false
}
```

### Messages

#### Send Message
```http
POST /api/channels/{channel_id}/messages
Authorization: Bearer <token>
Content-Type: application/json

{
  "content": "Hello, team!"
}
```

#### Get Messages
```http
GET /api/channels/{channel_id}/messages?limit=50&before={message_id}
Authorization: Bearer <token>
```

### WebSocket

Connect to `/ws` with a valid JWT token for real-time updates:

```javascript
const ws = new WebSocket('ws://localhost:8080/ws?token=<jwt_token>');

// Message types:
// - message: New message received
// - typing: User is typing
// - presence: User presence update
// - call: Call signaling
```

## 🧪 Testing

```bash
# Run all tests
cargo test --workspace

# Run backend tests
cargo test -p rust-teams-backend

# Run frontend tests
cargo test -p rust-teams-frontend

# Run with coverage
cargo tarpaulin --workspace
```

## 📁 Project Structure

### Backend (`backend/`)

```
backend/
├── src/
│   ├── main.rs           # Application entry point
│   ├── config.rs         # Configuration management
│   ├── error.rs          # Error handling
│   ├── middleware.rs     # Authentication middleware
│   ├── websocket.rs      # WebSocket server
│   ├── db/               # Database layer
│   │   ├── mod.rs
│   │   ├── users.rs
│   │   ├── teams.rs
│   │   ├── channels.rs
│   │   ├── messages.rs
│   │   ├── files.rs
│   │   ├── calls.rs
│   │   └── notifications.rs
│   ├── services/         # Business logic
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── users.rs
│   │   ├── teams.rs
│   │   ├── channels.rs
│   │   ├── messages.rs
│   │   ├── files.rs
│   │   ├── calls.rs
│   │   └── notifications.rs
│   └── handlers/         # HTTP handlers
│       ├── mod.rs
│       ├── health.rs
│       ├── auth.rs
│       ├── users.rs
│       ├── teams.rs
│       ├── channels.rs
│       ├── messages.rs
│       ├── files.rs
│       ├── calls.rs
│       ├── search.rs
│       └── notifications.rs
├── migrations/           # SQL migrations
└── Cargo.toml
```

### Frontend (`frontend/`)

```
frontend/
├── src/
│   ├── main.rs           # Application entry point
│   ├── state.rs          # Global state management
│   ├── api.rs            # API client
│   ├── websocket.rs      # WebSocket client
│   ├── components/       # Reusable UI components
│   │   ├── mod.rs
│   │   ├── avatar.rs
│   │   ├── button.rs
│   │   ├── input.rs
│   │   ├── message.rs
│   │   ├── message_input.rs
│   │   ├── modal.rs
│   │   ├── sidebar.rs
│   │   └── user_status.rs
│   └── pages/            # Page components
│       ├── mod.rs
│       ├── auth.rs
│       ├── chat.rs
│       ├── home.rs
│       ├── settings.rs
│       └── team.rs
├── index.html
└── Cargo.toml
```

### Shared (`shared/`)

```
shared/
├── src/
│   ├── lib.rs            # Module exports
│   ├── models.rs         # Domain models
│   ├── dto.rs            # Data transfer objects
│   ├── error.rs          # Error types
│   └── validation.rs     # Validation utilities
└── Cargo.toml
```

## 🔧 Configuration

Configuration is managed through environment variables. See `.env.example` for all available options.

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | Server host | `127.0.0.1` |
| `PORT` | Server port | `8080` |
| `DATABASE_URL` | SQLite database path | `sqlite:./data/rust_teams.db` |
| `JWT_SECRET` | JWT signing secret | Required |
| `JWT_EXPIRATION_HOURS` | Token expiration | `24` |
| `UPLOAD_DIR` | File upload directory | `./uploads` |
| `MAX_FILE_SIZE_MB` | Max upload size | `50` |

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Actix-web](https://actix.rs/) - Powerful, pragmatic, and extremely fast web framework for Rust
- [Dioxus](https://dioxuslabs.com/) - Fullstack GUI library for desktop, web, mobile, and more
- [SQLx](https://github.com/launchbadge/sqlx) - Async SQL toolkit for Rust
- [Microsoft Teams](https://www.microsoft.com/en-us/microsoft-teams/group-chat-software) - Inspiration for this project

## 📊 Roadmap

- [ ] End-to-end encryption for messages
- [ ] Screen sharing in video calls
- [ ] Message scheduling
- [ ] Bot/integration support
- [ ] Mobile apps (iOS/Android) using Dioxus
- [ ] Desktop apps using Dioxus
- [ ] Plugin system
- [ ] Advanced admin panel
- [ ] Analytics dashboard

---

Built with ❤️ and 🦀 Rust
