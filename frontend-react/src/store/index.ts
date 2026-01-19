import { create } from 'zustand';
import type { User, Team, Channel, Message, Notification } from '../types';

interface AppState {
  // Auth
  isAuthenticated: boolean;
  currentUser: User | null;
  accessToken: string | null;
  refreshToken: string | null;

  // Data
  teams: Team[];
  channels: Channel[];
  selectedTeamId: string | null;
  selectedChannelId: string | null;
  messages: Record<string, Message[]>;
  onlineUsers: Record<string, 'online' | 'away' | 'busy' | 'offline'>;
  typingUsers: Record<string, User[]>;
  unreadCounts: Record<string, number>;
  notifications: Notification[];

  // UI
  loading: boolean;
  error: string | null;

  // Actions
  setAuth: (user: User, accessToken: string, refreshToken: string) => void;
  logout: () => void;
  setTeams: (teams: Team[]) => void;
  setChannels: (channels: Channel[]) => void;
  setSelectedTeam: (teamId: string | null) => void;
  setSelectedChannel: (channelId: string | null) => void;
  setMessages: (channelId: string, messages: Message[]) => void;
  addMessage: (channelId: string, message: Message) => void;
  updateMessage: (channelId: string, message: Message) => void;
  deleteMessage: (channelId: string, messageId: string) => void;
  setUserStatus: (userId: string, status: 'online' | 'away' | 'busy' | 'offline') => void;
  addTypingUser: (channelId: string, user: User) => void;
  removeTypingUser: (channelId: string, userId: string) => void;
  setUnreadCount: (channelId: string, count: number) => void;
  clearUnread: (channelId: string) => void;
  setNotifications: (notifications: Notification[]) => void;
  addNotification: (notification: Notification) => void;
  markNotificationAsRead: (notificationId: string) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

export const useStore = create<AppState>((set) => ({
  // Auth
  isAuthenticated: false,
  currentUser: null,
  accessToken: null,
  refreshToken: null,

  // Data
  teams: [],
  channels: [],
  selectedTeamId: null,
  selectedChannelId: null,
  messages: {},
  onlineUsers: {},
  typingUsers: {},
  unreadCounts: {},
  notifications: [],

  // UI
  loading: false,
  error: null,

  // Actions
  setAuth: (user, accessToken, refreshToken) => {
    localStorage.setItem('rust_teams_token', accessToken);
    localStorage.setItem('rust_teams_refresh_token', refreshToken);
    set({
      isAuthenticated: true,
      currentUser: user,
      accessToken,
      refreshToken,
    });
  },

  logout: () => {
    localStorage.removeItem('rust_teams_token');
    localStorage.removeItem('rust_teams_refresh_token');
    set({
      isAuthenticated: false,
      currentUser: null,
      accessToken: null,
      refreshToken: null,
      teams: [],
      channels: [],
      selectedTeamId: null,
      selectedChannelId: null,
      messages: {},
      onlineUsers: {},
      typingUsers: {},
      unreadCounts: {},
      notifications: [],
    });
  },

  setTeams: (teams) => set({ teams }),
  setChannels: (channels) => set({ channels }),
  setSelectedTeam: (teamId) => set({ selectedTeamId: teamId }),
  setSelectedChannel: (channelId) => set({ selectedChannelId: channelId }),

  setMessages: (channelId, messages) =>
    set((state) => ({
      messages: { ...state.messages, [channelId]: messages },
    })),

  addMessage: (channelId, message) =>
    set((state) => ({
      messages: {
        ...state.messages,
        [channelId]: [...(state.messages[channelId] || []), message],
      },
    })),

  updateMessage: (channelId, message) =>
    set((state) => ({
      messages: {
        ...state.messages,
        [channelId]: (state.messages[channelId] || []).map((m) =>
          m.id === message.id ? message : m
        ),
      },
    })),

  deleteMessage: (channelId, messageId) =>
    set((state) => ({
      messages: {
        ...state.messages,
        [channelId]: (state.messages[channelId] || []).filter((m) => m.id !== messageId),
      },
    })),

  setUserStatus: (userId, status) =>
    set((state) => ({
      onlineUsers: { ...state.onlineUsers, [userId]: status },
    })),

  addTypingUser: (channelId, user) =>
    set((state) => ({
      typingUsers: {
        ...state.typingUsers,
        [channelId]: [...(state.typingUsers[channelId] || []), user],
      },
    })),

  removeTypingUser: (channelId, userId) =>
    set((state) => ({
      typingUsers: {
        ...state.typingUsers,
        [channelId]: (state.typingUsers[channelId] || []).filter((u) => u.id !== userId),
      },
    })),

  setUnreadCount: (channelId, count) =>
    set((state) => ({
      unreadCounts: { ...state.unreadCounts, [channelId]: count },
    })),

  clearUnread: (channelId) =>
    set((state) => {
      const newUnreadCounts = { ...state.unreadCounts };
      delete newUnreadCounts[channelId];
      return { unreadCounts: newUnreadCounts };
    }),

  setNotifications: (notifications) => set({ notifications }),
  addNotification: (notification) =>
    set((state) => ({
      notifications: [notification, ...state.notifications],
    })),
  markNotificationAsRead: (notificationId) =>
    set((state) => ({
      notifications: state.notifications.map((n) =>
        n.id === notificationId ? { ...n, read: true } : n
      ),
    })),
  setLoading: (loading) => set({ loading }),
  setError: (error) => set({ error }),
}));
