import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useStore } from '../store';
import { apiClient } from '../api/client';

function TeamPage() {
  const { teamId } = useParams<{ teamId: string }>();
  const navigate = useNavigate();
  const { currentUser, teams, channels, setChannels } = useStore();
  const [loading, setLoading] = useState(false);
  const [showCreateChannel, setShowCreateChannel] = useState(false);

  const team = teams.find((t) => t.id === teamId);

  useEffect(() => {
    if (teamId) {
      loadChannels();
    }
  }, [teamId]);

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

  const handleCreateChannel = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!teamId) return;
    const formData = new FormData(e.currentTarget);
    const name = formData.get('name') as string;
    const description = formData.get('description') as string;

    try {
      await apiClient.createChannel({
        team_id: teamId,
        name,
        description,
      });
      setShowCreateChannel(false);
      loadChannels();
    } catch (error) {
      console.error('Failed to create channel:', error);
    }
  };

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Header */}
        <div className="mb-8">
          <button
            onClick={() => navigate('/')}
            className="text-blue-600 hover:text-blue-800 font-medium"
          >
            ← Back to Teams
          </button>
          <h1 className="text-3xl font-bold text-gray-900">{team?.name || 'Team'}</h1>
        </div>

        <div className="grid gap-8 lg:grid-cols-3">
          {/* Channels */}
          <div className="lg:col-span-2">
            <div className="bg-white shadow rounded-lg">
              <div className="px-6 py-4 border-b border-gray-200 flex justify-between items-center">
                <h2 className="text-lg font-semibold">Channels</h2>
                <button
                  onClick={() => setShowCreateChannel(!showCreateChannel)}
                  className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 text-sm font-medium"
                >
                  + New Channel
                </button>
              </div>

              {showCreateChannel && (
                <form onSubmit={handleCreateChannel} className="p-6 bg-gray-50 border-b border-gray-200">
                  <div>
                    <label htmlFor="name" className="block text-sm font-medium text-gray-700 mb-2">
                      Channel Name
                    </label>
                    <input
                      id="name"
                      name="name"
                      type="text"
                      required
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
                      placeholder="#general"
                    />
                  </div>
                  <div>
                    <label htmlFor="description" className="block text-sm font-medium text-gray-700 mb-2">
                      Description
                    </label>
                    <textarea
                      id="description"
                      name="description"
                      rows={3}
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
                      placeholder="What's this channel about?"
                    />
                  </div>
                  <div className="flex gap-2">
                    <button
                      type="submit"
                      className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
                    >
                      Create
                    </button>
                    <button
                      type="button"
                      onClick={() => setShowCreateChannel(false)}
                      className="px-4 py-2 border border-gray-300 rounded-md hover:bg-gray-50"
                    >
                      Cancel
                    </button>
                  </div>
                </form>
              )}

              <div className="divide-y divide-gray-200">
                {loading ? (
                  <div className="p-8 text-center text-gray-500">Loading channels...</div>
                ) : channels.length === 0 ? (
                  <div className="p-8 text-center text-gray-500">No channels yet</div>
                ) : (
                  channels.map((channel) => (
                    <div
                      key={channel.id}
                      onClick={() => navigate(`/chat/${channel.id}`)}
                      className="px-6 py-4 hover:bg-gray-50 cursor-pointer transition-colors"
                    >
                      <div className="flex items-center">
                        <div className="flex-shrink-0 h-10 w-10 bg-blue-100 rounded-lg flex items-center justify-center">
                          <span className="text-blue-600 font-bold text-lg">#</span>
                        </div>
                        <div className="ml-4">
                          <h3 className="text-lg font-semibold text-gray-900">{channel.name}</h3>
                          <p className="text-sm text-gray-600">
                            {channel.description || 'No description'}
                          </p>
                          <p className="text-xs text-gray-500 mt-1">
                            {channel.member_count} member{channel.member_count !== 1 ? 's' : ''}
                          </p>
                        </div>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>
          </div>

          {/* Team Info */}
          <div className="lg:col-span-1">
            <div className="bg-white shadow rounded-lg">
              <div className="px-6 py-4 border-b border-gray-200">
                <h2 className="text-lg font-semibold">Team Info</h2>
              </div>
              <div className="p-6 space-y-4">
                <div>
                  <h3 className="text-sm font-medium text-gray-500 uppercase tracking-wide">Description</h3>
                  <p className="mt-1 text-gray-900">{team?.description || 'No description'}</p>
                </div>
                <div>
                  <h3 className="text-sm font-medium text-gray-500 uppercase tracking-wide">Owner</h3>
                  <p className="mt-1 text-gray-900">{team?.owner_id === currentUser?.id ? 'You' : 'Another user'}</p>
                </div>
                <div>
                  <h3 className="text-sm font-medium text-gray-500 uppercase tracking-wide">Members</h3>
                  <p className="mt-1 text-gray-900">{team?.member_count || 0}</p>
                </div>
                <div>
                  <h3 className="text-sm font-medium text-gray-500 uppercase tracking-wide">Created</h3>
                  <p className="mt-1 text-gray-900">
                    {team?.created_at ? new Date(team.created_at).toLocaleDateString() : 'N/A'}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default TeamPage;
