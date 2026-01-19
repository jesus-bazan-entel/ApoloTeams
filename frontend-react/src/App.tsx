import { useEffect } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { useStore } from './store';
import { apiClient } from './api/client';
import { wsClient } from './websocket/client';
import LoginPage from './pages/LoginPage';
import RegisterPage from './pages/RegisterPage';
import HomePage from './pages/HomePage';
import ChatPage from './pages/ChatPage';
import TeamPage from './pages/TeamPage';
import SettingsPage from './pages/SettingsPage';

function App() {
  const { isAuthenticated, accessToken, setAuth, logout } = useStore();

  useEffect(() => {
    // Check for existing auth on mount
    const token = localStorage.getItem('rust_teams_token');
    const refreshToken = localStorage.getItem('rust_teams_refresh_token');
    if (token && refreshToken) {
      // Validate token by fetching current user
      apiClient.getCurrentUser()
        .then((user) => {
          setAuth(user, token, refreshToken);
        })
        .catch(() => {
          logout();
        });
    }
  }, []);

  useEffect(() => {
    // Connect WebSocket if authenticated
    if (isAuthenticated && accessToken) {
      wsClient.connect(accessToken);

      // Set up WebSocket message handlers
      wsClient.on('Authenticated', (payload) => {
        console.log('WebSocket authenticated:', payload);
      });

      wsClient.on('NewMessage', (payload) => {
        console.log('New message:', payload);
      });

      wsClient.on('MessageUpdated', (payload) => {
        console.log('Message updated:', payload);
      });

      wsClient.on('MessageDeleted', (payload) => {
        console.log('Message deleted:', payload);
      });

      wsClient.on('UserTyping', (payload) => {
        console.log('User typing:', payload);
      });

      wsClient.on('UserStoppedTyping', (payload) => {
        console.log('User stopped typing:', payload);
      });

      wsClient.on('UserStatusChanged', (payload) => {
        console.log('User status changed:', payload);
      });

      wsClient.on('UserJoinedChannel', (payload) => {
        console.log('User joined channel:', payload);
      });

      wsClient.on('UserLeftChannel', (payload) => {
        console.log('User left channel:', payload);
      });

      wsClient.on('Notification', (payload) => {
        console.log('New notification:', payload);
      });

      wsClient.on('Error', (payload) => {
        console.error('WebSocket error:', payload);
      });

      return () => {
        wsClient.disconnect();
      };
    }
  }, [isAuthenticated, accessToken]);

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={isAuthenticated ? <HomePage /> : <Navigate to="/login" />} />
        <Route path="/login" element={isAuthenticated ? <Navigate to="/" /> : <LoginPage />} />
        <Route path="/register" element={isAuthenticated ? <Navigate to="/" /> : <RegisterPage />} />
        <Route path="/chat" element={isAuthenticated ? <ChatPage /> : <Navigate to="/login" />} />
        <Route path="/chat/:channelId" element={isAuthenticated ? <ChatPage /> : <Navigate to="/login" />} />
        <Route path="/teams/:teamId" element={isAuthenticated ? <TeamPage /> : <Navigate to="/login" />} />
        <Route path="/settings" element={isAuthenticated ? <SettingsPage /> : <Navigate to="/login" />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
