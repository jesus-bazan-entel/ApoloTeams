import { useEffect, useRef } from 'react';
import type { User } from '../../types';

interface ParticipantTileProps {
  stream?: MediaStream;
  user?: User;
  isLocal?: boolean;
  isMuted?: boolean;
  isVideoEnabled?: boolean;
}

function ParticipantTile({ stream, user, isLocal, isMuted, isVideoEnabled = true }: ParticipantTileProps) {
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    if (videoRef.current && stream) {
      videoRef.current.srcObject = stream;
    }
  }, [stream]);

  const getInitials = (name?: string) => {
    if (!name) return '?';
    return name
      .split(' ')
      .map((word) => word[0])
      .join('')
      .toUpperCase()
      .slice(0, 2);
  };

  const displayName = user?.display_name || 'Unknown';
  const hasVideo = stream && isVideoEnabled && stream.getVideoTracks().some(t => t.enabled);

  return (
    <div className="relative bg-slate-900 rounded-lg overflow-hidden aspect-video flex items-center justify-center">
      {/* Video or Avatar */}
      {hasVideo ? (
        <video
          ref={videoRef}
          autoPlay
          playsInline
          muted={isLocal}
          className="w-full h-full object-cover"
        />
      ) : (
        <div className="w-20 h-20 rounded-full bg-indigo-600 flex items-center justify-center text-white text-2xl font-semibold">
          {getInitials(displayName)}
        </div>
      )}

      {/* Overlay with name and status */}
      <div className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/70 to-transparent p-3">
        <div className="flex items-center gap-2">
          <span className="text-white text-sm font-medium truncate">
            {isLocal ? `${displayName} (You)` : displayName}
          </span>
          {isMuted && (
            <svg className="w-4 h-4 text-red-400" fill="currentColor" viewBox="0 0 24 24">
              <path d="M19 11c0 1.19-.34 2.3-.9 3.28l-1.23-1.23c.27-.62.43-1.31.43-2.05H19zm-4-6c0-1.66-1.34-3-3-3S9 3.34 9 5v.17l6 6V5zm-2.44 11.27L4.41 3.41c-.39-.39-1.02-.39-1.41 0s-.39 1.02 0 1.41l4.63 4.63c-.07.51-.13 1.02-.13 1.55 0 3.03 2.19 5.55 5.08 6.06V19c0 .55.45 1 1 1s1-.45 1-1v-2.35c1.05-.19 2-.62 2.78-1.22l1.64 1.64c.39.39 1.02.39 1.41 0s.39-1.02 0-1.41l-5.86-5.86z" />
            </svg>
          )}
        </div>
      </div>

      {/* Local indicator */}
      {isLocal && (
        <div className="absolute top-2 right-2">
          <span className="px-2 py-1 bg-teams-purple text-white text-xs rounded-full">
            You
          </span>
        </div>
      )}
    </div>
  );
}

interface ParticipantGridProps {
  localStream: MediaStream | null;
  remoteStreams: Record<string, MediaStream>;
  localUser?: User;
  participants?: Array<{ user: User; is_muted: boolean; is_video_enabled: boolean }>;
  isLocalAudioEnabled: boolean;
  isLocalVideoEnabled: boolean;
}

export function ParticipantGrid({
  localStream,
  remoteStreams,
  localUser,
  participants = [],
  isLocalAudioEnabled,
  isLocalVideoEnabled,
}: ParticipantGridProps) {
  const remoteEntries = Object.entries(remoteStreams);
  const totalParticipants = 1 + remoteEntries.length;

  // Determine grid layout based on participant count
  const getGridClass = () => {
    if (totalParticipants === 1) return 'grid-cols-1';
    if (totalParticipants === 2) return 'grid-cols-2';
    if (totalParticipants <= 4) return 'grid-cols-2 grid-rows-2';
    if (totalParticipants <= 6) return 'grid-cols-3 grid-rows-2';
    return 'grid-cols-3 grid-rows-3';
  };

  return (
    <div className={`grid gap-2 p-4 h-full ${getGridClass()}`}>
      {/* Local participant */}
      <ParticipantTile
        stream={localStream || undefined}
        user={localUser}
        isLocal={true}
        isMuted={!isLocalAudioEnabled}
        isVideoEnabled={isLocalVideoEnabled}
      />

      {/* Remote participants */}
      {remoteEntries.map(([peerId, stream]) => {
        const participant = participants.find((p) => p.user.id === peerId);
        return (
          <ParticipantTile
            key={peerId}
            stream={stream}
            user={participant?.user}
            isMuted={participant?.is_muted}
            isVideoEnabled={participant?.is_video_enabled}
          />
        );
      })}
    </div>
  );
}

export default ParticipantGrid;
