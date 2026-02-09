import { create } from 'zustand';
import type { User, Team, Channel, Message, Notification, Call, CallParticipant, Meeting } from '../types';

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

  // Call state
  activeCall: Call | null;
  incomingCall: Call | null;
  isLocalAudioEnabled: boolean;
  isLocalVideoEnabled: boolean;
  localStream: MediaStream | null;
  remoteStreams: Record<string, MediaStream>;
  isCallMinimized: boolean;

  // Meeting state
  meetings: Meeting[];
  selectedMeeting: Meeting | null;
  showCreateMeetingModal: boolean;

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
  incrementUnread: (channelId: string) => void;
  clearUnread: (channelId: string) => void;
  setNotifications: (notifications: Notification[]) => void;
  addNotification: (notification: Notification) => void;
  markNotificationAsRead: (notificationId: string) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;

  // Call actions
  setActiveCall: (call: Call | null) => void;
  setIncomingCall: (call: Call | null) => void;
  addCallParticipant: (callId: string, participant: CallParticipant) => void;
  removeCallParticipant: (callId: string, userId: string) => void;
  setLocalAudioEnabled: (enabled: boolean) => void;
  setLocalVideoEnabled: (enabled: boolean) => void;
  setLocalStream: (stream: MediaStream | null) => void;
  addRemoteStream: (peerId: string, stream: MediaStream) => void;
  removeRemoteStream: (peerId: string) => void;
  setCallMinimized: (minimized: boolean) => void;
  resetCallState: () => void;

  // Meeting actions
  setMeetings: (meetings: Meeting[]) => void;
  addMeeting: (meeting: Meeting) => void;
  updateMeeting: (meeting: Meeting) => void;
  removeMeeting: (meetingId: string) => void;
  setSelectedMeeting: (meeting: Meeting | null) => void;
  setShowCreateMeetingModal: (show: boolean) => void;
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

  // Call state
  activeCall: null,
  incomingCall: null,
  isLocalAudioEnabled: true,
  isLocalVideoEnabled: true,
  localStream: null,
  remoteStreams: {},
  isCallMinimized: false,

  // Meeting state
  meetings: [],
  selectedMeeting: null,
  showCreateMeetingModal: false,

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
    set((state) => {
      const existing = state.messages[channelId] || [];
      // Prevent duplicates (idempotent operation)
      if (existing.some((m) => m.id === message.id)) {
        return state;
      }
      return {
        messages: {
          ...state.messages,
          [channelId]: [...existing, message],
        },
      };
    }),

  incrementUnread: (channelId: string) =>
    set((state) => ({
      unreadCounts: {
        ...state.unreadCounts,
        [channelId]: (state.unreadCounts[channelId] || 0) + 1,
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
    set((state) => {
      const existing = state.typingUsers[channelId] || [];
      // Prevent duplicates (idempotent operation)
      if (existing.some((u) => u.id === user.id)) {
        return state;
      }
      return {
        typingUsers: {
          ...state.typingUsers,
          [channelId]: [...existing, user],
        },
      };
    }),

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

  // Call actions
  setActiveCall: (call) => set({ activeCall: call }),
  setIncomingCall: (call) => set({ incomingCall: call }),
  addCallParticipant: (callId, participant) =>
    set((state) => {
      if (!state.activeCall || state.activeCall.id !== callId) return state;
      const exists = state.activeCall.participants.some((p) => p.user.id === participant.user.id);
      if (exists) return state;
      return {
        activeCall: {
          ...state.activeCall,
          participants: [...state.activeCall.participants, participant],
        },
      };
    }),
  removeCallParticipant: (callId, userId) =>
    set((state) => {
      if (!state.activeCall || state.activeCall.id !== callId) return state;
      return {
        activeCall: {
          ...state.activeCall,
          participants: state.activeCall.participants.filter((p) => p.user.id !== userId),
        },
      };
    }),
  setLocalAudioEnabled: (enabled) => set({ isLocalAudioEnabled: enabled }),
  setLocalVideoEnabled: (enabled) => set({ isLocalVideoEnabled: enabled }),
  setLocalStream: (stream) => set({ localStream: stream }),
  addRemoteStream: (peerId, stream) =>
    set((state) => ({
      remoteStreams: { ...state.remoteStreams, [peerId]: stream },
    })),
  removeRemoteStream: (peerId) =>
    set((state) => {
      const newStreams = { ...state.remoteStreams };
      delete newStreams[peerId];
      return { remoteStreams: newStreams };
    }),
  setCallMinimized: (minimized) => set({ isCallMinimized: minimized }),
  resetCallState: () =>
    set({
      activeCall: null,
      incomingCall: null,
      isLocalAudioEnabled: true,
      isLocalVideoEnabled: true,
      localStream: null,
      remoteStreams: {},
      isCallMinimized: false,
    }),

  // Meeting actions
  setMeetings: (meetings) => set({ meetings }),
  addMeeting: (meeting) =>
    set((state) => ({
      meetings: [...state.meetings, meeting].sort(
        (a, b) => new Date(a.start_time).getTime() - new Date(b.start_time).getTime()
      ),
    })),
  updateMeeting: (meeting) =>
    set((state) => ({
      meetings: state.meetings.map((m) => (m.id === meeting.id ? meeting : m)),
    })),
  removeMeeting: (meetingId) =>
    set((state) => ({
      meetings: state.meetings.filter((m) => m.id !== meetingId),
    })),
  setSelectedMeeting: (meeting) => set({ selectedMeeting: meeting }),
  setShowCreateMeetingModal: (show) => set({ showCreateMeetingModal: show }),
}));
