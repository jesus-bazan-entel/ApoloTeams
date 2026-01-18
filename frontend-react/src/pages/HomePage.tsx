import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useStore } from '../store';
import { apiClient } from '../api/client';

// Icons
const TeamsIcon = () => (
  <svg className="w-6 h-6" viewBox="0 0 24 24" fill="currentColor">
    <path d="M19.5 3h-15C3.12 3 2 4.12 2 5.5v13C2 19.88 3.12 21 4.5 21h15c1.38 0 2.5-1.12 2.5-2.5v-13C22 4.12 20.88 3 19.5 3zm-7 3c1.1 0 2 .9 2 2s-.9 2-2 2-2-.9-2-2 .9-2 2-2zm0 10c2.7 0 5.8-1.29 6-2H8c.22.71 3.3 2 6 2zm5-3h-8v3h4v4h3l-5 6v-5h-2v3h4v1h-6V9h8v4z"/>
  </svg>
);

const ChatIcon = () => (
  <svg className="w-6 h-6" viewBox="0 0 24 24" fill="currentColor">
    <path d="M20 2H4c-1.1 0-2 .9-2 2v18l4-4h14c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm0 14H6l-2 2V4h16v12z"/>
  </svg>
);

const SettingsIcon = () => (
  <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
    <path d="M19.14 12.94c.04-.31.06-.63.06-.94 0-.31-.02-.63-.06-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.04.31-.06.63-.06.94s.02.63.06.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/>
  </svg>
);

const LogoutIcon = () => (
  <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
    <path d="M17 7l-1.41 1.41L18.17 11H8v2h10.17l-2.58 2.58L17 17l5-5zM4 5h8V3H4c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h8v-2H4V5z"/>
  </svg>
);

const PlusIcon = () => (
  <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
    <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
  </svg>
);

const SearchIcon = () => (
  <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
    <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
  </svg>
);

const MenuIcon = () => (
  <svg className="w-6 h-6" viewBox="0 0 24 24" fill="currentColor">
    <path d="M3 18h18v-2H3v2zm0-5h18v-2H3v2zm0-7v2h18V6H3z"/>
  </svg>
);

function HomePage() {
  const navigate = useNavigate();
  const { currentUser, teams, setTeams, setSelectedTeamId } = useStore();
  const [loading, setLoading] = useState(false);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newTeamName, setNewTeamName] = useState('');
  const [creating, setCreating] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  useEffect(() => {
    loadTeams();
  }, []);

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
    setSelectedTeamId(teamId);
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
      'from-purple-500 to-indigo-600',
      'from-blue-500 to-cyan-500',
      'from-emerald-500 to-teal-500',
      'from-orange-500 to-amber-500',
      'from-pink-500 to-rose-500',
      'from-cyan-500 to-blue-500',
    ];
    return colors[index % colors.length];
  };

  const filteredTeams = teams.filter(team =>
    team.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-50 to-gray-100 flex">
      {/* Mobile Sidebar Overlay */}
      {mobileMenuOpen && (
        <div 
          className="fixed inset-0 bg-black/20 z-40 lg:hidden"
          onClick={() => setMobileMenuOpen(false)}
        />
      )}

      {/* Sidebar - Hidden on mobile unless opened */}
      <aside className={`
        fixed lg:relative inset-y-0 left-0 z-50
        w-16 bg-gradient-to-b from-gray-900 to-gray-800 
        flex flex-col items-center py-4 gap-4
        transform transition-transform duration-300 ease-in-out
        ${mobileMenuOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}
      `}>
        {/* Logo */}
        <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-teams-purple to-teams-blue flex items-center justify-center text-white shadow-lg">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19.5 3h-15C3.12 3 2 4.12 2 5.5v13C2 19.88 3.12 21 4.5 21h15c1.38 0 2.5-1.12 2.5-2.5v-13C22 4.12 20.88 3 19.5 3z"/>
          </svg>
        </div>
        
        <div className="w-8 h-px bg-gray-700 my-2" />
        
        <div className="flex-1 flex flex-col gap-2">
          <button
            onClick={() => navigate('/chat')}
            className="sidebar-item group"
            title="Chat"
          >
            <ChatIcon />
          </button>
          <div className="sidebar-item-active" title="Teams">
            <TeamsIcon />
          </div>
        </div>

        <div className="w-8 h-px bg-gray-700 my-2" />

        <div className="flex flex-col gap-2">
          <button
            onClick={() => navigate('/settings')}
            className="sidebar-item"
            title="Settings"
          >
            <SettingsIcon />
          </button>
          <button
            onClick={() => {
              useStore.getState().logout();
              navigate('/login');
            }}
            className="sidebar-item text-red-400 hover:text-red-300 hover:bg-red-500/10"
            title="Sign out"
          >
            <LogoutIcon />
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col min-h-screen">
        {/* Header */}
        <header className="bg-white/80 backdrop-blur-md border-b border-gray-200 px-4 lg:px-6 py-4 sticky top-0 z-30">
          <div className="flex items-center justify-between gap-4">
            {/* Mobile menu button */}
            <button 
              onClick={() => setMobileMenuOpen(true)}
              className="lg:hidden btn-teams-icon"
            >
              <MenuIcon />
            </button>

            {/* Search */}
            <div className="flex-1 max-w-xl">
              <div className="relative">
                <SearchIcon />
                <input
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Search teams..."
                  className="w-full pl-10 pr-4 py-2.5 bg-gray-100 border-0 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-teams-purple/20 focus:bg-white transition-all"
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
                <PlusIcon />
                <span className="hidden sm:inline">New Team</span>
              </button>
            </div>
          </div>
        </header>

        {/* Content */}
        <div className="flex-1 p-4 lg:p-6 overflow-y-auto">
          {/* Welcome Banner */}
          <div className="bg-gradient-to-r from-teams-purple to-teams-blue rounded-2xl p-6 lg:p-8 mb-6 lg:mb-8 text-white">
            <h1 className="text-2xl lg:text-3xl font-bold mb-2">
              Welcome back, {currentUser?.display_name?.split(' ')[0] || 'there'}! 👋
            </h1>
            <p className="text-white/80 text-lg">
              Ready to collaborate with your team?
            </p>
          </div>

          {/* Quick Stats */}
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6 lg:mb-8">
            <div className="card-teams p-4">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-lg bg-teams-purple-100 flex items-center justify-center">
                  <TeamsIcon />
                </div>
                <div>
                  <p className="text-2xl font-bold text-gray-900">{teams.length}</p>
                  <p className="text-sm text-gray-500">Teams</p>
                </div>
              </div>
            </div>
            <div className="card-teams p-4">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-lg bg-blue-100 flex items-center justify-center">
                  <ChatIcon />
                </div>
                <div>
                  <p className="text-2xl font-bold text-gray-900">12</p>
                  <p className="text-sm text-gray-500">Messages</p>
                </div>
              </div>
            </div>
          </div>

          {/* Teams Section */}
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-xl font-semibold text-gray-900">Your Teams</h2>
            <span className="badge-primary">{filteredTeams.length} total</span>
          </div>

          {loading ? (
            <div className="flex items-center justify-center py-16">
              <div className="flex flex-col items-center gap-4">
                <div className="w-12 h-12 border-4 border-teams-purple-200 border-t-teams-purple rounded-full animate-spin"></div>
                <p className="text-gray-500">Loading teams...</p>
              </div>
            </div>
          ) : filteredTeams.length === 0 ? (
            <div className="card-teams text-center py-12 px-6">
              <div className="w-16 h-16 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-teams-purple-100 to-teams-blue-100 flex items-center justify-center">
                <TeamsIcon />
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
                  <PlusIcon />
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
                      <h3 className="font-semibold text-gray-900 group-hover:text-teams-purple transition-colors truncate">
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
                  <div className="pt-4 border-t border-gray-100 flex items-center justify-between">
                    <span className="text-xs text-gray-400">
                      Created {new Date(team.created_at).toLocaleDateString()}
                    </span>
                    <div className="w-8 h-8 rounded-full bg-gray-100 flex items-center justify-center text-gray-400 group-hover:bg-teams-purple group-hover:text-white transition-all">
                      <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
                      </svg>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </main>

      {/* Create Team Modal */}
      {showCreateModal && (
        <div className="modal-overlay" onClick={() => setShowCreateModal(false)}>
          <div className="modal-content" onClick={e => e.stopPropagation()}>
            <div className="p-6 border-b border-gray-100">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-teams-purple to-teams-blue flex items-center justify-center text-white">
                  <PlusIcon />
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
