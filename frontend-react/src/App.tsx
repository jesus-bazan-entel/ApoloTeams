import { useEffect } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { useStore } from './store';
import { apiClient } from './api/client';
import { useWebSocketSetup } from './hooks';
import { VideoCallModal, IncomingCallModal, CallMinimized } from './components/call';
import LoginPage from './pages/LoginPage';
import RegisterPage from './pages/RegisterPage';
import MainLayout from './pages/MainLayout';
import HomePage from './pages/HomePage';
import ChatPage from './pages/ChatPage';
import TeamPage from './pages/TeamPage';
import SettingsPage from './pages/SettingsPage';
import { CalendarPage } from './pages/CalendarPage';

function App() {
  const { isAuthenticated, isCallMinimized, setAuth, logout } = useStore();

  // Initialize WebSocket connection and handlers
  useWebSocketSetup();

  useEffect(() => {
    // Check for existing auth on mount
    const token = localStorage.getItem('rust_teams_token');
    const refreshToken = localStorage.getItem('rust_teams_refresh_token');
    if (token && refreshToken) {
      // Validate token by fetching current user
      // Note: if the token is expired, the axios interceptor will refresh it
      // and save the new token to localStorage before this .then() runs
      apiClient.getCurrentUser()
        .then((user) => {
          // Re-read tokens from localStorage since the interceptor may have refreshed them
          const freshToken = localStorage.getItem('rust_teams_token') || token;
          const freshRefreshToken = localStorage.getItem('rust_teams_refresh_token') || refreshToken;
          setAuth(user, freshToken, freshRefreshToken);
        })
        .catch(() => {
          logout();
        });
    }
  }, []);

  return (
    <BrowserRouter>
      <Routes>
        {/* Public routes */}
        <Route path="/login" element={isAuthenticated ? <Navigate to="/" /> : <LoginPage />} />
        <Route path="/register" element={isAuthenticated ? <Navigate to="/" /> : <RegisterPage />} />

        {/* Authenticated routes with shared sidebar layout */}
        <Route element={isAuthenticated ? <MainLayout /> : <Navigate to="/login" />}>
          <Route path="/" element={<HomePage />} />
          <Route path="/chat" element={<ChatPage />} />
          <Route path="/chat/:channelId" element={<ChatPage />} />
          <Route path="/teams/:teamId" element={<TeamPage />} />
          <Route path="/calendar" element={<CalendarPage />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Route>
      </Routes>

      {/* Global call modals - shown on all pages when authenticated */}
      {isAuthenticated && (
        <>
          <VideoCallModal />
          <IncomingCallModal />
          {isCallMinimized && <CallMinimized />}
        </>
      )}
    </BrowserRouter>
  );
}

export default App;
