import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useStore } from '../store';
import { apiClient } from '../api/client';

// Icons
const UserIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path fillRule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clipRule="evenodd" />
  </svg>
);

const LockIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path fillRule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clipRule="evenodd" />
  </svg>
);

const BellIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path d="M10 2a6 6 0 00-6 6v3.586l-.707.707A1 1 0 004 14h12a1 1 0 00.707-1.707L16 11.586V8a6 6 0 00-6-6zM10 18a3 3 0 01-3-3h6a3 3 0 01-3 3z" />
  </svg>
);

const ShieldIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path fillRule="evenodd" d="M2.166 4.999A11.954 11.954 0 0010 1.944 11.954 11.954 0 0017.834 5c.11.65.166 1.32.166 2.001 0 5.225-3.34 9.67-8 11.317C5.34 16.67 2 12.225 2 7c0-.682.057-1.35.166-2.001zm11.541 3.708a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
  </svg>
);

const ArrowLeftIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path fillRule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clipRule="evenodd" />
  </svg>
);

const CheckIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
  </svg>
);

type SettingsTab = 'profile' | 'security' | 'notifications' | 'privacy';
type UserStatus = 'online' | 'away' | 'busy' | 'offline';

function SettingsPage() {
  const navigate = useNavigate();
  const { currentUser, logout } = useStore();
  const [activeTab, setActiveTab] = useState<SettingsTab>('profile');
  const [displayName, setDisplayName] = useState(currentUser?.display_name || '');
  const [status, setStatus] = useState<UserStatus>((currentUser?.status as UserStatus) || 'offline');
  const [statusMessage, setStatusMessage] = useState(currentUser?.status_message || '');
  const [avatarUrl, setAvatarUrl] = useState(currentUser?.avatar_url || '');
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [successMessage, setSuccessMessage] = useState('');

  const handleUpdateProfile = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setLoading(true);

    try {
      await apiClient.updateCurrentUser({
        display_name: displayName,
        status,
        status_message: statusMessage,
        avatar_url: avatarUrl || undefined,
      });
      setSuccessMessage('Profile updated successfully!');
      setTimeout(() => setSuccessMessage(''), 3000);
    } catch (err: unknown) {
      const error = err as { response?: { data?: { error?: { message?: string } } } };
      setError(error.response?.data?.error?.message || 'Failed to update profile');
    } finally {
      setLoading(false);
    }
  };

  const handleChangePassword = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setLoading(true);

    if (newPassword !== confirmPassword) {
      setError('Passwords do not match');
      setLoading(false);
      return;
    }

    if (newPassword.length < 8) {
      setError('Password must be at least 8 characters');
      setLoading(false);
      return;
    }

    try {
      await apiClient.changePassword({
        current_password: currentPassword,
        new_password: newPassword,
      });
      setSuccessMessage('Password changed successfully!');
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
      setTimeout(() => setSuccessMessage(''), 3000);
    } catch (err: unknown) {
      const error = err as { response?: { data?: { error?: { message?: string } } } };
      setError(error.response?.data?.error?.message || 'Failed to change password');
    } finally {
      setLoading(false);
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

  const tabs = [
    { id: 'profile' as SettingsTab, label: 'Profile', icon: UserIcon },
    { id: 'security' as SettingsTab, label: 'Security', icon: LockIcon },
    { id: 'notifications' as SettingsTab, label: 'Notifications', icon: BellIcon },
    { id: 'privacy' as SettingsTab, label: 'Privacy', icon: ShieldIcon },
  ];

  const statusOptions: { value: UserStatus; label: string; color: string }[] = [
    { value: 'online', label: 'Available', color: 'bg-status-online' },
    { value: 'away', label: 'Away', color: 'bg-status-away' },
    { value: 'busy', label: 'Busy', color: 'bg-status-busy' },
    { value: 'offline', label: 'Appear offline', color: 'bg-status-offline' },
  ];

  return (
    <div className="min-h-screen bg-surface">
      {/* Header */}
      <header className="h-16 bg-white border-b border-gray-200 flex items-center px-6 shadow-sm">
        <button
          onClick={() => navigate('/')}
          className="p-2 hover:bg-gray-100 rounded-lg transition-colors mr-4"
          title="Back to Dashboard"
        >
          <ArrowLeftIcon />
        </button>
        <h1 className="text-xl font-semibold text-gray-900">Settings</h1>
      </header>

      <div className="max-w-5xl mx-auto px-4 py-8">
        <div className="flex flex-col md:flex-row gap-8">
          {/* Sidebar */}
          <aside className="w-full md:w-64 flex-shrink-0">
            {/* User Card */}
            <div className="card-teams mb-6">
              <div className="flex items-center gap-4">
                <div className="relative">
                  {avatarUrl ? (
                    <img
                      src={avatarUrl}
                      alt={displayName}
                      className="w-16 h-16 rounded-full object-cover"
                    />
                  ) : (
                    <div className="w-16 h-16 rounded-full bg-teams-purple flex items-center justify-center text-white text-xl font-semibold">
                      {getInitials(displayName || 'U')}
                    </div>
                  )}
                  <div className={`absolute bottom-0 right-0 w-4 h-4 rounded-full border-2 border-white ${
                    statusOptions.find(s => s.value === status)?.color || 'bg-status-offline'
                  }`}></div>
                </div>
                <div>
                  <h3 className="font-semibold text-gray-900">{displayName || 'User'}</h3>
                  <p className="text-sm text-gray-500">{currentUser?.email}</p>
                </div>
              </div>
            </div>

            {/* Navigation */}
            <nav className="space-y-1">
              {tabs.map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg text-left transition-colors ${
                    activeTab === tab.id
                      ? 'bg-teams-purple-50 text-teams-purple font-medium'
                      : 'text-gray-700 hover:bg-gray-100'
                  }`}
                >
                  <tab.icon />
                  {tab.label}
                </button>
              ))}
            </nav>
          </aside>

          {/* Main Content */}
          <main className="flex-1">
            {/* Success/Error Messages */}
            {successMessage && (
              <div className="mb-6 flex items-center gap-3 bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg animate-in">
                <CheckIcon />
                <span>{successMessage}</span>
              </div>
            )}

            {error && (
              <div className="mb-6 flex items-center gap-3 bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg animate-in">
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                </svg>
                <span>{error}</span>
              </div>
            )}

            {/* Profile Tab */}
            {activeTab === 'profile' && (
              <div className="space-y-6">
                <div className="card-teams">
                  <h2 className="text-lg font-semibold text-gray-900 mb-6">Profile Information</h2>
                  <form onSubmit={handleUpdateProfile} className="space-y-6">
                    {/* Avatar */}
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-3">
                        Profile Picture
                      </label>
                      <div className="flex items-center gap-6">
                        {avatarUrl ? (
                          <img
                            src={avatarUrl}
                            alt={displayName}
                            className="w-20 h-20 rounded-full object-cover"
                          />
                        ) : (
                          <div className="w-20 h-20 rounded-full bg-teams-purple flex items-center justify-center text-white text-2xl font-semibold">
                            {getInitials(displayName || 'U')}
                          </div>
                        )}
                        <div className="flex-1">
                          <input
                            type="url"
                            value={avatarUrl}
                            onChange={(e) => setAvatarUrl(e.target.value)}
                            className="input-teams"
                            placeholder="https://example.com/avatar.jpg"
                          />
                          <p className="text-xs text-gray-500 mt-1">Enter a URL for your profile picture</p>
                        </div>
                      </div>
                    </div>

                    {/* Display Name */}
                    <div>
                      <label htmlFor="displayName" className="block text-sm font-medium text-gray-700 mb-2">
                        Display Name
                      </label>
                      <input
                        id="displayName"
                        type="text"
                        value={displayName}
                        onChange={(e) => setDisplayName(e.target.value)}
                        className="input-teams"
                        placeholder="Your display name"
                      />
                    </div>

                    {/* Status */}
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-3">
                        Status
                      </label>
                      <div className="grid grid-cols-2 gap-3">
                        {statusOptions.map((option) => (
                          <button
                            key={option.value}
                            type="button"
                            onClick={() => setStatus(option.value)}
                            className={`flex items-center gap-3 px-4 py-3 rounded-lg border-2 transition-colors ${
                              status === option.value
                                ? 'border-teams-purple bg-teams-purple-50'
                                : 'border-gray-200 hover:border-gray-300'
                            }`}
                          >
                            <div className={`w-3 h-3 rounded-full ${option.color}`}></div>
                            <span className="font-medium text-gray-900">{option.label}</span>
                          </button>
                        ))}
                      </div>
                    </div>

                    {/* Status Message */}
                    <div>
                      <label htmlFor="statusMessage" className="block text-sm font-medium text-gray-700 mb-2">
                        Status Message
                      </label>
                      <input
                        id="statusMessage"
                        type="text"
                        value={statusMessage}
                        onChange={(e) => setStatusMessage(e.target.value)}
                        className="input-teams"
                        placeholder="What's on your mind?"
                      />
                    </div>

                    <div className="pt-4">
                      <button
                        type="submit"
                        disabled={loading}
                        className="btn-teams-primary w-full sm:w-auto disabled:opacity-50"
                      >
                        {loading ? 'Saving...' : 'Save Changes'}
                      </button>
                    </div>
                  </form>
                </div>
              </div>
            )}

            {/* Security Tab */}
            {activeTab === 'security' && (
              <div className="space-y-6">
                <div className="card-teams">
                  <h2 className="text-lg font-semibold text-gray-900 mb-6">Change Password</h2>
                  <form onSubmit={handleChangePassword} className="space-y-6">
                    <div>
                      <label htmlFor="currentPassword" className="block text-sm font-medium text-gray-700 mb-2">
                        Current Password
                      </label>
                      <input
                        id="currentPassword"
                        type="password"
                        required
                        value={currentPassword}
                        onChange={(e) => setCurrentPassword(e.target.value)}
                        className="input-teams"
                        placeholder="Enter current password"
                      />
                    </div>

                    <div>
                      <label htmlFor="newPassword" className="block text-sm font-medium text-gray-700 mb-2">
                        New Password
                      </label>
                      <input
                        id="newPassword"
                        type="password"
                        required
                        value={newPassword}
                        onChange={(e) => setNewPassword(e.target.value)}
                        className="input-teams"
                        placeholder="Enter new password"
                      />
                      <p className="text-xs text-gray-500 mt-1">Must be at least 8 characters</p>
                    </div>

                    <div>
                      <label htmlFor="confirmPassword" className="block text-sm font-medium text-gray-700 mb-2">
                        Confirm New Password
                      </label>
                      <input
                        id="confirmPassword"
                        type="password"
                        required
                        value={confirmPassword}
                        onChange={(e) => setConfirmPassword(e.target.value)}
                        className="input-teams"
                        placeholder="Confirm new password"
                      />
                    </div>

                    <div className="pt-4">
                      <button
                        type="submit"
                        disabled={loading}
                        className="btn-teams-primary w-full sm:w-auto disabled:opacity-50"
                      >
                        {loading ? 'Changing...' : 'Change Password'}
                      </button>
                    </div>
                  </form>
                </div>

                {/* Danger Zone */}
                <div className="card-teams border-2 border-red-200">
                  <h2 className="text-lg font-semibold text-red-600 mb-4">Danger Zone</h2>
                  <p className="text-sm text-gray-600 mb-6">
                    Once you sign out, you'll need to sign in again to access your account.
                  </p>
                  <button
                    onClick={() => {
                      logout();
                      navigate('/login');
                    }}
                    className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
                  >
                    Sign Out
                  </button>
                </div>
              </div>
            )}

            {/* Notifications Tab */}
            {activeTab === 'notifications' && (
              <div className="card-teams">
                <h2 className="text-lg font-semibold text-gray-900 mb-6">Notification Preferences</h2>
                <div className="space-y-6">
                  <div className="flex items-center justify-between py-3 border-b border-gray-100">
                    <div>
                      <h3 className="font-medium text-gray-900">Desktop Notifications</h3>
                      <p className="text-sm text-gray-500">Receive notifications on your desktop</p>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input type="checkbox" className="sr-only peer" defaultChecked />
                      <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-teams-purple-200 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-teams-purple"></div>
                    </label>
                  </div>

                  <div className="flex items-center justify-between py-3 border-b border-gray-100">
                    <div>
                      <h3 className="font-medium text-gray-900">Email Notifications</h3>
                      <p className="text-sm text-gray-500">Receive email updates about activity</p>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input type="checkbox" className="sr-only peer" />
                      <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-teams-purple-200 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-teams-purple"></div>
                    </label>
                  </div>

                  <div className="flex items-center justify-between py-3 border-b border-gray-100">
                    <div>
                      <h3 className="font-medium text-gray-900">Sound Notifications</h3>
                      <p className="text-sm text-gray-500">Play sounds for new messages</p>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input type="checkbox" className="sr-only peer" defaultChecked />
                      <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-teams-purple-200 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-teams-purple"></div>
                    </label>
                  </div>

                  <div className="flex items-center justify-between py-3">
                    <div>
                      <h3 className="font-medium text-gray-900">Mention Notifications</h3>
                      <p className="text-sm text-gray-500">Get notified when someone mentions you</p>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input type="checkbox" className="sr-only peer" defaultChecked />
                      <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-teams-purple-200 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-teams-purple"></div>
                    </label>
                  </div>
                </div>
              </div>
            )}

            {/* Privacy Tab */}
            {activeTab === 'privacy' && (
              <div className="card-teams">
                <h2 className="text-lg font-semibold text-gray-900 mb-6">Privacy Settings</h2>
                <div className="space-y-6">
                  <div className="flex items-center justify-between py-3 border-b border-gray-100">
                    <div>
                      <h3 className="font-medium text-gray-900">Show Online Status</h3>
                      <p className="text-sm text-gray-500">Let others see when you're online</p>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input type="checkbox" className="sr-only peer" defaultChecked />
                      <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-teams-purple-200 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-teams-purple"></div>
                    </label>
                  </div>

                  <div className="flex items-center justify-between py-3 border-b border-gray-100">
                    <div>
                      <h3 className="font-medium text-gray-900">Read Receipts</h3>
                      <p className="text-sm text-gray-500">Let others know when you've read their messages</p>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input type="checkbox" className="sr-only peer" defaultChecked />
                      <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-teams-purple-200 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-teams-purple"></div>
                    </label>
                  </div>

                  <div className="flex items-center justify-between py-3">
                    <div>
                      <h3 className="font-medium text-gray-900">Profile Visibility</h3>
                      <p className="text-sm text-gray-500">Control who can see your profile</p>
                    </div>
                    <select className="input-teams w-auto">
                      <option value="everyone">Everyone</option>
                      <option value="team">Team members only</option>
                      <option value="none">No one</option>
                    </select>
                  </div>
                </div>
              </div>
            )}
          </main>
        </div>
      </div>
    </div>
  );
}

export default SettingsPage;
