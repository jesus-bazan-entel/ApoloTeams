# Rust Teams Frontend (React + Vite)

This is the React + Vite frontend for the Rust Teams application.

## Prerequisites

- Node.js 18+ 
- npm or yarn

## Installation

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build
```

## Project Structure

```
frontend-react/
├── src/
│   ├── api/
│   │   └── client.ts          # API client with axios
│   ├── pages/
│   │   ├── LoginPage.tsx       # Login page
│   │   ├── RegisterPage.tsx    # Registration page
│   │   ├── HomePage.tsx        # Home/Teams page
│   │   ├── ChatPage.tsx         # Chat interface
│   │   ├── TeamPage.tsx         # Team details page
│   │   └── SettingsPage.tsx    # Settings page
│   ├── store/
│   │   └── index.ts            # Zustand state management
│   ├── types/
│   │   └── index.ts            # TypeScript types
│   ├── websocket/
│   │   └── client.ts          # WebSocket client
│   ├── App.tsx                  # Main App component
│   └── index.css                # Global styles
├── index.html                    # HTML entry point
├── package.json                  # Dependencies
├── tsconfig.json                # TypeScript config
├── tsconfig.node.json            # Node TypeScript config
├── vite.config.ts               # Vite config
└── tailwind.config.js            # Tailwind CSS config
```

## Features

- **Authentication**: Login and registration with JWT tokens
- **Teams**: View and manage your teams
- **Channels**: Create and manage channels within teams
- **Chat**: Real-time messaging with WebSocket
- **Settings**: Update profile and change password
- **State Management**: Zustand for global state
- **Styling**: Tailwind CSS for utility-first styling
- **TypeScript**: Full type safety

## API Integration

The frontend communicates with the Rust backend via REST API:

- Authentication (`/api/v1/auth/*`)
- Users (`/api/v1/users/*`)
- Teams (`/api/v1/teams/*`)
- Channels (`/api/v1/channels/*`)
- Messages (`/api/v1/channels/:id/messages`)
- Calls (`/api/v1/calls/*`)
- Notifications (`/api/v1/notifications/*`)

## WebSocket

Real-time updates are handled via WebSocket connection to `/ws`. The WebSocket client supports:

- Authentication
- Message sending/receiving
- Typing indicators
- User status updates
- Call signaling (WebRTC)

## Development

The development server runs on port 3000 and proxies API requests to the backend on port 8080.

## Building

To build for production:

```bash
npm run build
```

The built files will be in the `dist` directory.
