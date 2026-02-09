import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useStore } from '../store';
import { apiClient } from '../api/client';
import type { Channel, User } from '../types';
import { ArrowLeft, Settings, Plus, Hash, Users, Search, UserPlus, Check, Star, ChevronRight } from 'lucide-react';

function TeamPage() {
  const { teamId } = useParams<{ teamId: string }>();
  const navigate = useNavigate();
  const { currentUser, teams, channels, setChannels, setSelectedTeam } = useStore();
  const [loading, setLoading] = useState(false);
  const [showCreateChannel, setShowCreateChannel] = useState(false);
  const [newChannelName, setNewChannelName] = useState('');
  const [newChannelDescription, setNewChannelDescription] = useState('');
  const [creating, setCreating] = useState(false);

  // Invite members state
  const [showInviteModal, setShowInviteModal] = useState(false);
  const [userSearchQuery, setUserSearchQuery] = useState('');
  const [searchedUsers, setSearchedUsers] = useState<User[]>([]);
  const [searchingUsers, setSearchingUsers] = useState(false);
  const [addingMember, setAddingMember] = useState<string | null>(null);
  const [teamMembers, setTeamMembers] = useState<any[]>([]);

  const team = teams.find((t) => t.id === teamId);

  useEffect(() => {
    if (teamId) {
      setSelectedTeam(teamId);
      loadChannels();
      loadTeamMembers();
    }
  }, [teamId, setSelectedTeam]);

  // Search users effect with debounce
  useEffect(() => {
    const searchUsers = async () => {
      if (userSearchQuery.trim().length < 2) {
        setSearchedUsers([]);
        return;
      }
      setSearchingUsers(true);
      try {
        const users = await apiClient.searchUsers(userSearchQuery);
        // Filter out current user and existing team members
        const memberIds = teamMembers.map(m => m.user?.id || m.user_id);
        setSearchedUsers(users.filter(u => u.id !== currentUser?.id && !memberIds.includes(u.id)));
      } catch (error) {
        console.error('Failed to search users:', error);
      } finally {
        setSearchingUsers(false);
      }
    };

    const debounceTimer = setTimeout(searchUsers, 300);
    return () => clearTimeout(debounceTimer);
  }, [userSearchQuery, currentUser?.id, teamMembers]);

  const loadTeamMembers = async () => {
    if (!teamId) return;
    try {
      const members = await apiClient.listTeamMembers(teamId);
      setTeamMembers(members);
    } catch (error) {
      console.error('Failed to load team members:', error);
    }
  };

  const handleOpenInviteModal = () => {
    loadTeamMembers();
    setShowInviteModal(true);
  };

  const handleAddMember = async (userId: string) => {
    if (!teamId) return;
    setAddingMember(userId);
    try {
      await apiClient.addTeamMember(teamId, userId);
      // Refresh team members list
      await loadTeamMembers();
      // Remove the added user from search results
      setSearchedUsers(prev => prev.filter(u => u.id !== userId));
    } catch (error) {
      console.error('Failed to add team member:', error);
    } finally {
      setAddingMember(null);
    }
  };

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
      'bg-indigo-600',
      'bg-cyan-500',
      'bg-emerald-500',
      'bg-amber-500',
      'bg-rose-500',
      'bg-violet-500',
    ];
    return colors[index % colors.length];
  };

  return (
    <div className="h-full overflow-y-auto bg-surface">
      {/* Header */}
      <header className="h-16 bg-white border-b border-slate-200 flex items-center justify-between px-6 shadow-sm">
        <div className="flex items-center gap-4">
          <button
            onClick={() => navigate('/')}
            className="p-2 hover:bg-slate-100 rounded-lg transition-colors"
          >
            <ArrowLeft size={20} />
          </button>
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-indigo-600 flex items-center justify-center text-white font-bold">
              {team ? getInitials(team.name) : 'T'}
            </div>
            <div>
              <h1 className="text-lg font-semibold text-gray-900">{team?.name || 'Team'}</h1>
              <p className="text-sm text-gray-500">{team?.member_count || 0} members</p>
            </div>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button className="p-2 hover:bg-slate-100 rounded-lg transition-colors" title="Team settings">
            <Settings size={20} />
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
                <Plus size={20} />
                New Channel
              </button>
            </div>

            {/* Channels List */}
            <div className="card-teams">
              {loading ? (
                <div className="flex items-center justify-center py-12">
                  <div className="flex flex-col items-center gap-4">
                    <div className="w-10 h-10 border-4 border-indigo-200 border-t-indigo-600 rounded-full animate-spin"></div>
                    <p className="text-gray-500">Loading channels...</p>
                  </div>
                </div>
              ) : channels.length === 0 ? (
                <div className="text-center py-12">
                  <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-indigo-50 flex items-center justify-center text-indigo-600">
                    <Hash size={20} />
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
                <div className="divide-y divide-slate-100">
                  {channels.map((channel: Channel, index: number) => (
                    <div
                      key={channel.id}
                      onClick={() => navigate(`/chat/${channel.id}`)}
                      className="flex items-center gap-4 p-4 hover:bg-slate-50 cursor-pointer transition-colors group"
                    >
                      <div className={`w-12 h-12 rounded-lg ${getChannelColor(index)} flex items-center justify-center text-white`}>
                        <Hash size={20} />
                      </div>
                      <div className="flex-1 min-w-0">
                        <h3 className="font-semibold text-gray-900 group-hover:text-indigo-600 transition-colors">
                          {channel.name}
                        </h3>
                        <p className="text-sm text-gray-500 truncate">
                          {channel.description || 'No description'}
                        </p>
                      </div>
                      <div className="flex items-center gap-2 text-sm text-gray-400">
                        <Users size={20} />
                        <span>{channel.member_count}</span>
                      </div>
                      <ChevronRight size={18} className="text-gray-400 group-hover:text-indigo-600 group-hover:translate-x-1 transition-all" />
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

                <div className="pt-4 border-t border-slate-100 space-y-3">
                  <div className="flex items-center gap-3 text-sm">
                    <div className="w-8 h-8 rounded-lg bg-indigo-50 flex items-center justify-center text-indigo-600">
                      <Star size={16} />
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
                      <Users size={20} />
                    </div>
                    <div>
                      <p className="text-gray-500">Members</p>
                      <p className="font-medium text-gray-900">{team?.member_count || 0} members</p>
                    </div>
                  </div>

                  <div className="flex items-center gap-3 text-sm">
                    <div className="w-8 h-8 rounded-lg bg-orange-50 flex items-center justify-center text-orange-600">
                      <Users size={20} />
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
                <button
                  onClick={handleOpenInviteModal}
                  className="w-full flex items-center gap-3 px-4 py-3 rounded-xl hover:bg-slate-50 transition-colors text-left"
                >
                  <div className="w-8 h-8 rounded-lg bg-indigo-50 flex items-center justify-center text-indigo-600">
                    <UserPlus size={20} />
                  </div>
                  <span className="font-medium text-gray-700">Invite Members</span>
                </button>
                <button className="w-full flex items-center gap-3 px-4 py-3 rounded-xl hover:bg-slate-50 transition-colors text-left">
                  <div className="w-8 h-8 rounded-lg bg-slate-100 flex items-center justify-center text-gray-600">
                    <Settings size={20} />
                  </div>
                  <span className="font-medium text-gray-700">Team Settings</span>
                </button>
              </div>
            </div>

            {/* Team Members */}
            <div className="card-teams">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold text-gray-900">Members</h3>
                <span className="text-sm text-gray-500">{teamMembers.length}</span>
              </div>

              {teamMembers.length === 0 ? (
                <div className="text-center py-4 text-gray-500">
                  <p>No members yet</p>
                </div>
              ) : (
                <div className="space-y-3">
                  {teamMembers.map((member) => {
                    const isOwner = member.role === 'owner';
                    const isCurrentUser = member.user_id === currentUser?.id || member.user?.id === currentUser?.id;

                    return (
                      <div
                        key={member.user_id || member.user?.id}
                        className="flex items-center gap-3"
                      >
                        <div className={`w-10 h-10 rounded-full flex items-center justify-center text-white font-semibold ${
                          isOwner ? 'bg-indigo-600' : 'bg-emerald-500'
                        }`}>
                          {member.user?.display_name?.[0]?.toUpperCase() ||
                           member.user?.username?.[0]?.toUpperCase() ||
                           'U'}
                        </div>
                        <div className="flex-1 min-w-0">
                          <p className="font-medium text-gray-900 truncate">
                            {member.user?.display_name || member.user?.username || 'Unknown User'}
                            {isCurrentUser && (
                              <span className="text-indigo-600 ml-1">(You)</span>
                            )}
                          </p>
                          <div className="flex items-center gap-1 text-sm text-gray-500">
                            {isOwner && (
                              <span className="text-indigo-600 flex items-center gap-1">
                                <Star size={14} />
                              </span>
                            )}
                            <span>{member.role || 'member'}</span>
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Create Channel Modal */}
      {showCreateChannel && (
        <div className="modal-overlay">
          <div className="modal-content w-full max-w-md mx-4">
            <div className="p-6 border-b border-slate-200">
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

      {/* Invite Members Modal */}
      {showInviteModal && (
        <div className="modal-overlay">
          <div className="modal-content w-full max-w-md mx-4 max-h-[80vh] flex flex-col">
            <div className="p-6 border-b border-slate-200">
              <h2 className="text-xl font-semibold text-gray-900">Invite Members</h2>
              <p className="text-sm text-gray-500 mt-1">Search for users to add to {team?.name}</p>
            </div>

            <div className="p-6 flex-1 overflow-y-auto">
              {/* Search Input */}
              <div className="relative mb-4">
                <span className="absolute inset-y-0 left-0 pl-3 flex items-center text-gray-400">
                  <Search size={20} />
                </span>
                <input
                  type="text"
                  value={userSearchQuery}
                  onChange={(e) => setUserSearchQuery(e.target.value)}
                  className="input-teams pl-10"
                  placeholder="Search users by name or email..."
                  autoFocus
                />
              </div>

              {/* Search Results */}
              {searchingUsers ? (
                <div className="flex items-center justify-center py-8">
                  <div className="w-8 h-8 border-4 border-indigo-200 border-t-indigo-600 rounded-full animate-spin"></div>
                </div>
              ) : searchedUsers.length > 0 ? (
                <div className="space-y-2 mb-6">
                  <h4 className="text-sm font-medium text-gray-500 mb-2">Search Results</h4>
                  {searchedUsers.map((user) => (
                    <div
                      key={user.id}
                      className="flex items-center justify-between p-3 rounded-xl bg-slate-50 hover:bg-slate-100 transition-colors"
                    >
                      <div className="flex items-center gap-3">
                        <div className="w-10 h-10 rounded-full bg-indigo-600 flex items-center justify-center text-white font-semibold">
                          {user.display_name?.[0]?.toUpperCase() || user.username[0].toUpperCase()}
                        </div>
                        <div>
                          <p className="font-medium text-gray-900">{user.display_name || user.username}</p>
                          <p className="text-sm text-gray-500">{user.email}</p>
                        </div>
                      </div>
                      <button
                        onClick={() => handleAddMember(user.id)}
                        disabled={addingMember === user.id}
                        className="btn-teams-primary text-sm px-3 py-1.5 disabled:opacity-50"
                      >
                        {addingMember === user.id ? (
                          <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                        ) : (
                          'Add'
                        )}
                      </button>
                    </div>
                  ))}
                </div>
              ) : userSearchQuery.trim().length >= 2 ? (
                <div className="text-center py-6 text-gray-500">
                  No users found matching "{userSearchQuery}"
                </div>
              ) : null}

              {/* Current Team Members */}
              {teamMembers.length > 0 && (
                <div>
                  <h4 className="text-sm font-medium text-gray-500 mb-2">
                    Current Members ({teamMembers.length})
                  </h4>
                  <div className="space-y-2">
                    {teamMembers.map((member) => (
                      <div
                        key={member.user_id || member.user?.id}
                        className="flex items-center gap-3 p-3 rounded-xl bg-slate-50"
                      >
                        <div className="w-10 h-10 rounded-full bg-emerald-500 flex items-center justify-center text-white font-semibold">
                          {member.user?.display_name?.[0]?.toUpperCase() ||
                           member.user?.username?.[0]?.toUpperCase() ||
                           'U'}
                        </div>
                        <div className="flex-1">
                          <p className="font-medium text-gray-900">
                            {member.user?.display_name || member.user?.username || 'Unknown User'}
                            {member.user_id === currentUser?.id && (
                              <span className="text-indigo-600 ml-2">(You)</span>
                            )}
                          </p>
                          <p className="text-sm text-gray-500">{member.role || 'member'}</p>
                        </div>
                        <Check size={18} className="text-emerald-500" />
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>

            <div className="p-6 border-t border-slate-200">
              <button
                onClick={() => {
                  setShowInviteModal(false);
                  setUserSearchQuery('');
                  setSearchedUsers([]);
                }}
                className="w-full btn-teams-ghost"
              >
                Done
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default TeamPage;
