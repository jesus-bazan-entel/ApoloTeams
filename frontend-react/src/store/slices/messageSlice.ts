/**
 * Message Slice - Handles all message-related state
 *
 * Follows Single Responsibility Principle:
 * - Only manages message state and actions
 * - Decoupled from WebSocket implementation
 */

import type { StateCreator } from 'zustand';
import type { Message, User } from '../../types';

export interface MessageState {
  // State
  messages: Record<string, Message[]>;
  typingUsers: Record<string, User[]>;
  unreadCounts: Record<string, number>;

  // Actions
  setMessages: (channelId: string, messages: Message[]) => void;
  addMessage: (channelId: string, message: Message) => void;
  updateMessage: (channelId: string, message: Message) => void;
  deleteMessage: (channelId: string, messageId: string) => void;
  addTypingUser: (channelId: string, user: User) => void;
  removeTypingUser: (channelId: string, userId: string) => void;
  setUnreadCount: (channelId: string, count: number) => void;
  incrementUnread: (channelId: string) => void;
  clearUnread: (channelId: string) => void;
  clearChannelMessages: (channelId: string) => void;
}

export const createMessageSlice: StateCreator<MessageState> = (set) => ({
  // Initial state
  messages: {},
  typingUsers: {},
  unreadCounts: {},

  // Actions
  setMessages: (channelId, messages) =>
    set((state) => ({
      messages: { ...state.messages, [channelId]: messages },
    })),

  addMessage: (channelId, message) =>
    set((state) => {
      const existing = state.messages[channelId] || [];
      // Prevent duplicates (idempotent)
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
        [channelId]: (state.messages[channelId] || []).filter(
          (m) => m.id !== messageId
        ),
      },
    })),

  addTypingUser: (channelId, user) =>
    set((state) => {
      const existing = state.typingUsers[channelId] || [];
      // Prevent duplicates
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
        [channelId]: (state.typingUsers[channelId] || []).filter(
          (u) => u.id !== userId
        ),
      },
    })),

  setUnreadCount: (channelId, count) =>
    set((state) => ({
      unreadCounts: { ...state.unreadCounts, [channelId]: count },
    })),

  incrementUnread: (channelId) =>
    set((state) => ({
      unreadCounts: {
        ...state.unreadCounts,
        [channelId]: (state.unreadCounts[channelId] || 0) + 1,
      },
    })),

  clearUnread: (channelId) =>
    set((state) => {
      const newUnreadCounts = { ...state.unreadCounts };
      delete newUnreadCounts[channelId];
      return { unreadCounts: newUnreadCounts };
    }),

  clearChannelMessages: (channelId) =>
    set((state) => {
      const { [channelId]: _, ...rest } = state.messages;
      return { messages: rest };
    }),
});
