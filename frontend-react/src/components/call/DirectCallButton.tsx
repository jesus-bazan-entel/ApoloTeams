import { useState, useEffect, useRef } from 'react';
import { useWebRTC } from '../../hooks/useWebRTC';
import { apiClient } from '../../api/client';
import type { User, CallType } from '../../types';

const VideoIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path d="M2 6a2 2 0 012-2h6a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V6zM14.553 7.106A1 1 0 0014 8v4a1 1 0 00.553.894l2 1A1 1 0 0018 13V7a1 1 0 00-1.447-.894l-2 1z" />
  </svg>
);

const CallIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path d="M2 3a1 1 0 011-1h2.153a1 1 0 01.986.836l.74 4.435a1 1 0 01-.54 1.06l-1.548.773a11.037 11.037 0 006.105 6.105l.774-1.548a1 1 0 011.059-.54l4.435.74a1 1 0 01.836.986V17a1 1 0 01-1 1h-2C7.82 18 2 12.18 2 5V3z" />
  </svg>
);

const SearchIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
  </svg>
);

const CloseIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
  </svg>
);

interface DirectCallButtonProps {
  className?: string;
}

export function DirectCallButton({ className = '' }: DirectCallButtonProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedUser, setSelectedUser] = useState<User | null>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const { startDirectCall } = useWebRTC();

  useEffect(() => {
    if (isOpen && searchInputRef.current) {
      searchInputRef.current.focus();
    }
  }, [isOpen]);

  useEffect(() => {
    const searchUsers = async () => {
      if (searchQuery.length < 2) {
        setUsers([]);
        return;
      }

      setLoading(true);
      try {
        const results = await apiClient.searchUsers(searchQuery);
        setUsers(results);
      } catch (error) {
        console.error('Failed to search users:', error);
      } finally {
        setLoading(false);
      }
    };

    const debounce = setTimeout(searchUsers, 300);
    return () => clearTimeout(debounce);
  }, [searchQuery]);

  const handleStartCall = async (user: User, callType: CallType) => {
    try {
      setSelectedUser(user);
      await startDirectCall(user.id, callType);
      setIsOpen(false);
      setSearchQuery('');
      setSelectedUser(null);
    } catch (error) {
      console.error('Failed to start call:', error);
      setSelectedUser(null);
    }
  };

  const getInitials = (name: string) => {
    return name
      .split(' ')
      .map((word) => word[0])
      .join('')
      .toUpperCase()
      .slice(0, 2);
  };

  return (
    <>
      {/* Trigger Button */}
      <button
        onClick={() => setIsOpen(true)}
        className={`flex items-center gap-2 px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors ${className}`}
        title="Start a call"
      >
        <CallIcon />
        <span>Call</span>
      </button>

      {/* Modal */}
      {isOpen && (
        <div className="fixed inset-0 z-50 flex items-start justify-center pt-20 bg-black/40 backdrop-blur-sm">
          <div className="bg-white rounded-xl shadow-2xl w-full max-w-md mx-4 overflow-hidden">
            {/* Header */}
            <div className="flex items-center justify-between p-4 border-b">
              <h2 className="text-lg font-semibold text-gray-900">Start a call</h2>
              <button
                onClick={() => {
                  setIsOpen(false);
                  setSearchQuery('');
                }}
                className="p-1 hover:bg-gray-100 rounded-lg transition-colors"
              >
                <CloseIcon />
              </button>
            </div>

            {/* Search Input */}
            <div className="p-4">
              <div className="relative">
                <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-gray-400">
                  <SearchIcon />
                </div>
                <input
                  ref={searchInputRef}
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Search for a person..."
                  className="w-full pl-10 pr-4 py-3 border border-gray-200 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all"
                />
              </div>
            </div>

            {/* Results */}
            <div className="max-h-80 overflow-y-auto">
              {loading ? (
                <div className="flex items-center justify-center py-8">
                  <div className="w-8 h-8 border-4 border-indigo-200 border-t-indigo-600 rounded-full animate-spin"></div>
                </div>
              ) : users.length === 0 ? (
                <div className="text-center py-8 text-gray-500">
                  {searchQuery.length < 2 ? (
                    <p>Type at least 2 characters to search</p>
                  ) : (
                    <p>No users found</p>
                  )}
                </div>
              ) : (
                <div className="divide-y">
                  {users.map((user) => (
                    <div
                      key={user.id}
                      className="flex items-center justify-between p-4 hover:bg-gray-50 transition-colors"
                    >
                      <div className="flex items-center gap-3">
                        {/* Avatar */}
                        <div className="w-10 h-10 rounded-full bg-indigo-600 flex items-center justify-center text-white font-medium">
                          {user.avatar_url ? (
                            <img
                              src={user.avatar_url}
                              alt={user.display_name}
                              className="w-full h-full rounded-full object-cover"
                            />
                          ) : (
                            getInitials(user.display_name)
                          )}
                        </div>

                        {/* Info */}
                        <div>
                          <p className="font-medium text-gray-900">{user.display_name}</p>
                          <p className="text-sm text-gray-500">@{user.username}</p>
                        </div>
                      </div>

                      {/* Call buttons */}
                      <div className="flex items-center gap-2">
                        <button
                          onClick={() => handleStartCall(user, 'audio')}
                          disabled={selectedUser?.id === user.id}
                          className="p-2 text-gray-600 hover:bg-gray-100 hover:text-indigo-600 rounded-lg transition-colors disabled:opacity-50"
                          title="Audio call"
                        >
                          <CallIcon />
                        </button>
                        <button
                          onClick={() => handleStartCall(user, 'video')}
                          disabled={selectedUser?.id === user.id}
                          className="p-2 text-gray-600 hover:bg-gray-100 hover:text-indigo-600 rounded-lg transition-colors disabled:opacity-50"
                          title="Video call"
                        >
                          <VideoIcon />
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Footer */}
            <div className="p-4 border-t bg-gray-50">
              <p className="text-xs text-gray-500 text-center">
                Select a person to start a direct call
              </p>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

export default DirectCallButton;
