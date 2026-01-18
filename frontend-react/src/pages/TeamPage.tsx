import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useStore } from '../store';
import { apiClient } from '../api/client';
import type { Channel } from '../types';

// Icons
const HashIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path fillRule="evenodd" d="M9.243 3.03a1 1 0 01.727 1.213L9.53 6h2.94l.56-2.243a1 1 0 111.94.486L14.53 6H17a1 1 0 110 2h-2.97l-1 4H15a1 1 0 110 2h-2.47l-.56 2.242a1 1 0 11-1.94-.485L10.47 14H7.53l-.56 2.242a1 1 0 11-1.94-.485L5.47 14H3a1 1 0 110-2h2.97l1-4H5a1 1 0 110-2h2.47l.56-2.243a1 1 0 011.213-.727zM9.03 8l-1 4h2.94l1-4H9.03z" clipRule="evenodd" />
  </svg>
);

const ArrowLeftIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path fillRule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clipRule="evenodd" />
  </svg>
);

const PlusIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path fillRule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clipRule="evenodd" />
  </svg>
);

const UsersIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path d="M13 6a3 3 0 11-6 0 3 3 0 016 0zM18 8a2 2 0 11-4 0 2 2 0 014 0zM14 15a4 4 0 00-8 0v3h8v-3zM6 8a2 2 0 11-4 0 2 2 0 014 0zM16 18v-3a5.972 5.972 0 00-.75-2.906A3.005 3.005 0 0119 15v3h-3zM4.75 12.094A5.973 5.973 0 004 15v3H1v-3a3 3 0 013.75-2.906z" />
  </svg>
);

const CalendarIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path fillRule="evenodd" d="M6 2a1 1 0 00-1 1v1H4a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V6a2 2 0 00-2-2h-1V3a1 1 0 10-2 0v1H7V3a1 1 0 00-1-1zm0 5a1 1 0 000 2h8a1 1 0 100-2H6z" clipRule="evenodd" />
  </svg>
);

const CrownIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path d="M10 2l2.5 5 5.5.5-4 4 1 5.5L10 14l-5 3 1-5.5-4-4 5.5-.5L10 2z" />
  </svg>
);

const SettingsIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path fillRule="evenodd" d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z" clipRule="evenodd" />
  </svg>
);

function TeamPage() {
  const { teamId } = useParams<{ teamId: string }>();
  const navigate = useNavigate();
  const { currentUser, teams, channels, setChannels, setSelectedTeam } = useStore();
  const [loading, setLoading] = useState(false);
  const [showCreateChannel, setShowCreateChannel] = useState(false);
  const [newChannelName, setNewChannelName] = useState('');
  const [newChannelDescription, setNewChannelDescription] = useState('');
  const [creating, setCreating] = useState(false);

  const team = teams.find((t) => t.id === teamId);

  useEffect(() => {
    if (teamId) {
      setSelectedTeam(teamId);
      loadChannels();
    }
  }, [teamId, setSelectedTeam]);

  const loadChannels = async () => {
    if (!teamId) return;
    setLoading(true);
    try {
      const channelsData = await apiClient.listTeamChannels(teamId);
      setChannels(channelsData);
    } catch (error) {
      console.error('Failed to load channels:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateChannel = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!teamId || !newChannelName.trim()) return;

    setCreating(true);
    try {
      await apiClient.createChannel({
        team_id: teamId,
        name: newChannelName.trim(),
        description: newChannelDescription.trim() || undefined,
      });
      setShowCreateChannel(false);
      setNewChannelName('');
      setNewChannelDescription('');
      loadChannels();
    } catch (error) {
      console.error('Failed to create channel:', error);
    } finally {
      setCreating(false);
    }
  };

  const getInitials = (name: string) => {
    return name
      .split(' ')
      .map(word => word[0])
      .join('')
      .toUpperCase()
      .slice(0, 2);
  };

  const getChannelColor = (index: number) => {
    const colors = [
      'bg-teams-purple',
      'bg-teams-blue',
      'bg-emerald-500',
      'bg-orange-500',
      'bg-pink-500',
      'bg-cyan-500',
    ];
    return colors[index % colors.length];
  };

  return (
    <div className="min-h-screen bg-surface">
      {/* Header */}
      <header className="h-16 bg-white border-b border-gray-200 flex items-center justify-between px-6 shadow-sm">
        <div className="flex items-center gap-4">
          <button
            onClick={() => navigate('/')}
            className="p-2 hover:bg-gray-100 rounded-lg transition-colors"
          >
            <ArrowLeftIcon />
          </button>
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-teams-purple flex items-center justify-center text-white font-bold">
              {team ? getInitials(team.name) : 'T'}
            </div>
            <div>
              <h1 className="text-lg font-semibold text-gray-900">{team?.name || 'Team'}</h1>
              <p className="text-sm text-gray-500">{team?.member_count || 0} members</p>
            </div>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button className="p-2 hover:bg-gray-100 rounded-lg transition-colors" title="Team settings">
            <SettingsIcon />
          </button>
        </div>
      </header>

      <div className="max-w-7xl mx-auto px-6 py-8">
        <div className="grid gap-8 lg:grid-cols-3">
          {/* Channels Section */}
          <div className="lg:col-span-2 space-y-6">
            {/* Channels Header */}
            <div className="flex items-center justify-between">
              <h2 className="text-xl font-semibold text-gray-900">Channels</h2>
              <button
                onClick={() => setShowCreateChannel(true)}
                className="btn-teams-primary flex items-center gap-2"
              >
                <PlusIcon />
                New Channel
              </button>
            </div>

            {/* Channels List */}
            <div className="card-teams">
              {loading ? (
                <div className="flex items-center justify-center py-12">
                  <div className="flex flex-col items-center gap-4">
                    <div className="w-10 h-10 border-4 border-teams-purple-200 border-t-teams-purple rounded-full animate-spin"></div>
                    <p className="text-gray-500">Loading channels...</p>
                  </div>
                </div>
              ) : channels.length === 0 ? (
                <div className="text-center py-12">
                  <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-teams-purple-50 flex items-center justify-center">
                    <HashIcon />
                  </div>
                  <h3 className="text-lg font-semibold text-gray-900 mb-2">No channels yet</h3>
                  <p className="text-gray-500 mb-6">Create your first channel to start collaborating</p>
                  <button
                    onClick={() => setShowCreateChannel(true)}
                    className="btn-teams-primary"
                  >
                    Create Channel
                  </button>
                </div>
              ) : (
                <div className="divide-y divide-gray-100">
                  {channels.map((channel: Channel, index: number) => (
                    <div
                      key={channel.id}
                      onClick={() => navigate(`/chat/${channel.id}`)}
                      className="flex items-center gap-4 p-4 hover:bg-gray-50 cursor-pointer transition-colors group"
                    >
                      <div className={`w-12 h-12 rounded-lg ${getChannelColor(index)} flex items-center justify-center text-white`}>
                        <HashIcon />
                      </div>
                      <div className="flex-1 min-w-0">
                        <h3 className="font-semibold text-gray-900 group-hover:text-teams-purple transition-colors">
                          {channel.name}
                        </h3>
                        <p className="text-sm text-gray-500 truncate">
                          {channel.description || 'No description'}
                        </p>
                      </div>
                      <div className="flex items-center gap-2 text-sm text-gray-400">
                        <UsersIcon />
                        <span>{channel.member_count}</span>
                      </div>
                      <svg className="w-5 h-5 text-gray-400 group-hover:text-teams-purple group-hover:translate-x-1 transition-all" fill="currentColor" viewBox="0 0 20 20">
                        <path fillRule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clipRule="evenodd" />
                      </svg>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Team Info Sidebar */}
          <div className="space-y-6">
            {/* Team Details Card */}
            <div className="card-teams">
              <h3 className="text-lg font-semibold text-gray-900 mb-4">About this team</h3>
              
              <div className="space-y-4">
                {team?.description && (
                  <div>
                    <p className="text-gray-600">{team.description}</p>
                  </div>
                )}

                <div className="pt-4 border-t border-gray-100 space-y-3">
                  <div className="flex items-center gap-3 text-sm">
                    <div className="w-8 h-8 rounded-lg bg-teams-purple-50 flex items-center justify-center text-teams-purple">
                      <CrownIcon />
                    </div>
                    <div>
                      <p className="text-gray-500">Owner</p>
                      <p className="font-medium text-gray-900">
                        {team?.owner_id === currentUser?.id ? 'You' : 'Team Owner'}
                      </p>
                    </div>
                  </div>

                  <div className="flex items-center gap-3 text-sm">
                    <div className="w-8 h-8 rounded-lg bg-emerald-50 flex items-center justify-center text-emerald-600">
                      <UsersIcon />
                    </div>
                    <div>
                      <p className="text-gray-500">Members</p>
                      <p className="font-medium text-gray-900">{team?.member_count || 0} members</p>
                    </div>
                  </div>

                  <div className="flex items-center gap-3 text-sm">
                    <div className="w-8 h-8 rounded-lg bg-orange-50 flex items-center justify-center text-orange-600">
                      <CalendarIcon />
                    </div>
                    <div>
                      <p className="text-gray-500">Created</p>
                      <p className="font-medium text-gray-900">
                        {team?.created_at ? new Date(team.created_at).toLocaleDateString(undefined, {
                          year: 'numeric',
                          month: 'long',
                          day: 'numeric'
                        }) : 'N/A'}
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            {/* Quick Actions */}
            <div className="card-teams">
              <h3 className="text-lg font-semibold text-gray-900 mb-4">Quick Actions</h3>
              <div className="space-y-2">
                <button className="w-full flex items-center gap-3 px-4 py-3 rounded-lg hover:bg-gray-50 transition-colors text-left">
                  <div className="w-8 h-8 rounded-lg bg-teams-purple-50 flex items-center justify-center text-teams-purple">
                    <UsersIcon />
                  </div>
                  <span className="font-medium text-gray-700">Invite Members</span>
                </button>
                <button className="w-full flex items-center gap-3 px-4 py-3 rounded-lg hover:bg-gray-50 transition-colors text-left">
                  <div className="w-8 h-8 rounded-lg bg-gray-100 flex items-center justify-center text-gray-600">
                    <SettingsIcon />
                  </div>
                  <span className="font-medium text-gray-700">Team Settings</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Create Channel Modal */}
      {showCreateChannel && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 animate-in">
          <div className="bg-white rounded-lg shadow-teams-xl w-full max-w-md mx-4 animate-in">
            <div className="p-6 border-b border-gray-200">
              <h2 className="text-xl font-semibold text-gray-900">Create a new channel</h2>
              <p className="text-sm text-gray-500 mt-1">Channels are where your team communicates.</p>
            </div>
            <form onSubmit={handleCreateChannel} className="p-6 space-y-4">
              <div>
                <label htmlFor="channelName" className="block text-sm font-medium text-gray-700 mb-2">
                  Channel name
                </label>
                <div className="relative">
                  <span className="absolute inset-y-0 left-0 pl-3 flex items-center text-gray-400">
                    #
                  </span>
                  <input
                    id="channelName"
                    type="text"
                    value={newChannelName}
                    onChange={(e) => setNewChannelName(e.target.value)}
                    className="input-teams pl-8"
                    placeholder="e.g., general"
                    autoFocus
                  />
                </div>
              </div>
              <div>
                <label htmlFor="channelDescription" className="block text-sm font-medium text-gray-700 mb-2">
                  Description <span className="text-gray-400">(optional)</span>
                </label>
                <textarea
                  id="channelDescription"
                  value={newChannelDescription}
                  onChange={(e) => setNewChannelDescription(e.target.value)}
                  className="input-teams resize-none"
                  rows={3}
                  placeholder="What's this channel about?"
                />
              </div>
              <div className="flex justify-end gap-3 pt-4">
                <button
                  type="button"
                  onClick={() => {
                    setShowCreateChannel(false);
                    setNewChannelName('');
                    setNewChannelDescription('');
                  }}
                  className="btn-teams-ghost"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={creating || !newChannelName.trim()}
                  className="btn-teams-primary disabled:opacity-50"
                >
                  {creating ? 'Creating...' : 'Create Channel'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}

export default TeamPage;
