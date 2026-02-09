import { useCallback, useEffect, useRef } from 'react';
import { useStore } from '../store';
import { wsClient } from '../websocket/client';
import { apiClient } from '../api/client';
import type { Call, CallType } from '../types';

const DEFAULT_ICE_CONFIG: RTCConfiguration = {
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'stun:stun1.l.google.com:19302' },
  ],
};

export function useWebRTC() {
  const {
    currentUser,
    activeCall,
    incomingCall,
    isLocalAudioEnabled,
    isLocalVideoEnabled,
    localStream,
    setActiveCall,
    setIncomingCall,
    setLocalAudioEnabled,
    setLocalVideoEnabled,
    setLocalStream,
    addRemoteStream,
    removeRemoteStream,
    resetCallState,
    setCallMinimized,
  } = useStore();

  // Store peer connections by remote user ID
  const peerConnectionsRef = useRef<Map<string, RTCPeerConnection>>(new Map());
  const pendingCandidatesRef = useRef<Map<string, RTCIceCandidateInit[]>>(new Map());
  const iceConfigRef = useRef<RTCConfiguration>(DEFAULT_ICE_CONFIG);

  // Fetch ICE servers from backend (includes TURN if configured)
  const fetchIceServers = useCallback(async () => {
    try {
      const { ice_servers } = await apiClient.getIceServers();
      iceConfigRef.current = { iceServers: ice_servers };
      console.log('ICE servers configured:', ice_servers.length, 'servers');
    } catch (error) {
      console.warn('Failed to fetch ICE servers, using defaults:', error);
      iceConfigRef.current = DEFAULT_ICE_CONFIG;
    }
  }, []);

  // Initialize local media stream
  const initializeLocalMedia = useCallback(async (isVideo: boolean) => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: true,
        video: isVideo,
      });
      setLocalStream(stream);
      return stream;
    } catch (error) {
      console.error('Failed to get user media:', error);
      throw error;
    }
  }, [setLocalStream]);

  // Create a peer connection for a remote user
  const createPeerConnection = useCallback((remoteUserId: string, callId: string): RTCPeerConnection => {
    const existingPc = peerConnectionsRef.current.get(remoteUserId);
    if (existingPc) {
      return existingPc;
    }

    const pc = new RTCPeerConnection(iceConfigRef.current);

    // Handle ICE candidates
    pc.onicecandidate = (event) => {
      if (event.candidate) {
        wsClient.send({
          type: 'WebRTCIceCandidate',
          payload: {
            call_id: callId,
            from_user_id: currentUser?.id || '',
            candidate: JSON.stringify(event.candidate),
          },
        });
      }
    };

    // Handle connection state changes
    pc.onconnectionstatechange = () => {
      console.log(`Connection state with ${remoteUserId}:`, pc.connectionState);
      if (pc.connectionState === 'disconnected' || pc.connectionState === 'failed') {
        removeRemoteStream(remoteUserId);
        peerConnectionsRef.current.delete(remoteUserId);
      }
    };

    // Handle incoming tracks
    pc.ontrack = (event) => {
      console.log('Received remote track from', remoteUserId);
      if (event.streams[0]) {
        addRemoteStream(remoteUserId, event.streams[0]);
      }
    };

    // Add local tracks to peer connection
    const stream = useStore.getState().localStream;
    if (stream) {
      stream.getTracks().forEach((track) => {
        pc.addTrack(track, stream);
      });
    }

    peerConnectionsRef.current.set(remoteUserId, pc);
    return pc;
  }, [currentUser?.id, addRemoteStream, removeRemoteStream]);

  // Send an offer to a remote user
  const sendOffer = useCallback(async (remoteUserId: string, callId: string) => {
    const pc = createPeerConnection(remoteUserId, callId);

    try {
      const offer = await pc.createOffer();
      await pc.setLocalDescription(offer);

      wsClient.send({
        type: 'WebRTCOffer',
        payload: {
          call_id: callId,
          from_user_id: currentUser?.id || '',
          sdp: JSON.stringify(offer),
        },
      });
    } catch (error) {
      console.error('Failed to create offer:', error);
    }
  }, [createPeerConnection, currentUser?.id]);

  // Handle incoming WebRTC offer
  const handleOffer = useCallback(async (callId: string, fromUserId: string, sdp: string) => {
    const pc = createPeerConnection(fromUserId, callId);

    try {
      const offer = JSON.parse(sdp) as RTCSessionDescriptionInit;
      await pc.setRemoteDescription(new RTCSessionDescription(offer));

      // Apply any pending ICE candidates
      const pending = pendingCandidatesRef.current.get(fromUserId) || [];
      for (const candidate of pending) {
        await pc.addIceCandidate(new RTCIceCandidate(candidate));
      }
      pendingCandidatesRef.current.delete(fromUserId);

      const answer = await pc.createAnswer();
      await pc.setLocalDescription(answer);

      wsClient.send({
        type: 'WebRTCAnswer',
        payload: {
          call_id: callId,
          from_user_id: currentUser?.id || '',
          sdp: JSON.stringify(answer),
        },
      });
    } catch (error) {
      console.error('Failed to handle offer:', error);
    }
  }, [createPeerConnection, currentUser?.id]);

  // Handle incoming WebRTC answer
  const handleAnswer = useCallback(async (fromUserId: string, sdp: string) => {
    const pc = peerConnectionsRef.current.get(fromUserId);
    if (!pc) {
      console.error('No peer connection for', fromUserId);
      return;
    }

    try {
      const answer = JSON.parse(sdp) as RTCSessionDescriptionInit;
      await pc.setRemoteDescription(new RTCSessionDescription(answer));

      // Apply any pending ICE candidates
      const pending = pendingCandidatesRef.current.get(fromUserId) || [];
      for (const candidate of pending) {
        await pc.addIceCandidate(new RTCIceCandidate(candidate));
      }
      pendingCandidatesRef.current.delete(fromUserId);
    } catch (error) {
      console.error('Failed to handle answer:', error);
    }
  }, []);

  // Handle incoming ICE candidate
  const handleIceCandidate = useCallback(async (fromUserId: string, candidateStr: string) => {
    const pc = peerConnectionsRef.current.get(fromUserId);
    const candidate = JSON.parse(candidateStr) as RTCIceCandidateInit;

    if (pc && pc.remoteDescription) {
      try {
        await pc.addIceCandidate(new RTCIceCandidate(candidate));
      } catch (error) {
        console.error('Failed to add ICE candidate:', error);
      }
    } else {
      // Queue the candidate for later
      const pending = pendingCandidatesRef.current.get(fromUserId) || [];
      pending.push(candidate);
      pendingCandidatesRef.current.set(fromUserId, pending);
    }
  }, []);

  // Start a new call in a channel
  const startCall = useCallback(async (channelId: string, callType: CallType) => {
    try {
      // Fetch ICE servers (includes TURN if configured)
      await fetchIceServers();

      // Initialize media
      await initializeLocalMedia(callType === 'video');

      // Create call via API
      const call = await apiClient.startCall({ channel_id: channelId, call_type: callType });
      setActiveCall(call);

      // Join the call signaling channel
      wsClient.send({
        type: 'JoinCall',
        payload: { call_id: call.id },
      });

      return call;
    } catch (error) {
      console.error('Failed to start call:', error);
      throw error;
    }
  }, [fetchIceServers, initializeLocalMedia, setActiveCall]);

  // Start a direct call to a user (no channel required)
  const startDirectCall = useCallback(async (targetUserId: string, callType: CallType) => {
    try {
      // Fetch ICE servers (includes TURN if configured)
      await fetchIceServers();

      // Initialize media
      await initializeLocalMedia(callType === 'video');

      // Create direct call via API (creates DM channel automatically)
      const call = await apiClient.startDirectCall(targetUserId, callType);
      setActiveCall(call);

      // Join the call signaling channel
      wsClient.send({
        type: 'JoinCall',
        payload: { call_id: call.id },
      });

      return call;
    } catch (error) {
      console.error('Failed to start direct call:', error);
      throw error;
    }
  }, [fetchIceServers, initializeLocalMedia, setActiveCall]);

  // Join an existing call
  const joinCall = useCallback(async (call: Call) => {
    try {
      // Fetch ICE servers (includes TURN if configured)
      await fetchIceServers();

      // Initialize media
      await initializeLocalMedia(call.call_type === 'video');

      // Join call via API
      const joinedCall = await apiClient.joinCall(call.id);
      setActiveCall(joinedCall);
      setIncomingCall(null);

      // Join the call signaling channel
      wsClient.send({
        type: 'JoinCall',
        payload: { call_id: call.id },
      });

      // Send offers to existing participants
      for (const participant of call.participants) {
        if (participant.user.id !== currentUser?.id) {
          await sendOffer(participant.user.id, call.id);
        }
      }

      return joinedCall;
    } catch (error) {
      console.error('Failed to join call:', error);
      throw error;
    }
  }, [fetchIceServers, initializeLocalMedia, setActiveCall, setIncomingCall, currentUser?.id, sendOffer]);

  // Leave the current call
  const leaveCall = useCallback(async () => {
    const call = useStore.getState().activeCall;
    if (!call) return;

    try {
      // Leave call via API
      await apiClient.leaveCall(call.id);

      // Leave the call signaling channel
      wsClient.send({
        type: 'LeaveCall',
        payload: { call_id: call.id },
      });

      // Cleanup
      cleanup();
    } catch (error) {
      console.error('Failed to leave call:', error);
    }
  }, []);

  // Reject an incoming call
  const rejectCall = useCallback(() => {
    setIncomingCall(null);
  }, [setIncomingCall]);

  // Toggle local audio
  const toggleAudio = useCallback(() => {
    const stream = useStore.getState().localStream;
    if (stream) {
      const audioTrack = stream.getAudioTracks()[0];
      if (audioTrack) {
        audioTrack.enabled = !audioTrack.enabled;
        setLocalAudioEnabled(audioTrack.enabled);
      }
    }
  }, [setLocalAudioEnabled]);

  // Toggle local video
  const toggleVideo = useCallback(() => {
    const stream = useStore.getState().localStream;
    if (stream) {
      const videoTrack = stream.getVideoTracks()[0];
      if (videoTrack) {
        videoTrack.enabled = !videoTrack.enabled;
        setLocalVideoEnabled(videoTrack.enabled);
      }
    }
  }, [setLocalVideoEnabled]);

  // Cleanup all peer connections and media
  const cleanup = useCallback(() => {
    // Stop local stream
    const stream = useStore.getState().localStream;
    if (stream) {
      stream.getTracks().forEach((track) => track.stop());
    }

    // Close all peer connections
    peerConnectionsRef.current.forEach((pc) => pc.close());
    peerConnectionsRef.current.clear();
    pendingCandidatesRef.current.clear();

    // Reset store state
    resetCallState();
  }, [resetCallState]);

  // Set up WebSocket handlers for WebRTC signaling
  // Note: CallStarted/CallEnded are handled in App.tsx globally
  useEffect(() => {
    // Handle participant joined - pre-create peer connection (joiner sends offers)
    wsClient.on('ParticipantJoined', (data) => {
      const call = useStore.getState().activeCall;
      if (call?.id === data.call_id && data.participant.user.id !== currentUser?.id) {
        createPeerConnection(data.participant.user.id, data.call_id);
      }
    });

    // Handle participant left - cleanup their stream and connection
    wsClient.on('ParticipantLeft', (data) => {
      removeRemoteStream(data.user_id);
      const pc = peerConnectionsRef.current.get(data.user_id);
      if (pc) {
        pc.close();
        peerConnectionsRef.current.delete(data.user_id);
      }
    });

    // Handle WebRTC signaling messages
    wsClient.on('WebRTCOffer', (data) => {
      handleOffer(data.call_id, data.from_user_id, data.sdp);
    });

    wsClient.on('WebRTCAnswer', (data) => {
      handleAnswer(data.from_user_id, data.sdp);
    });

    wsClient.on('WebRTCIceCandidate', (data) => {
      handleIceCandidate(data.from_user_id, data.candidate);
    });

    // Cleanup on unmount
    return () => {
      wsClient.off('ParticipantJoined');
      wsClient.off('ParticipantLeft');
      wsClient.off('WebRTCOffer');
      wsClient.off('WebRTCAnswer');
      wsClient.off('WebRTCIceCandidate');
    };
  }, [currentUser?.id, handleOffer, handleAnswer, handleIceCandidate, createPeerConnection, removeRemoteStream]);

  return {
    // State
    activeCall,
    incomingCall,
    isLocalAudioEnabled,
    isLocalVideoEnabled,
    localStream,

    // Actions
    startCall,
    startDirectCall,
    joinCall,
    leaveCall,
    rejectCall,
    toggleAudio,
    toggleVideo,
    cleanup,
    setCallMinimized,
  };
}
