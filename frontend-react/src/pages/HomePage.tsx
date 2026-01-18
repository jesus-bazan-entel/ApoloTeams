import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useStore } from '../store';
import { apiClient } from '../api/client';

// Icons
const TeamsIcon = () => (
  <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
    <path d="M13 6a3 3 0 11-6 0 3 3 0 016 0zM18 8a2 2 0 11-4 0 2 2 0 014 0zM14 15a4 4 0 00-8 0v3h8v-3zM6 8a2 2 0 11-4 0 2 2 0 014 0zM16 18v-3a5.972 5.972 0 00-.75-2.906A3.005 3.005 0 0119 15v3h-3zM4.75 12.094A5.973 5.973 0 004 15v3H1v-3a3 3 0 013.75-2.906z" />
  </svg>
);

const ChatIcon = () => (
  <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
    <path fillRule="evenodd" d="M18 10c0 3.866-3.582 7-8 7a8.841 8.841 0 01-4.083-.98L2 17l1.338-3.123C2.493 12.767 2 11.434 2 10c0-3.866 3.582-7 8-7s8 3.134 8 7zM7 9H5v2h2V9zm8 0h-2v2h2V9zM9 9h2v2H9V9z" clipRule="evenodd" />
  </svg>
);

const SettingsIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path fillRule="evenodd" d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z" clipRule="evenodd" />
  </svg>
);

const LogoutIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path fillRule="evenodd" d="M3 3a1 1 0 00-1 1v12a1 1 0 102 0V4a1 1 0 00-1-1zm10.293 9.293a1 1 0 001.414 1.414l3-3a1 1 0 000-1.414l-3-3a1 1 0 10-1.414 1.414L14.586 9H7a1 1 0 100 2h7.586l-1.293 1.293z" clipRule="evenodd" />
  </svg>
);

const PlusIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path fillRule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clipRule="evenodd" />
  </svg>
);

function HomePage() {
  const navigate = useNavigate();
  const { currentUser, teams, setTeams, setSelectedTeam } = useStore();
  const [loading, setLoading] = useState(false);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newTeamName, setNewTeamName] = useState('');
  const [creating, setCreating] = useState(false);

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
    <div className="min-h-screen bg-surface flex">
      {/* Sidebar */}
      <aside className="w-16 bg-sidebar flex flex-col items-center py-4 gap-4">
        <div className="w-10 h-10 rounded-lg bg-teams-purple flex items-center justify-center text-white">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 3H5C3.9 3 3 3.9 3 5V19C3 20.1 3.9 21 5 21H19C20.1 21 21 20.1 21 19V5C21 3.9 20.1 3 19 3ZM19 19H5V5H19V19Z"/>
            <path d="M7 12H9V17H7V12ZM11 7H13V17H11V7ZM15 10H17V17H15V10Z"/>
          </svg>
        </div>
        
        <div className="flex-1 flex flex-col gap-2 mt-4">
          <button
            onClick={() => navigate('/chat')}
            className="w-10 h-10 rounded-lg hover:bg-sidebar-hover flex items-center justify-center text-sidebar-text-muted hover:text-white transition-colors"
            title="Chat"
          >
            <ChatIcon />
          </button>
          <button
            className="w-10 h-10 rounded-lg bg-sidebar-active flex items-center justify-center text-white"
            title="Teams"
          >
            <TeamsIcon />
          </button>
        </div>

        <div className="flex flex-col gap-2">
          <button
            onClick={() => navigate('/settings')}
            className="w-10 h-10 rounded-lg hover:bg-sidebar-hover flex items-center justify-center text-sidebar-text-muted hover:text-white transition-colors"
            title="Settings"
          >
            <SettingsIcon />
          </button>
          <button
            onClick={() => {
              useStore.getState().logout();
              navigate('/login');
            }}
            className="w-10 h-10 rounded-lg hover:bg-sidebar-hover flex items-center justify-center text-sidebar-text-muted hover:text-white transition-colors"
            title="Sign out"
          >
            <LogoutIcon />
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col">
        {/* Header */}
        <header className="h-16 bg-white border-b border-gray-200 flex items-center justify-between px-6 shadow-sm">
          <div className="flex items-center gap-4">
            <h1 className="text-xl font-semibold text-gray-900">Teams</h1>
          </div>
          <div className="flex items-center gap-4">
            <div className="relative">
              <div className="flex items-center gap-3">
                <div className="avatar-teams-md">
                  {currentUser?.display_name ? getInitials(currentUser.display_name) : 'U'}
                </div>
                <div className="hidden sm:block">
                  <p className="text-sm font-medium text-gray-900">{currentUser?.display_name || 'User'}</p>
                  <p className="text-xs text-gray-500">{currentUser?.email}</p>
                </div>
              </div>
            </div>
          </div>
        </header>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6">
          {/* Welcome Section */}
          <div className="mb-8">
            <h2 className="text-2xl font-bold text-gray-900 mb-2">
              Welcome back, {currentUser?.display_name?.split(' ')[0] || 'there'}! 👋
            </h2>
            <p className="text-gray-600">
              Select a team to start collaborating or create a new one.
            </p>
          </div>

          {/* Quick Actions */}
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
            <button
              onClick={() => setShowCreateModal(true)}
              className="card-teams flex items-center gap-4 hover:shadow-teams-lg transition-shadow group"
            >
              <div className="w-12 h-12 rounded-lg bg-teams-purple-50 flex items-center justify-center text-teams-purple group-hover:bg-teams-purple group-hover:text-white transition-colors">
                <PlusIcon />
              </div>
              <div className="text-left">
                <p className="font-medium text-gray-900">Create Team</p>
                <p className="text-sm text-gray-500">Start a new workspace</p>
              </div>
            </button>

            <button
              onClick={() => navigate('/chat')}
              className="card-teams flex items-center gap-4 hover:shadow-teams-lg transition-shadow group"
            >
              <div className="w-12 h-12 rounded-lg bg-emerald-50 flex items-center justify-center text-emerald-600 group-hover:bg-emerald-500 group-hover:text-white transition-colors">
                <ChatIcon />
              </div>
              <div className="text-left">
                <p className="font-medium text-gray-900">Go to Chat</p>
                <p className="text-sm text-gray-500">View all messages</p>
              </div>
            </button>
          </div>

          {/* Teams Grid */}
          <div className="mb-4 flex items-center justify-between">
            <h3 className="text-lg font-semibold text-gray-900">Your Teams</h3>
            <span className="text-sm text-gray-500">{teams.length} team{teams.length !== 1 ? 's' : ''}</span>
          </div>

          {loading ? (
            <div className="flex items-center justify-center py-16">
              <div className="flex flex-col items-center gap-4">
                <div className="w-12 h-12 border-4 border-teams-purple-200 border-t-teams-purple rounded-full animate-spin"></div>
                <p className="text-gray-500">Loading teams...</p>
              </div>
            </div>
          ) : teams.length === 0 ? (
            <div className="card-teams text-center py-16">
              <div className="w-20 h-20 mx-auto mb-6 rounded-full bg-teams-purple-50 flex items-center justify-center">
                <TeamsIcon />
              </div>
              <h3 className="text-xl font-semibold text-gray-900 mb-2">No teams yet</h3>
              <p className="text-gray-600 mb-6 max-w-md mx-auto">
                Create your first team to start collaborating with others. Teams help you organize conversations and work together.
              </p>
              <button
                onClick={() => setShowCreateModal(true)}
                className="btn-teams-primary"
              >
                <PlusIcon />
                <span className="ml-2">Create your first team</span>
              </button>
            </div>
          ) : (
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {teams.map((team, index) => (
                <div
                  key={team.id}
                  onClick={() => handleTeamClick(team.id)}
                  className="card-teams hover:shadow-teams-lg transition-all cursor-pointer group"
                >
                  <div className="flex items-start gap-4">
                    <div className={`w-14 h-14 rounded-lg ${getTeamColor(index)} flex items-center justify-center text-white font-bold text-lg flex-shrink-0`}>
                      {getInitials(team.name)}
                    </div>
                    <div className="flex-1 min-w-0">
                      <h4 className="font-semibold text-gray-900 group-hover:text-teams-purple transition-colors truncate">
                        {team.name}
                      </h4>
                      <p className="text-sm text-gray-500 mt-1">
                        {team.member_count} member{team.member_count !== 1 ? 's' : ''}
                      </p>
                      {team.description && (
                        <p className="text-sm text-gray-600 mt-2 truncate-2">
                          {team.description}
                        </p>
                      )}
                    </div>
                  </div>
                  <div className="mt-4 pt-4 border-t border-gray-100 flex items-center justify-between">
                    <span className="text-xs text-gray-400">
                      Created {new Date(team.created_at).toLocaleDateString()}
                    </span>
                    <svg className="w-5 h-5 text-gray-400 group-hover:text-teams-purple group-hover:translate-x-1 transition-all" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clipRule="evenodd" />
                    </svg>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </main>

      {/* Create Team Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 animate-in">
          <div className="bg-white rounded-lg shadow-teams-xl w-full max-w-md mx-4 animate-in">
            <div className="p-6 border-b border-gray-200">
              <h2 className="text-xl font-semibold text-gray-900">Create a new team</h2>
              <p className="text-sm text-gray-500 mt-1">Teams help you organize your work and collaborate with others.</p>
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
                    className="input-teams"
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
                  {creating ? 'Creating...' : 'Create team'}
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
