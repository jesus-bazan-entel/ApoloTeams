/**
 * WebSocket Manager Hook
 *
 * Centralized WebSocket connection management following best practices:
 * - Single source of truth for WS connection
 * - Automatic reconnection with exponential backoff
 * - Type-safe message handling
 * - Clean separation from UI components
 */

import { useEffect, useCallback, useRef } from 'react';
import { useStore } from '../store';
import { wsClient } from '../websocket/client';
import type { Message, User, Call, Meeting, Notification, UserStatus } from '../types';

interface WebSocketHandlers {
  onMessage?: (channelId: string, message: Message) => void;
  onMessageUpdated?: (channelId: string, message: Message) => void;
  onMessageDeleted?: (channelId: string, messageId: string) => void;
  onTypingStart?: (channelId: string, user: User) => void;
  onTypingStop?: (channelId: string, userId: string) => void;
  onUserStatusChange?: (userId: string, status: UserStatus) => void;
  onCallStarted?: (call: Call) => void;
  onCallEnded?: (callId: string) => void;
  onMeetingInvite?: (meeting: Meeting) => void;
  onNotification?: (notification: Notification) => void;
}

export function useWebSocketManager(handlers?: WebSocketHandlers) {
  const {
    isAuthenticated,
    accessToken,
    // Message actions
    addMessage,
    updateMessage,
    deleteMessage,
    addTypingUser,
    removeTypingUser,
    // User actions
    setUserStatus,
    // Call actions
    setIncomingCall,
    resetCallState,
    // Meeting actions
    addMeeting,
    updateMeeting,
    removeMeeting,
    // Notification actions
    addNotification,
  } = useStore();

  const handlersRef = useRef(handlers);
  handlersRef.current = handlers;

  // Setup WebSocket handlers
  const setupHandlers = useCallback(() => {
    // Message handlers
    wsClient.on('NewMessage', (payload) => {
      const { message } = payload;
      addMessage(message.channel_id, message);

      // Increment unread if not in the channel
      const currentChannel = useStore.getState().selectedChannelId;
      if (message.channel_id !== currentChannel) {
        useStore.getState().incrementUnread?.(message.channel_id);
      }

      handlersRef.current?.onMessage?.(message.channel_id, message);
    });

    wsClient.on('MessageUpdated', (payload) => {
      const { message } = payload;
      updateMessage(message.channel_id, message);
      handlersRef.current?.onMessageUpdated?.(message.channel_id, message);
    });

    wsClient.on('MessageDeleted', (payload) => {
      deleteMessage(payload.channel_id, payload.message_id);
      handlersRef.current?.onMessageDeleted?.(payload.channel_id, payload.message_id);
    });

    // Typing indicators
    wsClient.on('UserTyping', (payload) => {
      addTypingUser(payload.channel_id, payload.user);
      handlersRef.current?.onTypingStart?.(payload.channel_id, payload.user);

      // Auto-remove typing after 3 seconds (safety net)
      setTimeout(() => {
        removeTypingUser(payload.channel_id, payload.user.id);
      }, 3000);
    });

    wsClient.on('UserStoppedTyping', (payload) => {
      removeTypingUser(payload.channel_id, payload.user_id);
      handlersRef.current?.onTypingStop?.(payload.channel_id, payload.user_id);
    });

    // User status
    wsClient.on('UserStatusChanged', (payload) => {
      setUserStatus(payload.user_id, payload.status);
      handlersRef.current?.onUserStatusChange?.(payload.user_id, payload.status);
    });

    // Call handlers
    wsClient.on('CallStarted', (payload) => {
      const user = useStore.getState().currentUser;
      if (payload.call.initiator.id !== user?.id) {
        setIncomingCall(payload.call);
      }
      handlersRef.current?.onCallStarted?.(payload.call);
    });

    wsClient.on('CallEnded', (payload) => {
      const { activeCall, incomingCall } = useStore.getState();
      if (activeCall?.id === payload.call_id) {
        resetCallState();
      }
      if (incomingCall?.id === payload.call_id) {
        setIncomingCall(null);
      }
      handlersRef.current?.onCallEnded?.(payload.call_id);
    });

    // Meeting handlers
    wsClient.on('MeetingInvite', (payload) => {
      addMeeting(payload.meeting);
      handlersRef.current?.onMeetingInvite?.(payload.meeting);
    });

    wsClient.on('MeetingUpdated', (payload) => {
      updateMeeting(payload.meeting);
    });

    wsClient.on('MeetingCancelled', (payload) => {
      removeMeeting(payload.meeting_id);
    });

    // Notifications
    wsClient.on('Notification', (payload) => {
      addNotification(payload.notification);
      handlersRef.current?.onNotification?.(payload.notification);
    });

    // Error handling
    wsClient.on('Error', (payload) => {
      console.error('[WebSocket Error]', payload.code, payload.message);
    });

    // Authentication confirmation
    wsClient.on('Authenticated', (payload) => {
      console.log('[WebSocket] Authenticated as', payload.user.display_name);
    });
  }, [
    addMessage,
    updateMessage,
    deleteMessage,
    addTypingUser,
    removeTypingUser,
    setUserStatus,
    setIncomingCall,
    resetCallState,
    addMeeting,
    updateMeeting,
    removeMeeting,
    addNotification,
  ]);

  // Connect and setup on auth
  useEffect(() => {
    if (isAuthenticated && accessToken) {
      setupHandlers();
      wsClient.connect(accessToken);

      return () => {
        wsClient.disconnect();
      };
    }
  }, [isAuthenticated, accessToken, setupHandlers]);

  // Channel subscription helpers
  const joinChannel = useCallback((channelId: string) => {
    wsClient.send({
      type: 'JoinChannel',
      payload: { channel_id: channelId },
    });
  }, []);

  const leaveChannel = useCallback((channelId: string) => {
    wsClient.send({
      type: 'LeaveChannel',
      payload: { channel_id: channelId },
    });
  }, []);

  // Typing indicator helpers
  const startTyping = useCallback((channelId: string) => {
    wsClient.send({
      type: 'StartTyping',
      payload: { channel_id: channelId },
    });
  }, []);

  const stopTyping = useCallback((channelId: string) => {
    wsClient.send({
      type: 'StopTyping',
      payload: { channel_id: channelId },
    });
  }, []);

  // Status update
  const updateStatus = useCallback((status: UserStatus, statusMessage?: string) => {
    wsClient.send({
      type: 'UpdateStatus',
      payload: { status, status_message: statusMessage },
    });
  }, []);

  return {
    isConnected: wsClient.isConnected(),
    joinChannel,
    leaveChannel,
    startTyping,
    stopTyping,
    updateStatus,
  };
}
