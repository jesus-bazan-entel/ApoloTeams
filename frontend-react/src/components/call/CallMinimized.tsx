import { useEffect, useState } from 'react';
import { useStore } from '../../store';
import { useWebRTC } from '../../hooks/useWebRTC';

const ExpandIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
  </svg>
);

const HangUpIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
    <path d="M12 9c-1.6 0-3.15.25-4.6.72v3.1c0 .39-.23.74-.56.9-.98.49-1.87 1.12-2.66 1.85-.18.18-.43.28-.7.28-.28 0-.53-.11-.71-.29L.29 13.08c-.18-.17-.29-.42-.29-.7 0-.28.11-.53.29-.71C3.34 8.78 7.46 7 12 7s8.66 1.78 11.71 4.67c.18.18.29.43.29.71 0 .28-.11.53-.29.71l-2.48 2.48c-.18.18-.43.29-.71.29-.27 0-.52-.11-.7-.28-.79-.74-1.69-1.36-2.67-1.85-.33-.16-.56-.5-.56-.9v-3.1C15.15 9.25 13.6 9 12 9z" />
  </svg>
);

export function CallMinimized() {
  const { activeCall, isCallMinimized, setCallMinimized } = useStore();
  const { leaveCall } = useWebRTC();
  const [duration, setDuration] = useState('00:00');

  useEffect(() => {
    if (!activeCall) return;

    const interval = setInterval(() => {
      const start = new Date(activeCall.started_at);
      const now = new Date();
      const diff = Math.floor((now.getTime() - start.getTime()) / 1000);
      const minutes = Math.floor(diff / 60);
      const seconds = diff % 60;
      setDuration(`${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`);
    }, 1000);

    return () => clearInterval(interval);
  }, [activeCall]);

  if (!activeCall || !isCallMinimized) {
    return null;
  }

  const handleExpand = () => {
    setCallMinimized(false);
  };

  const handleHangUp = async () => {
    await leaveCall();
  };

  return (
    <div className="fixed bottom-4 right-4 z-50">
      <div className="bg-slate-800 border border-slate-700 rounded-xl shadow-2xl p-3 flex items-center gap-3 animate-slide-up">
        {/* Call indicator */}
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-green-500 animate-pulse"></div>
          <span className="text-white text-sm font-medium">
            {activeCall.call_type === 'video' ? 'Video Call' : 'Call'}
          </span>
        </div>

        {/* Duration */}
        <span className="text-slate-400 text-sm font-mono">{duration}</span>

        {/* Participants count */}
        <span className="text-slate-400 text-sm">
          {activeCall.participants.length} participant{activeCall.participants.length !== 1 ? 's' : ''}
        </span>

        {/* Actions */}
        <div className="flex items-center gap-1 ml-2">
          <button
            onClick={handleExpand}
            className="p-2 hover:bg-slate-700 rounded-lg transition-colors text-gray-300 hover:text-white"
            title="Expand"
          >
            <ExpandIcon />
          </button>
          <button
            onClick={handleHangUp}
            className="p-2 bg-red-600 hover:bg-red-700 rounded-lg transition-colors text-white"
            title="End call"
          >
            <HangUpIcon />
          </button>
        </div>
      </div>
    </div>
  );
}

export default CallMinimized;
