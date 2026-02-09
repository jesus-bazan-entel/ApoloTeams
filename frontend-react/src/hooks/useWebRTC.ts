import { useCallback, useEffect } from 'react';
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

// ── Shared module-level state (singleton) ──────────────────────────────────
const peerConnections = new Map<string, RTCPeerConnection>();
const pendingCandidates = new Map<string, RTCIceCandidateInit[]>();
let iceConfig: RTCConfiguration = DEFAULT_ICE_CONFIG;
let signalHandlersRegistered = false;

// ── Call tone generation (Web Audio API) ──────────────────────────────────
let ringbackCtx: AudioContext | null = null;
let ringbackOsc: OscillatorNode | null = null;
let ringbackGain: GainNode | null = null;
let ringbackTimer: ReturnType<typeof setTimeout> | null = null;

let ringtoneCtx: AudioContext | null = null;
let ringtoneOsc: OscillatorNode | null = null;
let ringtoneGain: GainNode | null = null;
let ringtoneTimer: ReturnType<typeof setTimeout> | null = null;

/** Ringback tone: caller hears 440Hz, 2s on / 4s off */
export function startRingbackTone() {
  stopRingbackTone();
  try {
    ringbackCtx = new AudioContext();
    ringbackGain = ringbackCtx.createGain();
    ringbackGain.gain.value = 0.15;
    ringbackGain.connect(ringbackCtx.destination);
    ringbackOsc = ringbackCtx.createOscillator();
    ringbackOsc.type = 'sine';
    ringbackOsc.frequency.value = 440;
    ringbackOsc.connect(ringbackGain);
    ringbackOsc.start();

    let on = true;
    const tick = () => {
      if (!ringbackGain) return;
      on = !on;
      ringbackGain.gain.value = on ? 0.15 : 0;
      ringbackTimer = setTimeout(tick, on ? 2000 : 4000);
    };
    ringbackTimer = setTimeout(tick, 2000);
  } catch (e) {
    console.warn('Failed to start ringback tone:', e);
  }
}

export function stopRingbackTone() {
  if (ringbackTimer) { clearTimeout(ringbackTimer); ringbackTimer = null; }
  if (ringbackOsc) { try { ringbackOsc.stop(); } catch (_) { /* already stopped */ } ringbackOsc = null; }
  if (ringbackGain) { ringbackGain.disconnect(); ringbackGain = null; }
  if (ringbackCtx) { ringbackCtx.close().catch(() => {}); ringbackCtx = null; }
}

/** Ringtone: callee hears 523Hz (C5), 1s on / 2s off */
export function startRingtone() {
  stopRingtone();
  try {
    ringtoneCtx = new AudioContext();
    ringtoneGain = ringtoneCtx.createGain();
    ringtoneGain.gain.value = 0.2;
    ringtoneGain.connect(ringtoneCtx.destination);
    ringtoneOsc = ringtoneCtx.createOscillator();
    ringtoneOsc.type = 'sine';
    ringtoneOsc.frequency.value = 523.25;
    ringtoneOsc.connect(ringtoneGain);
    ringtoneOsc.start();

    let on = true;
    const tick = () => {
      if (!ringtoneGain) return;
      on = !on;
      ringtoneGain.gain.value = on ? 0.2 : 0;
      ringtoneTimer = setTimeout(tick, on ? 1000 : 2000);
    };
    ringtoneTimer = setTimeout(tick, 1000);
  } catch (e) {
    console.warn('Failed to start ringtone:', e);
  }
}

export function stopRingtone() {
  if (ringtoneTimer) { clearTimeout(ringtoneTimer); ringtoneTimer = null; }
  if (ringtoneOsc) { try { ringtoneOsc.stop(); } catch (_) { /* already stopped */ } ringtoneOsc = null; }
  if (ringtoneGain) { ringtoneGain.disconnect(); ringtoneGain = null; }
  if (ringtoneCtx) { ringtoneCtx.close().catch(() => {}); ringtoneCtx = null; }
}

// ── Module-level helper functions ──────────────────────────────────────────

function fetchIceServers(): Promise<void> {
  return apiClient.getIceServers()
    .then(({ ice_servers }) => {
      iceConfig = { iceServers: ice_servers };
      console.log('ICE servers configured:', ice_servers.length, 'servers');
    })
    .catch((error) => {
      console.warn('Failed to fetch ICE servers, using defaults:', error);
      iceConfig = DEFAULT_ICE_CONFIG;
    });
}

function createPeerConnection(remoteUserId: string, callId: string, stream?: MediaStream | null): RTCPeerConnection {
  const existing = peerConnections.get(remoteUserId);
  if (existing) return existing;

  const currentUser = useStore.getState().currentUser;
  const pc = new RTCPeerConnection(iceConfig);

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

  pc.oniceconnectionstatechange = () => {
    console.log(`[WebRTC] ICE state with ${remoteUserId}:`, pc.iceConnectionState);
  };

  pc.onconnectionstatechange = () => {
    console.log(`[WebRTC] Connection state with ${remoteUserId}:`, pc.connectionState);
    if (pc.connectionState === 'connected') {
      // Media is flowing — stop tones
      stopRingbackTone();
      stopRingtone();
    }
    if (pc.connectionState === 'disconnected' || pc.connectionState === 'failed') {
      useStore.getState().removeRemoteStream(remoteUserId);
      peerConnections.delete(remoteUserId);
    }
  };

  pc.onsignalingstatechange = () => {
    console.log(`[WebRTC] Signaling state with ${remoteUserId}:`, pc.signalingState);
  };

  pc.ontrack = (event) => {
    console.log('[WebRTC] Received remote track from', remoteUserId);
    if (event.streams[0]) {
      useStore.getState().addRemoteStream(remoteUserId, event.streams[0]);
    }
  };

  // Add local tracks — prefer the explicitly passed stream over the store
  const localStream = stream ?? useStore.getState().localStream;
  if (localStream) {
    localStream.getTracks().forEach((track) => {
      pc.addTrack(track, localStream);
    });
    console.log(`[WebRTC] Added ${localStream.getTracks().length} local tracks to PC for ${remoteUserId}`);
  } else {
    console.warn(`[WebRTC] No local stream available when creating PC for ${remoteUserId}`);
  }

  peerConnections.set(remoteUserId, pc);
  return pc;
}

async function sendOffer(remoteUserId: string, callId: string, stream?: MediaStream | null) {
  const pc = createPeerConnection(remoteUserId, callId, stream);
  const currentUser = useStore.getState().currentUser;

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
    console.log(`[WebRTC] Sent offer to ${remoteUserId} for call ${callId}`);
  } catch (error) {
    console.error('Failed to create offer:', error);
  }
}

async function handleOffer(callId: string, fromUserId: string, sdp: string) {
  const pc = createPeerConnection(fromUserId, callId, useStore.getState().localStream);
  const currentUser = useStore.getState().currentUser;

  try {
    const offer = JSON.parse(sdp) as RTCSessionDescriptionInit;
    await pc.setRemoteDescription(new RTCSessionDescription(offer));

    const pending = pendingCandidates.get(fromUserId) || [];
    for (const candidate of pending) {
      await pc.addIceCandidate(new RTCIceCandidate(candidate));
    }
    pendingCandidates.delete(fromUserId);

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
}

async function handleAnswer(fromUserId: string, sdp: string) {
  const pc = peerConnections.get(fromUserId);
  if (!pc) {
    console.error('No peer connection for', fromUserId);
    return;
  }

  try {
    const answer = JSON.parse(sdp) as RTCSessionDescriptionInit;
    await pc.setRemoteDescription(new RTCSessionDescription(answer));

    const pending = pendingCandidates.get(fromUserId) || [];
    for (const candidate of pending) {
      await pc.addIceCandidate(new RTCIceCandidate(candidate));
    }
    pendingCandidates.delete(fromUserId);
  } catch (error) {
    console.error('Failed to handle answer:', error);
  }
}

async function handleIceCandidate(fromUserId: string, candidateStr: string) {
  const pc = peerConnections.get(fromUserId);
  const candidate = JSON.parse(candidateStr) as RTCIceCandidateInit;

  if (pc && pc.remoteDescription) {
    try {
      await pc.addIceCandidate(new RTCIceCandidate(candidate));
    } catch (error) {
      console.error('Failed to add ICE candidate:', error);
    }
  } else {
    const pending = pendingCandidates.get(fromUserId) || [];
    pending.push(candidate);
    pendingCandidates.set(fromUserId, pending);
  }
}

function cleanupConnections() {
  stopRingbackTone();
  stopRingtone();
  const stream = useStore.getState().localStream;
  if (stream) {
    stream.getTracks().forEach((track) => track.stop());
  }
  peerConnections.forEach((pc) => pc.close());
  peerConnections.clear();
  pendingCandidates.clear();
  useStore.getState().resetCallState();
}

// ── Register signaling handlers ONCE at module level ───────────────────────
function registerSignalHandlers() {
  if (signalHandlersRegistered) return;
  signalHandlersRegistered = true;

  wsClient.on('ParticipantJoined', (data) => {
    const store = useStore.getState();
    store.addCallParticipant(data.call_id, data.participant);

    const call = store.activeCall;
    if (call?.id === data.call_id && data.participant.user.id !== store.currentUser?.id) {
      console.log(`[WebRTC] New participant ${data.participant.user.id} joined our call`);
      stopRingbackTone();
      // Pre-create the peer connection with local tracks so it's ready
      // when the joiner's offer arrives. The JOINER sends the offer (in joinCall),
      // not us — sending from both sides causes a glare condition.
      createPeerConnection(data.participant.user.id, data.call_id, store.localStream);
    }
  });

  wsClient.on('ParticipantLeft', (data) => {
    // Update participant count in store
    useStore.getState().removeCallParticipant(data.call_id, data.user_id);
    // Cleanup peer connection
    useStore.getState().removeRemoteStream(data.user_id);
    const pc = peerConnections.get(data.user_id);
    if (pc) {
      pc.close();
      peerConnections.delete(data.user_id);
    }
  });

  wsClient.on('WebRTCOffer', (data) => {
    handleOffer(data.call_id, data.from_user_id, data.sdp);
  });

  wsClient.on('WebRTCAnswer', (data) => {
    handleAnswer(data.from_user_id, data.sdp);
  });

  wsClient.on('WebRTCIceCandidate', (data) => {
    handleIceCandidate(data.from_user_id, data.candidate);
  });

  console.log('[WebRTC] Signaling handlers registered:', wsClient.debugHandlers());
}

// Register immediately on module import
registerSignalHandlers();

// ── React hook (thin wrapper) ──────────────────────────────────────────────
export function useWebRTC() {
  const {
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
    setCallMinimized,
  } = useStore();

  // Ensure handlers are registered (idempotent)
  useEffect(() => {
    registerSignalHandlers();
  }, []);

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

  const startCall = useCallback(async (channelId: string, callType: CallType) => {
    try {
      await fetchIceServers();
      await initializeLocalMedia(callType === 'video');

      const call = await apiClient.startCall({ channel_id: channelId, call_type: callType });
      setActiveCall(call);

      wsClient.send({
        type: 'JoinCall',
        payload: { call_id: call.id },
      });

      // Caller hears ringback while waiting for callee
      startRingbackTone();

      return call;
    } catch (error) {
      console.error('Failed to start call:', error);
      throw error;
    }
  }, [initializeLocalMedia, setActiveCall]);

  const startDirectCall = useCallback(async (targetUserId: string, callType: CallType) => {
    try {
      await fetchIceServers();
      await initializeLocalMedia(callType === 'video');

      const call = await apiClient.startDirectCall(targetUserId, callType);
      setActiveCall(call);

      wsClient.send({
        type: 'JoinCall',
        payload: { call_id: call.id },
      });

      // Caller hears ringback while waiting for callee
      startRingbackTone();

      return call;
    } catch (error) {
      console.error('Failed to start direct call:', error);
      throw error;
    }
  }, [initializeLocalMedia, setActiveCall]);

  const joinCall = useCallback(async (call: Call) => {
    try {
      // Stop ringtone — callee is answering
      stopRingtone();

      await fetchIceServers();
      const stream = await initializeLocalMedia(call.call_type === 'video');

      const joinedCall = await apiClient.joinCall(call.id);
      setActiveCall(joinedCall);
      setIncomingCall(null);

      wsClient.send({
        type: 'JoinCall',
        payload: { call_id: call.id },
      });

      // Send offers to existing participants, passing stream directly
      // to avoid timing issue with the Zustand store update
      for (const participant of call.participants) {
        if (participant.user.id !== useStore.getState().currentUser?.id) {
          await sendOffer(participant.user.id, call.id, stream);
        }
      }

      return joinedCall;
    } catch (error) {
      console.error('Failed to join call:', error);
      throw error;
    }
  }, [initializeLocalMedia, setActiveCall, setIncomingCall]);

  const leaveCall = useCallback(async () => {
    const call = useStore.getState().activeCall;
    if (!call) return;

    try {
      await apiClient.leaveCall(call.id);
      wsClient.send({
        type: 'LeaveCall',
        payload: { call_id: call.id },
      });
      cleanupConnections();
    } catch (error) {
      console.error('Failed to leave call:', error);
    }
  }, []);

  const rejectCall = useCallback(() => {
    stopRingtone();
    setIncomingCall(null);
  }, [setIncomingCall]);

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

  return {
    activeCall,
    incomingCall,
    isLocalAudioEnabled,
    isLocalVideoEnabled,
    localStream,
    startCall,
    startDirectCall,
    joinCall,
    leaveCall,
    rejectCall,
    toggleAudio,
    toggleVideo,
    cleanup: cleanupConnections,
    setCallMinimized,
  };
}
