/**
 * useChannel Hook
 *
 * Encapsulates all channel-related logic:
 * - Message loading and caching
 * - WebSocket subscription lifecycle
 * - Typing indicators
 * - Optimistic updates
 */

import { useEffect, useCallback, useRef, useState } from 'react';
import { useStore } from '../store';
import { apiClient } from '../api/client';
import { wsClient } from '../websocket/client';
import type { Message, SendMessageRequest } from '../types';

interface UseChannelOptions {
  /** Auto-load messages on mount */
  autoLoad?: boolean;
  /** Number of messages to load initially */
  initialLimit?: number;
}

interface UseChannelReturn {
  messages: Message[];
  loading: boolean;
  error: string | null;
  hasMore: boolean;
  typingUsers: string[];
  sendMessage: (content: string, options?: Partial<SendMessageRequest>) => Promise<Message>;
  loadMessages: () => Promise<void>;
  loadMoreMessages: () => Promise<void>;
  startTyping: () => void;
  stopTyping: () => void;
  markAsRead: () => Promise<void>;
}

export function useChannel(
  channelId: string | undefined,
  options: UseChannelOptions = {}
): UseChannelReturn {
  const { autoLoad = true, initialLimit = 50 } = options;

  const {
    messages: allMessages,
    typingUsers: allTypingUsers,
    currentUser,
    setMessages,
    addMessage,
    clearUnread,
  } = useStore();

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasMore, setHasMore] = useState(true);

  // Refs for cleanup and debouncing
  const typingTimeoutRef = useRef<ReturnType<typeof setTimeout>>();
  const isTypingRef = useRef(false);

  // Get messages for this channel
  const messages = channelId ? allMessages[channelId] || [] : [];
  const typingUsers = channelId
    ? (allTypingUsers[channelId] || [])
        .filter((u) => u.id !== currentUser?.id)
        .map((u) => u.display_name)
    : [];

  // Load messages from API
  const loadMessages = useCallback(async () => {
    if (!channelId) return;

    setLoading(true);
    setError(null);

    try {
      const msgs = await apiClient.listMessages(channelId, initialLimit);
      setMessages(channelId, msgs);
      setHasMore(msgs.length === initialLimit);
    } catch (err: any) {
      setError(err.message || 'Failed to load messages');
    } finally {
      setLoading(false);
    }
  }, [channelId, initialLimit, setMessages]);

  // Load older messages (pagination)
  const loadMoreMessages = useCallback(async () => {
    if (!channelId || !hasMore || loading) return;

    const oldestMessage = messages[0];
    if (!oldestMessage) return;

    setLoading(true);
    try {
      const olderMsgs = await apiClient.listMessages(channelId, initialLimit);
      // Prepend older messages
      setMessages(channelId, [...olderMsgs, ...messages]);
      setHasMore(olderMsgs.length === initialLimit);
    } catch (err: any) {
      setError(err.message || 'Failed to load more messages');
    } finally {
      setLoading(false);
    }
  }, [channelId, hasMore, loading, messages, initialLimit, setMessages]);

  // Send message with optimistic update
  const sendMessage = useCallback(
    async (content: string, options?: Partial<SendMessageRequest>): Promise<Message> => {
      if (!channelId) throw new Error('No channel selected');

      // Stop typing indicator
      if (isTypingRef.current) {
        wsClient.send({ type: 'StopTyping', payload: { channel_id: channelId } });
        isTypingRef.current = false;
      }

      // Create optimistic message
      const optimisticId = `temp-${Date.now()}`;
      const optimisticMessage: Message = {
        id: optimisticId,
        channel_id: channelId,
        sender: currentUser!,
        content,
        message_type: options?.message_type || 'text',
        reactions: [],
        attachments: [],
        edited: false,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };

      // Add optimistically
      addMessage(channelId, optimisticMessage);

      try {
        // Send to server
        const message = await apiClient.sendMessage(channelId, {
          content,
          ...options,
        });

        // Replace optimistic message with real one
        useStore.setState((state) => ({
          messages: {
            ...state.messages,
            [channelId]: state.messages[channelId].map((m) =>
              m.id === optimisticId ? message : m
            ),
          },
        }));

        return message;
      } catch (err) {
        // Remove optimistic message on failure
        useStore.setState((state) => ({
          messages: {
            ...state.messages,
            [channelId]: state.messages[channelId].filter((m) => m.id !== optimisticId),
          },
        }));
        throw err;
      }
    },
    [channelId, currentUser, addMessage]
  );

  // Typing indicators with debouncing
  const startTyping = useCallback(() => {
    if (!channelId || isTypingRef.current) return;

    isTypingRef.current = true;
    wsClient.send({ type: 'StartTyping', payload: { channel_id: channelId } });

    // Auto-stop after 3 seconds of inactivity
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
    }
    typingTimeoutRef.current = setTimeout(() => {
      if (channelId && isTypingRef.current) {
        wsClient.send({ type: 'StopTyping', payload: { channel_id: channelId } });
        isTypingRef.current = false;
      }
    }, 3000);
  }, [channelId]);

  const stopTyping = useCallback(() => {
    if (!channelId || !isTypingRef.current) return;

    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
    }
    wsClient.send({ type: 'StopTyping', payload: { channel_id: channelId } });
    isTypingRef.current = false;
  }, [channelId]);

  // Mark channel as read
  const markAsRead = useCallback(async () => {
    if (!channelId) return;
    try {
      await apiClient.markChannelAsRead(channelId);
      clearUnread(channelId);
    } catch (err) {
      console.error('Failed to mark as read:', err);
    }
  }, [channelId, clearUnread]);

  // Track if we've already loaded messages for this channel
  const hasLoadedRef = useRef<string | null>(null);

  // Subscribe to channel on mount
  useEffect(() => {
    if (!channelId) return;

    // Join channel for real-time updates
    wsClient.send({ type: 'JoinChannel', payload: { channel_id: channelId } });

    // Load messages only once per channel
    if (autoLoad && hasLoadedRef.current !== channelId) {
      hasLoadedRef.current = channelId;
      loadMessages();
    }

    // Mark as read
    markAsRead();

    // Cleanup
    return () => {
      // Stop typing if active
      if (isTypingRef.current) {
        wsClient.send({ type: 'StopTyping', payload: { channel_id: channelId } });
        isTypingRef.current = false;
      }
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
      }

      // Leave channel
      wsClient.send({ type: 'LeaveChannel', payload: { channel_id: channelId } });
    };
  }, [channelId]); // Only re-run when channelId changes

  return {
    messages,
    loading,
    error,
    hasMore,
    typingUsers,
    sendMessage,
    loadMessages,
    loadMoreMessages,
    startTyping,
    stopTyping,
    markAsRead,
  };
}
