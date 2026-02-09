import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useStore } from '../store';
import { apiClient } from '../api/client';
import { useWebRTC } from '../hooks/useWebRTC';
import type { User, CallType } from '../types';
import { Plus, Search, Users, Phone, Video, MessageSquare } from 'lucide-react';

function HomePage() {
  const navigate = useNavigate();
  const { currentUser, teams, setTeams, setSelectedTeam } = useStore();
  const { startDirectCall } = useWebRTC();
  const [loading, setLoading] = useState(false);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newTeamName, setNewTeamName] = useState('');
  const [creating, setCreating] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [activeTab, setActiveTab] = useState<'teams' | 'users'>('teams');
  const [userSearchQuery, setUserSearchQuery] = useState('');
  const [searchedUsers, setSearchedUsers] = useState<User[]>([]);
  const [searchingUsers, setSearchingUsers] = useState(false);
  const [callingUser, setCallingUser] = useState<string | null>(null);

  const handleStartCall = async (userId: string, callType: CallType) => {
    try {
      setCallingUser(userId);
      await startDirectCall(userId, callType);
    } catch (error) {
      console.error('Failed to start call:', error);
    } finally {
      setCallingUser(null);
    }
  };

  useEffect(() => {
    loadTeams();
  }, []);

  useEffect(() => {
    const searchUsers = async () => {
      if (userSearchQuery.trim().length < 2) {
        setSearchedUsers([]);
        return;
      }
      setSearchingUsers(true);
      try {
        const users = await apiClient.searchUsers(userSearchQuery);
        setSearchedUsers(users.filter(u => u.id !== currentUser?.id));
      } catch (error) {
        console.error('Failed to search users:', error);
      } finally {
        setSearchingUsers(false);
      }
    };

    const debounceTimer = setTimeout(searchUsers, 300);
    return () => clearTimeout(debounceTimer);
  }, [userSearchQuery, currentUser?.id]);

  const handleStartDirectMessage = async (userId: string) => {
    try {
      // Create or get existing direct message channel
      const channel = await apiClient.createDmChannel(userId);
      navigate(`/chat/${channel.id}`);
    } catch (error) {
      console.error('Failed to create DM:', error);
    }
  };

  const loadTeams = async () => {
    setLoading(true);
    try {
      const teamsData = await apiClient.listTeams();
      setTeams(teamsData);
    } catch (error) {
      console.error('Failed to load teams:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleTeamClick = (teamId: string) => {
    setSelectedTeam(teamId);
    navigate(`/teams/${teamId}`);
  };

  const handleCreateTeam = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newTeamName.trim()) return;

    setCreating(true);
    try {
      await apiClient.createTeam({ name: newTeamName.trim() });
      setNewTeamName('');
      setShowCreateModal(false);
      loadTeams();
    } catch (error) {
      console.error('Failed to create team:', error);
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

  const getTeamColor = (index: number) => {
    const colors = [
      'from-indigo-500 to-indigo-700',
      'from-cyan-500 to-teal-500',
      'from-emerald-500 to-green-600',
      'from-amber-500 to-orange-500',
      'from-rose-500 to-pink-600',
      'from-violet-500 to-purple-600',
    ];
    return colors[index % colors.length];
  };

  const filteredTeams = teams.filter(team =>
    team.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="h-full bg-gradient-to-br from-slate-50 to-slate-100 flex flex-col overflow-hidden">
      {/* Main Content */}
      <div className="flex-1 flex flex-col min-h-0">
        {/* Header */}
        <header className="bg-white/80 backdrop-blur-md border-b border-slate-200 px-4 lg:px-6 py-4 sticky top-0 z-30">
          <div className="flex items-center justify-between gap-4">
            {/* Search */}
            <div className="flex-1 max-w-xl">
              <div className="relative">
                <Search className="w-5 h-5 absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
                <input
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Search teams..."
                  className="w-full pl-10 pr-4 py-2.5 bg-slate-100 border-0 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:bg-white transition-all"
                />
              </div>
            </div>

            {/* User */}
            <div className="flex items-center gap-3">
              <div className="hidden sm:flex items-center gap-3">
                <div className="avatar-teams-md">
                  {currentUser?.display_name ? getInitials(currentUser.display_name) : 'U'}
                </div>
                <div className="hidden md:block text-right">
                  <p className="text-sm font-medium text-gray-900">{currentUser?.display_name || 'User'}</p>
                  <p className="text-xs text-gray-500">{currentUser?.email}</p>
                </div>
              </div>
              <button
                onClick={() => setShowCreateModal(true)}
                className="btn-teams-primary"
              >
                <Plus className="w-5 h-5" />
                <span className="hidden sm:inline">New Team</span>
              </button>
            </div>
          </div>
        </header>

        {/* Content */}
        <div className="flex-1 p-4 lg:p-6 overflow-y-auto">
          {/* Welcome Banner */}
          <div className="bg-gradient-to-r from-indigo-600 to-cyan-500 rounded-2xl p-6 lg:p-8 mb-6 lg:mb-8 text-white">
            <h1 className="text-2xl lg:text-3xl font-bold mb-2">
              Welcome back, {currentUser?.display_name?.split(' ')[0] || 'there'}! 👋
            </h1>
            <p className="text-white/80 text-lg">
              Ready to collaborate with your team?
            </p>
          </div>

          {/* Quick Stats */}
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6 lg:mb-8">
            <div
              className={`card-teams cursor-pointer transition-all ${activeTab === 'teams' ? 'ring-2 ring-indigo-500' : ''}`}
              onClick={() => setActiveTab('teams')}
            >
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-lg bg-indigo-100 flex items-center justify-center text-indigo-600">
                  <Users className="w-6 h-6" />
                </div>
                <div>
                  <p className="text-2xl font-bold text-gray-900">{teams.length}</p>
                  <p className="text-sm text-gray-500">Teams</p>
                </div>
              </div>
            </div>
            <div
              className={`card-teams cursor-pointer transition-all ${activeTab === 'users' ? 'ring-2 ring-indigo-500' : ''}`}
              onClick={() => setActiveTab('users')}
            >
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-lg bg-green-100 flex items-center justify-center text-green-600">
                  <Users className="w-6 h-6" />
                </div>
                <div>
                  <p className="text-2xl font-bold text-gray-900">Users</p>
                  <p className="text-sm text-gray-500">Find contacts</p>
                </div>
              </div>
            </div>
            <div
              className="card-teams cursor-pointer transition-all hover:ring-2 hover:ring-cyan-500"
              onClick={() => navigate('/chat')}
            >
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-lg bg-cyan-100 flex items-center justify-center text-cyan-600">
                  <MessageSquare className="w-6 h-6" />
                </div>
                <div>
                  <p className="text-2xl font-bold text-gray-900">Chat</p>
                  <p className="text-sm text-gray-500">Messages</p>
                </div>
              </div>
            </div>
          </div>

          {/* Users Search Section */}
          {activeTab === 'users' && (
            <div className="mb-6">
              <div className="card-teams p-6">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">Search Users</h3>
                <div className="relative mb-4">
                  <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                    <Search className="w-5 h-5 text-gray-400" />
                  </div>
                  <input
                    type="text"
                    value={userSearchQuery}
                    onChange={(e) => setUserSearchQuery(e.target.value)}
                    placeholder="Search by name, username or email..."
                    className="w-full pl-10 pr-4 py-3 bg-slate-100 border-0 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:bg-white transition-all"
                    autoFocus
                  />
                </div>

                {searchingUsers ? (
                  <div className="flex items-center justify-center py-8">
                    <div className="w-8 h-8 border-4 border-indigo-200 border-t-indigo-600 rounded-full animate-spin"></div>
                  </div>
                ) : searchedUsers.length > 0 ? (
                  <div className="space-y-2">
                    {searchedUsers.map((user) => (
                      <div
                        key={user.id}
                        className="flex items-center justify-between p-3 rounded-xl hover:bg-slate-50 transition-colors"
                      >
                        <div className="flex items-center gap-3">
                          <div className="w-10 h-10 rounded-full bg-gradient-to-br from-indigo-600 to-cyan-500 flex items-center justify-center text-white font-medium">
                            {getInitials(user.display_name)}
                          </div>
                          <div>
                            <p className="font-medium text-gray-900">{user.display_name}</p>
                            <p className="text-sm text-gray-500">@{user.username}</p>
                          </div>
                        </div>
                        <div className="flex items-center gap-2">
                          <button
                            onClick={() => handleStartCall(user.id, 'audio')}
                            disabled={callingUser === user.id}
                            className="p-2 text-gray-600 hover:bg-gray-100 hover:text-indigo-600 rounded-lg transition-colors disabled:opacity-50"
                            title="Audio call"
                          >
                            <Phone className="w-5 h-5" />
                          </button>
                          <button
                            onClick={() => handleStartCall(user.id, 'video')}
                            disabled={callingUser === user.id}
                            className="p-2 text-gray-600 hover:bg-gray-100 hover:text-indigo-600 rounded-lg transition-colors disabled:opacity-50"
                            title="Video call"
                          >
                            <Video className="w-5 h-5" />
                          </button>
                          <button
                            onClick={() => handleStartDirectMessage(user.id)}
                            className="flex items-center gap-2 px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors"
                          >
                            <MessageSquare className="w-5 h-5" />
                            <span>Message</span>
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                ) : userSearchQuery.length >= 2 ? (
                  <div className="text-center py-8 text-gray-500">
                    No users found matching "{userSearchQuery}"
                  </div>
                ) : (
                  <div className="text-center py-8 text-gray-500">
                    Type at least 2 characters to search for users
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Teams Section */}
          {activeTab === 'teams' && (
          <>
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-xl font-semibold text-gray-900">Your Teams</h2>
            <span className="badge-primary">{filteredTeams.length} total</span>
          </div>

          {loading ? (
            <div className="flex items-center justify-center py-16">
              <div className="flex flex-col items-center gap-4">
                <div className="w-12 h-12 border-4 border-indigo-200 border-t-indigo-600 rounded-full animate-spin"></div>
                <p className="text-gray-500">Loading teams...</p>
              </div>
            </div>
          ) : filteredTeams.length === 0 ? (
            <div className="card-teams text-center py-12 px-6">
              <div className="w-16 h-16 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-indigo-100 to-cyan-100 flex items-center justify-center text-indigo-600">
                <Users className="w-6 h-6" />
              </div>
              <h3 className="text-xl font-semibold text-gray-900 mb-2">
                {searchQuery ? 'No teams found' : 'No teams yet'}
              </h3>
              <p className="text-gray-600 mb-6 max-w-md mx-auto">
                {searchQuery
                  ? `No teams matching "${searchQuery}"`
                  : 'Create your first team to start collaborating with others.'}
              </p>
              {!searchQuery && (
                <button
                  onClick={() => setShowCreateModal(true)}
                  className="btn-teams-primary"
                >
                  <Plus className="w-5 h-5" />
                  Create your first team
                </button>
              )}
            </div>
          ) : (
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {filteredTeams.map((team, index) => (
                <div
                  key={team.id}
                  onClick={() => handleTeamClick(team.id)}
                  className="card-teams-hover p-5 cursor-pointer group"
                >
                  {/* Header */}
                  <div className="flex items-start gap-4 mb-4">
                    <div className={`w-14 h-14 rounded-xl bg-gradient-to-br ${getTeamColor(index)} flex items-center justify-center text-white font-bold text-lg shadow-md flex-shrink-0`}>
                      {getInitials(team.name)}
                    </div>
                    <div className="flex-1 min-w-0">
                      <h3 className="font-semibold text-gray-900 group-hover:text-indigo-600 transition-colors truncate">
                        {team.name}
                      </h3>
                      <p className="text-sm text-gray-500">
                        {team.member_count || 0} member{(team.member_count || 0) !== 1 ? 's' : ''}
                      </p>
                    </div>
                  </div>

                  {/* Description */}
                  {team.description && (
                    <p className="text-sm text-gray-600 mb-4 truncate-2">
                      {team.description}
                    </p>
                  )}

                  {/* Footer */}
                  <div className="pt-4 border-t border-slate-100 flex items-center justify-between">
                    <span className="text-xs text-gray-400">
                      Created {new Date(team.created_at).toLocaleDateString()}
                    </span>
                    <div className="w-8 h-8 rounded-full bg-slate-100 flex items-center justify-center text-slate-400 group-hover:bg-indigo-600 group-hover:text-white transition-all">
                      <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
                      </svg>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
          </>
          )}
        </div>
      </div>

      {/* Create Team Modal */}
      {showCreateModal && (
        <div className="modal-overlay" onClick={() => setShowCreateModal(false)}>
          <div className="modal-content" onClick={e => e.stopPropagation()}>
            <div className="p-6 border-b border-slate-100">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-indigo-600 to-cyan-500 flex items-center justify-center text-white">
                  <Plus className="w-5 h-5" />
                </div>
                <div>
                  <h2 className="text-xl font-semibold text-gray-900">Create new team</h2>
                  <p className="text-sm text-gray-500">Start collaborating with your team</p>
                </div>
              </div>
            </div>
            <form onSubmit={handleCreateTeam} className="p-6">
              <div className="space-y-4">
                <div>
                  <label htmlFor="teamName" className="block text-sm font-medium text-gray-700 mb-2">
                    Team name
                  </label>
                  <input
                    id="teamName"
                    type="text"
                    value={newTeamName}
                    onChange={(e) => setNewTeamName(e.target.value)}
                    className="input-teams-lg"
                    placeholder="e.g., Marketing Team"
                    autoFocus
                  />
                </div>
              </div>
              <div className="flex justify-end gap-3 mt-6">
                <button
                  type="button"
                  onClick={() => setShowCreateModal(false)}
                  className="btn-teams-ghost"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={creating || !newTeamName.trim()}
                  className="btn-teams-primary disabled:opacity-50"
                >
                  {creating ? (
                    <>
                      <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                      Creating...
                    </>
                  ) : (
                    'Create team'
                  )}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

    </div>
  );
}

export default HomePage;
