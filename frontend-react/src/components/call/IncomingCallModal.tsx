import { useStore } from '../../store';
import { useWebRTC } from '../../hooks/useWebRTC';

const AcceptIcon = () => (
  <svg className="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
    <path d="M20.01 15.38c-1.23 0-2.42-.2-3.53-.56-.35-.12-.74-.03-1.01.24l-1.57 1.97c-2.83-1.35-5.48-3.9-6.89-6.83l1.95-1.66c.27-.28.35-.67.24-1.02-.37-1.11-.56-2.3-.56-3.53 0-.54-.45-.99-.99-.99H4.19C3.65 3 3 3.24 3 3.99 3 13.28 10.73 21 20.01 21c.71 0 .99-.63.99-1.18v-3.45c0-.54-.45-.99-.99-.99z" />
  </svg>
);

const DeclineIcon = () => (
  <svg className="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
    <path d="M12 9c-1.6 0-3.15.25-4.6.72v3.1c0 .39-.23.74-.56.9-.98.49-1.87 1.12-2.66 1.85-.18.18-.43.28-.7.28-.28 0-.53-.11-.71-.29L.29 13.08c-.18-.17-.29-.42-.29-.7 0-.28.11-.53.29-.71C3.34 8.78 7.46 7 12 7s8.66 1.78 11.71 4.67c.18.18.29.43.29.71 0 .28-.11.53-.29.71l-2.48 2.48c-.18.18-.43.29-.71.29-.27 0-.52-.11-.7-.28-.79-.74-1.69-1.36-2.67-1.85-.33-.16-.56-.5-.56-.9v-3.1C15.15 9.25 13.6 9 12 9z" />
  </svg>
);

export function IncomingCallModal() {
  const { incomingCall } = useStore();
  const { joinCall, rejectCall } = useWebRTC();

  if (!incomingCall) {
    return null;
  }

  const caller = incomingCall.initiator;
  const isVideo = incomingCall.call_type === 'video';

  const getInitials = (name: string) => {
    return name
      .split(' ')
      .map((word) => word[0])
      .join('')
      .toUpperCase()
      .slice(0, 2);
  };

  const handleAccept = async () => {
    try {
      await joinCall(incomingCall);
    } catch (error) {
      console.error('Failed to join call:', error);
    }
  };

  const handleDecline = () => {
    rejectCall();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-white rounded-2xl shadow-2xl p-8 max-w-sm w-full mx-4 animate-pulse-slow">
        {/* Caller info */}
        <div className="flex flex-col items-center mb-8">
          {/* Avatar with ring animation */}
          <div className="relative mb-4">
            <div className="absolute inset-0 rounded-full bg-green-400 animate-ping opacity-20"></div>
            <div className="absolute inset-0 rounded-full bg-green-400 animate-ping opacity-20 animation-delay-300"></div>
            <div className="relative w-24 h-24 rounded-full bg-indigo-600 flex items-center justify-center text-white text-3xl font-semibold">
              {caller.avatar_url ? (
                <img
                  src={caller.avatar_url}
                  alt={caller.display_name}
                  className="w-full h-full rounded-full object-cover"
                />
              ) : (
                getInitials(caller.display_name)
              )}
            </div>
          </div>

          <h2 className="text-xl font-semibold text-gray-900 mb-1">
            {caller.display_name}
          </h2>
          <p className="text-gray-500">
            {isVideo ? 'Video Call' : 'Audio Call'}
          </p>
        </div>

        {/* Call type indicator */}
        <div className="flex items-center justify-center gap-2 mb-8 text-gray-600">
          {isVideo ? (
            <>
              <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                <path d="M2 6a2 2 0 012-2h6a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V6zM14.553 7.106A1 1 0 0014 8v4a1 1 0 00.553.894l2 1A1 1 0 0018 13V7a1 1 0 00-1.447-.894l-2 1z" />
              </svg>
              <span>Incoming video call...</span>
            </>
          ) : (
            <>
              <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                <path d="M2 3a1 1 0 011-1h2.153a1 1 0 01.986.836l.74 4.435a1 1 0 01-.54 1.06l-1.548.773a11.037 11.037 0 006.105 6.105l.774-1.548a1 1 0 011.059-.54l4.435.74a1 1 0 01.836.986V17a1 1 0 01-1 1h-2C7.82 18 2 12.18 2 5V3z" />
              </svg>
              <span>Incoming call...</span>
            </>
          )}
        </div>

        {/* Action buttons */}
        <div className="flex items-center justify-center gap-8">
          {/* Decline button */}
          <button
            onClick={handleDecline}
            className="flex flex-col items-center gap-2 group"
          >
            <div className="w-16 h-16 rounded-full bg-red-500 hover:bg-red-600 flex items-center justify-center text-white transition-all transform group-hover:scale-110 shadow-lg">
              <DeclineIcon />
            </div>
            <span className="text-sm text-gray-600">Decline</span>
          </button>

          {/* Accept button */}
          <button
            onClick={handleAccept}
            className="flex flex-col items-center gap-2 group"
          >
            <div className="w-16 h-16 rounded-full bg-green-500 hover:bg-green-600 flex items-center justify-center text-white transition-all transform group-hover:scale-110 shadow-lg">
              <AcceptIcon />
            </div>
            <span className="text-sm text-gray-600">Accept</span>
          </button>
        </div>
      </div>
    </div>
  );
}

export default IncomingCallModal;
