/**
 * useWebSocketSetup Hook
 *
 * Encapsulates WebSocket connection and event handler setup.
 * Centralizes all WebSocket message handling in one place.
 *
 * This hook should be called once at the app root level (App.tsx).
 */

import { useEffect } from 'react';
import { useStore } from '../store';
import { wsClient } from '../websocket/client';

// Initialize handlers immediately on module load (before any React renders)
// This ensures handlers are ready before any WebSocket messages arrive
let handlersInitialized = false;

function initializeHandlers() {
  if (handlersInitialized) return;
  handlersInitialized = true;
  console.log('[WS] Initializing message handlers...');

  // Authentication
  wsClient.on('Authenticated', (payload) => {
    console.log('[WS] Authenticated as:', payload.user.display_name);
  });

  // Message handlers - use getState() to always get fresh store actions
  wsClient.on('NewMessage', (payload) => {
    const { message } = payload;
    const store = useStore.getState();
    console.log('[WS] NewMessage received:', message.id, 'for channel:', message.channel_id);

    store.addMessage(message.channel_id, message);

    // Increment unread if not in the current channel
    if (message.channel_id !== store.selectedChannelId) {
      store.incrementUnread(message.channel_id);
    }
  });

  wsClient.on('MessageUpdated', (payload) => {
    const { message } = payload;
    console.log('[WS] MessageUpdated:', message.id);
    useStore.getState().updateMessage(message.channel_id, message);
  });

  wsClient.on('MessageDeleted', (payload) => {
    console.log('[WS] MessageDeleted:', payload.message_id);
    useStore.getState().deleteMessage(payload.channel_id, payload.message_id);
  });

  // Typing indicators with auto-cleanup
  wsClient.on('UserTyping', (payload) => {
    console.log('[WS] UserTyping:', payload.user.display_name);
    useStore.getState().addTypingUser(payload.channel_id, payload.user);

    // Auto-remove after 3 seconds as safety net
    setTimeout(() => {
      useStore.getState().removeTypingUser(payload.channel_id, payload.user.id);
    }, 3000);
  });

  wsClient.on('UserStoppedTyping', (payload) => {
    console.log('[WS] UserStoppedTyping:', payload.user_id);
    useStore.getState().removeTypingUser(payload.channel_id, payload.user_id);
  });

  // User status updates
  wsClient.on('UserStatusChanged', (payload) => {
    console.log('[WS] UserStatusChanged:', payload.user_id, payload.status);
    useStore.getState().setUserStatus(payload.user_id, payload.status);
  });

  // Channel membership events
  wsClient.on('UserJoinedChannel', (payload) => {
    console.log('[WS] UserJoinedChannel:', payload);
  });

  wsClient.on('UserLeftChannel', (payload) => {
    console.log('[WS] UserLeftChannel:', payload);
  });

  // Call handlers
  wsClient.on('CallStarted', (payload) => {
    console.log('[WS] CallStarted:', payload.call.id);
    const store = useStore.getState();
    if (payload.call.initiator.id !== store.currentUser?.id) {
      store.setIncomingCall(payload.call);
    }
  });

  wsClient.on('CallEnded', (payload) => {
    console.log('[WS] CallEnded:', payload.call_id);
    const store = useStore.getState();
    if (store.activeCall?.id === payload.call_id) {
      store.resetCallState();
    }
    if (store.incomingCall?.id === payload.call_id) {
      store.setIncomingCall(null);
    }
  });

  // ParticipantJoined/ParticipantLeft handlers are in useWebRTC.ts
  // (they manage both store updates and peer connections)

  // Meeting handlers
  wsClient.on('MeetingInvite', (payload) => {
    console.log('[WS] MeetingInvite:', payload.meeting.title);
    useStore.getState().addMeeting(payload.meeting);
  });

  wsClient.on('MeetingUpdated', (payload) => {
    console.log('[WS] MeetingUpdated:', payload.meeting.id);
    useStore.getState().updateMeeting(payload.meeting);
  });

  wsClient.on('MeetingCancelled', (payload) => {
    console.log('[WS] MeetingCancelled:', payload.meeting_id);
    useStore.getState().removeMeeting(payload.meeting_id);
  });

  wsClient.on('MeetingStarting', (payload) => {
    console.log('[WS] MeetingStarting:', payload);
  });

  // Notifications
  wsClient.on('Notification', (payload) => {
    console.log('[WS] Notification:', payload.notification);
    useStore.getState().addNotification(payload.notification);
  });

  // Error handling
  wsClient.on('Error', (payload) => {
    console.error('[WS] Error:', payload.code, payload.message);
  });

  console.log('[WS] All handlers registered');
}

// Initialize handlers immediately when this module is imported
initializeHandlers();

export function useWebSocketSetup() {
  const { isAuthenticated, accessToken } = useStore();

  // Connect WebSocket when authenticated
  useEffect(() => {
    if (isAuthenticated && accessToken) {
      console.log('[WS] Connecting with token...');
      wsClient.connect(accessToken);

      return () => {
        console.log('[WS] Disconnecting...');
        wsClient.disconnect();
      };
    }
  }, [isAuthenticated, accessToken]);

  // Return connection status
  return {
    isConnected: wsClient.isConnected(),
  };
}
