import { useStore } from '../../store';
import { useWebRTC } from '../../hooks/useWebRTC';
import { CallControls } from './CallControls';
import { ParticipantGrid } from './ParticipantGrid';

const MinimizeIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M18 12H6" />
  </svg>
);

export function VideoCallModal() {
  const {
    activeCall,
    isLocalAudioEnabled,
    isLocalVideoEnabled,
    localStream,
    remoteStreams,
    currentUser,
    setCallMinimized,
  } = useStore();

  const { toggleAudio, toggleVideo, leaveCall } = useWebRTC();

  if (!activeCall) {
    return null;
  }

  const handleMinimize = () => {
    setCallMinimized(true);
  };

  const formatDuration = () => {
    const start = new Date(activeCall.started_at);
    const now = new Date();
    const diff = Math.floor((now.getTime() - start.getTime()) / 1000);
    const minutes = Math.floor(diff / 60);
    const seconds = diff % 60;
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
  };

  return (
    <div className="fixed inset-0 z-50 bg-slate-900 flex flex-col">
      {/* Header */}
      <header className="h-14 bg-slate-800 flex items-center justify-between px-4">
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2">
            {activeCall.call_type === 'video' ? (
              <svg className="w-5 h-5 text-green-400" fill="currentColor" viewBox="0 0 20 20">
                <path d="M2 6a2 2 0 012-2h6a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V6zM14.553 7.106A1 1 0 0014 8v4a1 1 0 00.553.894l2 1A1 1 0 0018 13V7a1 1 0 00-1.447-.894l-2 1z" />
              </svg>
            ) : (
              <svg className="w-5 h-5 text-green-400" fill="currentColor" viewBox="0 0 20 20">
                <path d="M2 3a1 1 0 011-1h2.153a1 1 0 01.986.836l.74 4.435a1 1 0 01-.54 1.06l-1.548.773a11.037 11.037 0 006.105 6.105l.774-1.548a1 1 0 011.059-.54l4.435.74a1 1 0 01.836.986V17a1 1 0 01-1 1h-2C7.82 18 2 12.18 2 5V3z" />
              </svg>
            )}
            <span className="text-white font-medium">
              {activeCall.call_type === 'video' ? 'Video Call' : 'Audio Call'}
            </span>
          </div>
          <span className="text-slate-400 text-sm">|</span>
          <span className="text-slate-300 text-sm">
            {activeCall.participants.length} participant{activeCall.participants.length !== 1 ? 's' : ''}
          </span>
        </div>

        <div className="flex items-center gap-4">
          <span className="text-slate-400 text-sm font-mono">{formatDuration()}</span>
          <button
            onClick={handleMinimize}
            className="p-2 hover:bg-slate-700 rounded-lg transition-colors text-slate-300"
            title="Minimize"
          >
            <MinimizeIcon />
          </button>
        </div>
      </header>

      {/* Main content - participant grid */}
      <main className="flex-1 overflow-hidden">
        <ParticipantGrid
          localStream={localStream}
          remoteStreams={remoteStreams}
          localUser={currentUser || undefined}
          participants={activeCall.participants}
          isLocalAudioEnabled={isLocalAudioEnabled}
          isLocalVideoEnabled={isLocalVideoEnabled}
        />
      </main>

      {/* Footer with controls */}
      <footer className="h-20 bg-slate-800 flex items-center justify-center">
        <CallControls
          isAudioEnabled={isLocalAudioEnabled}
          isVideoEnabled={isLocalVideoEnabled}
          onToggleAudio={toggleAudio}
          onToggleVideo={toggleVideo}
          onHangUp={leaveCall}
          showVideoButton={activeCall.call_type === 'video'}
        />
      </footer>
    </div>
  );
}

export default VideoCallModal;
