
interface CallControlsProps {
  isAudioEnabled: boolean;
  isVideoEnabled: boolean;
  onToggleAudio: () => void;
  onToggleVideo: () => void;
  onHangUp: () => void;
  showVideoButton?: boolean;
}

const MicOnIcon = () => (
  <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
    <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3zm5.91-3c-.49 0-.9.36-.98.85C16.52 14.2 14.47 16 12 16s-4.52-1.8-4.93-4.15c-.08-.49-.49-.85-.98-.85-.61 0-1.09.54-1 1.14.49 3 2.89 5.35 5.91 5.78V20c0 .55.45 1 1 1s1-.45 1-1v-2.08c3.02-.43 5.42-2.78 5.91-5.78.1-.6-.39-1.14-1-1.14z" />
  </svg>
);

const MicOffIcon = () => (
  <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
    <path d="M19 11c0 1.19-.34 2.3-.9 3.28l-1.23-1.23c.27-.62.43-1.31.43-2.05H19zm-4-6c0-1.66-1.34-3-3-3S9 3.34 9 5v.17l6 6V5zm-2.44 11.27L4.41 3.41c-.39-.39-1.02-.39-1.41 0s-.39 1.02 0 1.41l4.63 4.63c-.07.51-.13 1.02-.13 1.55 0 3.03 2.19 5.55 5.08 6.06V19c0 .55.45 1 1 1s1-.45 1-1v-2.35c1.05-.19 2-.62 2.78-1.22l1.64 1.64c.39.39 1.02.39 1.41 0s.39-1.02 0-1.41l-5.86-5.86z" />
  </svg>
);

const VideoOnIcon = () => (
  <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
    <path d="M17 10.5V7c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h12c.55 0 1-.45 1-1v-3.5l4 4v-11l-4 4z" />
  </svg>
);

const VideoOffIcon = () => (
  <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
    <path d="M21 6.5l-4 4V7c0-.55-.45-1-1-1H9.82L21 17.18V6.5zM3.27 2L2 3.27 4.73 6H4c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h12c.21 0 .39-.08.55-.18L19.73 21 21 19.73 3.27 2z" />
  </svg>
);

const HangUpIcon = () => (
  <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
    <path d="M12 9c-1.6 0-3.15.25-4.6.72v3.1c0 .39-.23.74-.56.9-.98.49-1.87 1.12-2.66 1.85-.18.18-.43.28-.7.28-.28 0-.53-.11-.71-.29L.29 13.08c-.18-.17-.29-.42-.29-.7 0-.28.11-.53.29-.71C3.34 8.78 7.46 7 12 7s8.66 1.78 11.71 4.67c.18.18.29.43.29.71 0 .28-.11.53-.29.71l-2.48 2.48c-.18.18-.43.29-.71.29-.27 0-.52-.11-.7-.28-.79-.74-1.69-1.36-2.67-1.85-.33-.16-.56-.5-.56-.9v-3.1C15.15 9.25 13.6 9 12 9z" />
  </svg>
);

export function CallControls({
  isAudioEnabled,
  isVideoEnabled,
  onToggleAudio,
  onToggleVideo,
  onHangUp,
  showVideoButton = true,
}: CallControlsProps) {
  return (
    <div className="flex items-center justify-center gap-4">
      {/* Mute/Unmute Button */}
      <button
        onClick={onToggleAudio}
        className={`p-4 rounded-full transition-colors ${
          isAudioEnabled
            ? 'bg-slate-700 hover:bg-slate-600 text-white'
            : 'bg-red-500 hover:bg-red-600 text-white'
        }`}
        title={isAudioEnabled ? 'Mute microphone' : 'Unmute microphone'}
      >
        {isAudioEnabled ? <MicOnIcon /> : <MicOffIcon />}
      </button>

      {/* Video On/Off Button */}
      {showVideoButton && (
        <button
          onClick={onToggleVideo}
          className={`p-4 rounded-full transition-colors ${
            isVideoEnabled
              ? 'bg-slate-700 hover:bg-slate-600 text-white'
              : 'bg-red-500 hover:bg-red-600 text-white'
          }`}
          title={isVideoEnabled ? 'Turn off camera' : 'Turn on camera'}
        >
          {isVideoEnabled ? <VideoOnIcon /> : <VideoOffIcon />}
        </button>
      )}

      {/* Hang Up Button */}
      <button
        onClick={onHangUp}
        className="p-4 rounded-full bg-red-600 hover:bg-red-700 text-white transition-colors"
        title="End call"
      >
        <HangUpIcon />
      </button>
    </div>
  );
}

export default CallControls;
