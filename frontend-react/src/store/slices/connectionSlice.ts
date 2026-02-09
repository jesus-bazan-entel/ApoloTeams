/**
 * Connection Slice - Handles WebSocket connection state
 *
 * Tracks connection status, subscribed channels, and online users
 */

import type { StateCreator } from 'zustand';
import type { UserStatus } from '../../types';

export interface ConnectionState {
  // State
  isConnected: boolean;
  isAuthenticated: boolean;
  subscribedChannels: Set<string>;
  onlineUsers: Record<string, UserStatus>;
  connectionError: string | null;

  // Actions
  setConnected: (connected: boolean) => void;
  setWsAuthenticated: (authenticated: boolean) => void;
  subscribeToChannel: (channelId: string) => void;
  unsubscribeFromChannel: (channelId: string) => void;
  setUserStatus: (userId: string, status: UserStatus) => void;
  setConnectionError: (error: string | null) => void;
  resetConnectionState: () => void;
}

export const createConnectionSlice: StateCreator<ConnectionState> = (set) => ({
  // Initial state
  isConnected: false,
  isAuthenticated: false,
  subscribedChannels: new Set(),
  onlineUsers: {},
  connectionError: null,

  // Actions
  setConnected: (connected) => set({ isConnected: connected }),

  setWsAuthenticated: (authenticated) => set({ isAuthenticated: authenticated }),

  subscribeToChannel: (channelId) =>
    set((state) => ({
      subscribedChannels: new Set([...state.subscribedChannels, channelId]),
    })),

  unsubscribeFromChannel: (channelId) =>
    set((state) => {
      const newSet = new Set(state.subscribedChannels);
      newSet.delete(channelId);
      return { subscribedChannels: newSet };
    }),

  setUserStatus: (userId, status) =>
    set((state) => ({
      onlineUsers: { ...state.onlineUsers, [userId]: status },
    })),

  setConnectionError: (error) => set({ connectionError: error }),

  resetConnectionState: () =>
    set({
      isConnected: false,
      isAuthenticated: false,
      subscribedChannels: new Set(),
      connectionError: null,
    }),
});
