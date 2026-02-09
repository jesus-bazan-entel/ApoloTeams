import { useEffect, useState, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useStore } from '../store';
import { apiClient } from '../api/client';
import { useChannel } from '../hooks';
import { useWebRTC } from '../hooks/useWebRTC';
import { DirectCallButton } from '../components/call';
import type { Message, Channel, CallType } from '../types';
import { Hash, Lock, Search, Send, Smile, Paperclip, MoreHorizontal, Phone, Video, MessageSquare, Users } from 'lucide-react';

function ChatPage() {
  const { channelId } = useParams<{ channelId: string }>();
  const navigate = useNavigate();
  const { currentUser, setSelectedChannel, channels, setChannels } = useStore();
  const [newMessage, setNewMessage] = useState('');
  const [sidebarSearch, setSidebarSearch] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const { startCall } = useWebRTC();

  // Use the useChannel hook for encapsulated channel logic
  const {
    messages: currentMessages,
    loading,
    error,
    typingUsers,
    sendMessage,
    startTyping,
    stopTyping,
  } = useChannel(channelId);

  const handleStartCall = async (callType: CallType) => {
    if (!channelId) return;
    try {
      await startCall(channelId, callType);
    } catch (error) {
      console.error('Failed to start call:', error);
    }
  };

  useEffect(() => {
    loadChannels();
  }, []);

  useEffect(() => {
    if (channelId) {
      setSelectedChannel(channelId);
    }
  }, [channelId, setSelectedChannel]);

  useEffect(() => {
    scrollToBottom();
  }, [currentMessages]);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  const loadChannels = async () => {
    try {
      // Load all channels for the user (includes DMs and team channels)
      const channelsData = await apiClient.listChannels();
      setChannels(channelsData);
    } catch (error) {
      console.error('Failed to load channels:', error);
    }
  };

  const handleSendMessage = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newMessage.trim() || !channelId) return;

    try {
      await sendMessage(newMessage);
      setNewMessage('');
      inputRef.current?.focus();
    } catch (error) {
      console.error('Failed to send message:', error);
    }
  };

  // Handle typing indicator
  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setNewMessage(e.target.value);
    if (e.target.value) {
      startTyping();
    } else {
      stopTyping();
    }
  };
  const currentChannel = channels.find((c: Channel) => c.id === channelId);

  const getInitials = (name: string) => {
    return name
      .split(' ')
      .map(word => word[0])
      .join('')
      .toUpperCase()
      .slice(0, 2);
  };

  const formatTime = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    const today = new Date();
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);

    if (date.toDateString() === today.toDateString()) {
      return 'Today';
    } else if (date.toDateString() === yesterday.toDateString()) {
      return 'Yesterday';
    }
    return date.toLocaleDateString([], { weekday: 'long', month: 'short', day: 'numeric' });
  };

  const groupMessagesByDate = (msgs: Message[]) => {
    const groups: { [key: string]: Message[] } = {};
    msgs.forEach(msg => {
      const dateKey = new Date(msg.created_at).toDateString();
      if (!groups[dateKey]) {
        groups[dateKey] = [];
      }
      groups[dateKey].push(msg);
    });
    return groups;
  };

  const messageGroups = groupMessagesByDate(currentMessages);

  return (
    <div className="flex h-full bg-surface">
      {/* Sidebar - Conversations */}
      <aside className="w-72 bg-white border-r border-slate-200 flex flex-col flex-shrink-0">
        {/* Sidebar Header */}
        <div className="h-16 px-4 flex items-center justify-between border-b border-slate-200">
          <h2 className="font-semibold text-gray-900">Conversaciones</h2>
        </div>

        {/* Search */}
        <div className="p-3">
          <div className="relative">
            <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <Search className="w-4 h-4 text-gray-400" />
            </div>
            <input
              type="text"
              value={sidebarSearch}
              onChange={(e) => setSidebarSearch(e.target.value)}
              placeholder="Buscar conversaciones..."
              className="w-full pl-9 pr-4 py-2 text-sm bg-slate-100 border-0 rounded-xl focus:ring-2 focus:ring-indigo-500/20 focus:bg-white transition-all"
            />
          </div>
        </div>

        {/* Direct Call Button */}
        <div className="px-3 pb-2">
          <DirectCallButton className="w-full justify-center" />
        </div>

        {/* Channel List - Priority: DMs > Teams > Channels */}
        <div className="flex-1 overflow-y-auto">
          {channels.length === 0 ? (
            <div className="p-4 text-center text-gray-500 text-sm">
              <p className="mb-4">Sin conversaciones</p>
              <button
                onClick={() => navigate('/')}
                className="text-indigo-600 hover:underline"
              >
                Buscar usuarios o crear un equipo
              </button>
            </div>
          ) : (
            <div className="py-1">
              {/* 1. Direct Messages (highest priority) */}
              {channels
                .filter((c: Channel) => c.channel_type === 'direct_message')
                .filter((c: Channel) => !sidebarSearch || c.name.toLowerCase().includes(sidebarSearch.toLowerCase()))
                .length > 0 && (
                <>
                  <div className="px-4 py-2 text-[11px] font-semibold text-gray-400 uppercase tracking-wider">
                    Mensajes Directos
                  </div>
                  {channels
                    .filter((c: Channel) => c.channel_type === 'direct_message')
                    .filter((c: Channel) => !sidebarSearch || c.name.toLowerCase().includes(sidebarSearch.toLowerCase()))
                    .map((channel: Channel) => {
                      const displayName = channel.name.replace(/^(dm-|DM:\s*)/i, '');
                      const isSelected = channel.id === channelId;
                      return (
                        <button
                          key={channel.id}
                          onClick={() => navigate(`/chat/${channel.id}`)}
                          className={`w-full flex items-center gap-3 px-4 py-2.5 text-left transition-all ${
                            isSelected
                              ? 'bg-indigo-50 text-indigo-700'
                              : 'hover:bg-slate-50 text-gray-700'
                          }`}
                        >
                          <div className="relative flex-shrink-0">
                            <div className={`w-8 h-8 rounded-full flex items-center justify-center text-xs font-semibold ${
                              isSelected
                                ? 'bg-indigo-600 text-white'
                                : 'bg-gradient-to-br from-indigo-500 to-cyan-400 text-white'
                            }`}>
                              {getInitials(displayName)}
                            </div>
                            <div className="absolute -bottom-0.5 -right-0.5 w-3 h-3 bg-green-400 rounded-full border-2 border-white" />
                          </div>
                          <div className="flex-1 min-w-0">
                            <span className={`block truncate text-sm ${isSelected ? 'font-semibold' : 'font-medium'}`}>
                              {displayName}
                            </span>
                            {channel.last_message && (
                              <span className="block truncate text-xs text-gray-400 mt-0.5">
                                {channel.last_message.content}
                              </span>
                            )}
                          </div>
                          {channel.unread_count > 0 && (
                            <span className="flex-shrink-0 bg-indigo-600 text-white text-[10px] font-bold min-w-[20px] h-5 flex items-center justify-center px-1.5 rounded-full">
                              {channel.unread_count}
                            </span>
                          )}
                        </button>
                      );
                    })}
                </>
              )}

              {/* 2. Team Channels (grouped by team) */}
              {channels
                .filter((c: Channel) => c.channel_type !== 'direct_message' && c.team_id)
                .filter((c: Channel) => !sidebarSearch || c.name.toLowerCase().includes(sidebarSearch.toLowerCase()))
                .length > 0 && (
                <>
                  <div className="px-4 py-2 mt-3 text-[11px] font-semibold text-gray-400 uppercase tracking-wider">
                    Equipos
                  </div>
                  {channels
                    .filter((c: Channel) => c.channel_type !== 'direct_message' && c.team_id)
                    .filter((c: Channel) => !sidebarSearch || c.name.toLowerCase().includes(sidebarSearch.toLowerCase()))
                    .map((channel: Channel) => {
                      const isSelected = channel.id === channelId;
                      return (
                        <button
                          key={channel.id}
                          onClick={() => navigate(`/chat/${channel.id}`)}
                          className={`w-full flex items-center gap-3 px-4 py-2 text-left transition-all ${
                            isSelected
                              ? 'bg-indigo-50 text-indigo-700'
                              : 'hover:bg-slate-50 text-gray-700'
                          }`}
                        >
                          <div className={`w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0 ${
                            isSelected
                              ? 'bg-indigo-100 text-indigo-600'
                              : 'bg-slate-100 text-slate-500'
                          }`}>
                            <Users className="w-4 h-4" />
                          </div>
                          <span className={`flex-1 truncate text-sm ${isSelected ? 'font-semibold' : 'font-medium'}`}>
                            {channel.name}
                          </span>
                          {channel.unread_count > 0 && (
                            <span className="flex-shrink-0 bg-indigo-600 text-white text-[10px] font-bold min-w-[20px] h-5 flex items-center justify-center px-1.5 rounded-full">
                              {channel.unread_count}
                            </span>
                          )}
                        </button>
                      );
                    })}
                </>
              )}

              {/* 3. General Channels (no team) */}
              {channels
                .filter((c: Channel) => c.channel_type !== 'direct_message' && !c.team_id)
                .filter((c: Channel) => !sidebarSearch || c.name.toLowerCase().includes(sidebarSearch.toLowerCase()))
                .length > 0 && (
                <>
                  <div className="px-4 py-2 mt-3 text-[11px] font-semibold text-gray-400 uppercase tracking-wider">
                    Canales
                  </div>
                  {channels
                    .filter((c: Channel) => c.channel_type !== 'direct_message' && !c.team_id)
                    .filter((c: Channel) => !sidebarSearch || c.name.toLowerCase().includes(sidebarSearch.toLowerCase()))
                    .map((channel: Channel) => {
                      const isSelected = channel.id === channelId;
                      return (
                        <button
                          key={channel.id}
                          onClick={() => navigate(`/chat/${channel.id}`)}
                          className={`w-full flex items-center gap-3 px-4 py-2 text-left transition-all ${
                            isSelected
                              ? 'bg-indigo-50 text-indigo-700'
                              : 'hover:bg-slate-50 text-gray-700'
                          }`}
                        >
                          <div className={`w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0 ${
                            isSelected
                              ? 'bg-indigo-100 text-indigo-600'
                              : 'bg-slate-100 text-slate-500'
                          }`}>
                            {channel.channel_type === 'private' ? <Lock className="w-4 h-4" /> : <Hash className="w-4 h-4" />}
                          </div>
                          <span className={`flex-1 truncate text-sm ${isSelected ? 'font-semibold' : 'font-medium'}`}>
                            {channel.name}
                          </span>
                          {channel.unread_count > 0 && (
                            <span className="flex-shrink-0 bg-indigo-600 text-white text-[10px] font-bold min-w-[20px] h-5 flex items-center justify-center px-1.5 rounded-full">
                              {channel.unread_count}
                            </span>
                          )}
                        </button>
                      );
                    })}
                </>
              )}
            </div>
          )}
        </div>
      </aside>

      {/* Main Chat Area */}
      <main className="flex-1 flex flex-col">
        {/* Chat Header */}
        <header className="h-16 bg-white border-b border-slate-200 px-6 flex items-center justify-between shadow-sm">
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2 text-gray-900">
              {currentChannel?.channel_type === 'direct_message' ? (
                <div className="w-8 h-8 rounded-full bg-gradient-to-br from-indigo-500 to-cyan-400 flex items-center justify-center text-white text-xs font-semibold">
                  {getInitials(currentChannel.name.replace(/^(dm-|DM:\s*)/i, ''))}
                </div>
              ) : currentChannel?.team_id ? (
                <Users className="w-5 h-5 text-indigo-600" />
              ) : (
                <Hash className="w-5 h-5" />
              )}
              <h1 className="text-lg font-semibold">
                {currentChannel
                  ? currentChannel.channel_type === 'direct_message'
                    ? currentChannel.name.replace(/^(dm-|DM:\s*)/i, '')
                    : currentChannel.name
                  : 'Seleccionar conversacion'}
              </h1>
            </div>
            {currentChannel?.description && (
              <span className="text-sm text-gray-500 hidden md:block">
                | {currentChannel.description}
              </span>
            )}
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={() => handleStartCall('video')}
              className="p-2 hover:bg-slate-100 rounded-lg transition-colors"
              title="Start video call"
            >
              <Video className="w-5 h-5" />
            </button>
            <button
              onClick={() => handleStartCall('audio')}
              className="p-2 hover:bg-slate-100 rounded-lg transition-colors"
              title="Start audio call"
            >
              <Phone className="w-5 h-5" />
            </button>
            <button className="p-2 hover:bg-slate-100 rounded-lg transition-colors" title="More options">
              <MoreHorizontal className="w-5 h-5" />
            </button>
          </div>
        </header>

        {/* Messages Area */}
        <div className="flex-1 overflow-y-auto bg-white">
          {error ? (
            <div className="flex items-center justify-center h-full">
              <div className="flex flex-col items-center gap-4 text-center">
                <div className="w-16 h-16 rounded-full bg-red-100 flex items-center justify-center">
                  <svg className="w-8 h-8 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                </div>
                <p className="text-red-600">{error}</p>
              </div>
            </div>
          ) : loading ? (
            <div className="flex items-center justify-center h-full">
              <div className="flex flex-col items-center gap-4">
                <div className="w-10 h-10 border-4 border-indigo-200 border-t-indigo-600 rounded-full animate-spin"></div>
                <p className="text-gray-500">Loading messages...</p>
              </div>
            </div>
          ) : currentMessages.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-center p-8">
              <div className="w-20 h-20 rounded-full bg-indigo-50 flex items-center justify-center mb-6">
                <MessageSquare size={36} className="text-indigo-600" />
              </div>
              <h3 className="text-xl font-semibold text-gray-900 mb-2">No messages yet</h3>
              <p className="text-gray-500 max-w-md">
                Be the first to start the conversation in this channel!
              </p>
            </div>
          ) : (
            <div className="p-4">
              {Object.entries(messageGroups).map(([dateKey, msgs]) => (
                <div key={dateKey}>
                  {/* Date Separator */}
                  <div className="flex items-center gap-4 my-6">
                    <div className="flex-1 h-px bg-slate-200"></div>
                    <span className="text-xs font-medium text-gray-500 px-2">
                      {formatDate(msgs[0].created_at)}
                    </span>
                    <div className="flex-1 h-px bg-slate-200"></div>
                  </div>

                  {/* Messages */}
                  {msgs.map((message) => {
                    const isOwn = message.sender?.id === currentUser?.id;
                    const senderName = message.sender?.display_name || 'Unknown User';

                    return (
                      <div
                        key={message.id}
                        className={`group flex gap-3 px-2 py-2 mt-3 ${
                          isOwn ? 'flex-row-reverse' : 'flex-row'
                        }`}
                      >
                        {/* Avatar */}
                        <div className="w-10 flex-shrink-0">
                          <div className={`w-10 h-10 rounded-full flex items-center justify-center text-white font-medium ${
                            isOwn ? 'bg-gradient-to-br from-indigo-600 to-cyan-500' : 'bg-slate-600'
                          }`}>
                            {isOwn ? 'Yo' : getInitials(senderName)}
                          </div>
                        </div>

                        {/* Message Content */}
                        <div className={`max-w-[70%] ${isOwn ? 'items-end' : 'items-start'}`}>
                          <div className={`flex items-baseline gap-2 mb-1 ${
                            isOwn ? 'flex-row-reverse' : 'flex-row'
                          }`}>
                            <span className="font-semibold text-gray-900 text-sm">
                              {isOwn ? 'Yo' : senderName}
                            </span>
                            <span className="text-xs text-gray-500">
                              {formatTime(message.created_at)}
                            </span>
                          </div>
                          <div className={`px-4 py-2 rounded-2xl break-words ${
                            isOwn
                              ? 'message-bubble-own'
                              : 'bg-slate-100 text-gray-800 rounded-tl-sm'
                          }`}>
                            {message.content}
                          </div>
                        </div>

                        {/* Message Actions (on hover) */}
                        <div className={`opacity-0 group-hover:opacity-100 flex items-start gap-1 transition-opacity ${
                          isOwn ? 'flex-row-reverse' : 'flex-row'
                        }`}>
                          <button className="p-1 hover:bg-slate-200 rounded" title="React">
                            <Smile className="w-5 h-5" />
                          </button>
                          <button className="p-1 hover:bg-slate-200 rounded" title="More">
                            <MoreHorizontal className="w-5 h-5" />
                          </button>
                        </div>
                      </div>
                    );
                  })}
                </div>
              ))}
              <div ref={messagesEndRef} />
            </div>
          )}
        </div>

        {/* Message Input */}
        <div className="bg-white border-t border-slate-200 p-4">
          <form onSubmit={handleSendMessage} className="relative">
            <div className="flex items-end gap-2 bg-slate-100 rounded-xl p-2">
              <button
                type="button"
                className="p-2 hover:bg-slate-200 rounded-lg transition-colors text-gray-500"
                title="Attach file"
              >
                <Paperclip className="w-5 h-5" />
              </button>
              <input
                ref={inputRef}
                type="text"
                value={newMessage}
                onChange={handleInputChange}
                placeholder={currentChannel?.channel_type === 'direct_message'
                  ? `Mensaje a ${currentChannel.name.replace(/^(dm-|DM:\s*)/i, '')}...`
                  : `Mensaje en ${currentChannel?.name ? '#' + currentChannel.name : 'canal'}...`
                }
                className="flex-1 bg-transparent border-0 focus:ring-0 text-gray-900 placeholder-gray-500 py-2"
              />
              <button
                type="button"
                className="p-2 hover:bg-slate-200 rounded-lg transition-colors text-gray-500"
                title="Add emoji"
              >
                <Smile className="w-5 h-5" />
              </button>
              <button
                type="submit"
                disabled={!newMessage.trim()}
                className={`p-2 rounded-lg transition-colors ${
                  newMessage.trim()
                    ? 'bg-indigo-600 text-white hover:bg-indigo-700'
                    : 'bg-slate-200 text-gray-400 cursor-not-allowed'
                }`}
                title="Send message"
              >
                <Send className="w-5 h-5" />
              </button>
            </div>
            <div className="flex items-center gap-4 mt-2 text-xs text-gray-500">
              {typingUsers.length > 0 ? (
                <span className="text-indigo-600 animate-pulse">
                  {typingUsers.length === 1
                    ? `${typingUsers[0]} is typing...`
                    : `${typingUsers.slice(0, 2).join(', ')}${typingUsers.length > 2 ? ` and ${typingUsers.length - 2} more` : ''} are typing...`}
                </span>
              ) : (
                <span>Press <kbd className="px-1 py-0.5 bg-slate-200 rounded text-gray-600">Enter</kbd> to send</span>
              )}
            </div>
          </form>
        </div>
      </main>
    </div>
  );
}

export default ChatPage;
