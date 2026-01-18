import { useEffect, useState, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useStore } from '../store';
import { apiClient } from '../api/client';
import { wsClient } from '../websocket/client';
import type { Message, Channel } from '../types';

// Icons
const HashIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path fillRule="evenodd" d="M9.243 3.03a1 1 0 01.727 1.213L9.53 6h2.94l.56-2.243a1 1 0 111.94.486L14.53 6H17a1 1 0 110 2h-2.97l-1 4H15a1 1 0 110 2h-2.47l-.56 2.242a1 1 0 11-1.94-.485L10.47 14H7.53l-.56 2.242a1 1 0 11-1.94-.485L5.47 14H3a1 1 0 110-2h2.97l1-4H5a1 1 0 110-2h2.47l.56-2.243a1 1 0 011.213-.727zM9.03 8l-1 4h2.94l1-4H9.03z" clipRule="evenodd" />
  </svg>
);

const SendIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
  </svg>
);

const EmojiIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
  </svg>
);

const AttachIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13" />
  </svg>
);

const MoreIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
    <path d="M6 10a2 2 0 11-4 0 2 2 0 014 0zM12 10a2 2 0 11-4 0 2 2 0 014 0zM16 12a2 2 0 100-4 2 2 0 000 4z" />
  </svg>
);

const SearchIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
  </svg>
);

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

function ChatPage() {
  const { channelId } = useParams<{ channelId: string }>();
  const navigate = useNavigate();
  const { currentUser, setSelectedChannel, messages, setMessages, addMessage, channels, setChannels, selectedTeamId } = useStore();
  const [newMessage, setNewMessage] = useState('');
  const [loading, setLoading] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    loadChannels();
  }, [selectedTeamId]);

  useEffect(() => {
    if (channelId) {
      setSelectedChannel(channelId);
      loadMessages();
    }
  }, [channelId, setSelectedChannel]);

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  const loadChannels = async () => {
    if (!selectedTeamId) return;
    try {
      const channelsData = await apiClient.listTeamChannels(selectedTeamId);
      setChannels(channelsData);
    } catch (error) {
      console.error('Failed to load channels:', error);
    }
  };

  const loadMessages = async () => {
    if (!channelId) return;
    setLoading(true);
    try {
      const msgs = await apiClient.listMessages(channelId);
      setMessages(channelId, msgs);
    } catch (error) {
      console.error('Failed to load messages:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleSendMessage = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newMessage.trim() || !channelId) return;

    try {
      const message = await apiClient.sendMessage(channelId, { content: newMessage });
      addMessage(channelId, message);
      setNewMessage('');
      inputRef.current?.focus();

      wsClient.send({
        type: 'SendMessage',
        payload: { channel_id: channelId, content: newMessage },
      });
    } catch (error) {
      console.error('Failed to send message:', error);
    }
  };

  const currentMessages: Message[] = channelId ? (messages[channelId] || []) : [];
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
    <div className="flex h-screen bg-surface">
      {/* Sidebar - Channels */}
      <aside className="w-64 bg-white border-r border-gray-200 flex flex-col">
        {/* Sidebar Header */}
        <div className="h-16 px-4 flex items-center justify-between border-b border-gray-200">
          <h2 className="font-semibold text-gray-900">Channels</h2>
          <button className="p-1 hover:bg-gray-100 rounded transition-colors">
            <svg className="w-5 h-5 text-gray-500" fill="currentColor" viewBox="0 0 20 20">
              <path fillRule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clipRule="evenodd" />
            </svg>
          </button>
        </div>

        {/* Search */}
        <div className="p-3">
          <div className="relative">
            <SearchIcon />
            <input
              type="text"
              placeholder="Search channels"
              className="w-full pl-10 pr-4 py-2 text-sm bg-gray-100 border-0 rounded-md focus:ring-2 focus:ring-teams-purple focus:bg-white transition-all"
            />
            <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <SearchIcon />
            </div>
          </div>
        </div>

        {/* Channel List */}
        <div className="flex-1 overflow-y-auto">
          {channels.length === 0 ? (
            <div className="p-4 text-center text-gray-500 text-sm">
              No channels yet
            </div>
          ) : (
            <div className="py-2">
              {channels.map((channel: Channel) => (
                <button
                  key={channel.id}
                  onClick={() => navigate(`/chat/${channel.id}`)}
                  className={`w-full flex items-center gap-3 px-4 py-2 text-left transition-colors ${
                    channel.id === channelId
                      ? 'bg-teams-purple-50 text-teams-purple border-l-4 border-teams-purple'
                      : 'hover:bg-gray-100 text-gray-700'
                  }`}
                >
                  <HashIcon />
                  <span className="truncate font-medium">{channel.name}</span>
                </button>
              ))}
            </div>
          )}
        </div>
      </aside>

      {/* Main Chat Area */}
      <main className="flex-1 flex flex-col">
        {/* Chat Header */}
        <header className="h-16 bg-white border-b border-gray-200 px-6 flex items-center justify-between shadow-sm">
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2 text-gray-900">
              <HashIcon />
              <h1 className="text-lg font-semibold">
                {currentChannel?.name || 'Select a channel'}
              </h1>
            </div>
            {currentChannel?.description && (
              <span className="text-sm text-gray-500 hidden md:block">
                | {currentChannel.description}
              </span>
            )}
          </div>
          <div className="flex items-center gap-2">
            <button className="p-2 hover:bg-gray-100 rounded-lg transition-colors" title="Start video call">
              <VideoIcon />
            </button>
            <button className="p-2 hover:bg-gray-100 rounded-lg transition-colors" title="Start audio call">
              <CallIcon />
            </button>
            <button className="p-2 hover:bg-gray-100 rounded-lg transition-colors" title="More options">
              <MoreIcon />
            </button>
          </div>
        </header>

        {/* Messages Area */}
        <div className="flex-1 overflow-y-auto bg-white">
          {loading ? (
            <div className="flex items-center justify-center h-full">
              <div className="flex flex-col items-center gap-4">
                <div className="w-10 h-10 border-4 border-teams-purple-200 border-t-teams-purple rounded-full animate-spin"></div>
                <p className="text-gray-500">Loading messages...</p>
              </div>
            </div>
          ) : currentMessages.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-center p-8">
              <div className="w-20 h-20 rounded-full bg-teams-purple-50 flex items-center justify-center mb-6">
                <svg className="w-10 h-10 text-teams-purple" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M18 10c0 3.866-3.582 7-8 7a8.841 8.841 0 01-4.083-.98L2 17l1.338-3.123C2.493 12.767 2 11.434 2 10c0-3.866 3.582-7 8-7s8 3.134 8 7zM7 9H5v2h2V9zm8 0h-2v2h2V9zM9 9h2v2H9V9z" clipRule="evenodd" />
                </svg>
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
                    <div className="flex-1 h-px bg-gray-200"></div>
                    <span className="text-xs font-medium text-gray-500 px-2">
                      {formatDate(msgs[0].created_at)}
                    </span>
                    <div className="flex-1 h-px bg-gray-200"></div>
                  </div>

                  {/* Messages */}
                  {msgs.map((message, index) => {
                    const isOwn = message.sender.id === currentUser?.id;
                    const showAvatar = index === 0 || msgs[index - 1]?.sender.id !== message.sender.id;

                    return (
                      <div
                        key={message.id}
                        className={`group flex gap-3 px-2 py-1 hover:bg-gray-50 rounded-lg transition-colors ${
                          showAvatar ? 'mt-4' : 'mt-1'
                        }`}
                      >
                        {/* Avatar */}
                        <div className="w-10 flex-shrink-0">
                          {showAvatar && (
                            <div className={`w-10 h-10 rounded-full flex items-center justify-center text-white font-medium ${
                              isOwn ? 'bg-teams-purple' : 'bg-teams-blue'
                            }`}>
                              {getInitials(message.sender.display_name)}
                            </div>
                          )}
                        </div>

                        {/* Message Content */}
                        <div className="flex-1 min-w-0">
                          {showAvatar && (
                            <div className="flex items-baseline gap-2 mb-1">
                              <span className="font-semibold text-gray-900">
                                {message.sender.display_name}
                              </span>
                              <span className="text-xs text-gray-500">
                                {formatTime(message.created_at)}
                              </span>
                            </div>
                          )}
                          <div className="text-gray-800 break-words">
                            {message.content}
                          </div>
                        </div>

                        {/* Message Actions (on hover) */}
                        <div className="opacity-0 group-hover:opacity-100 flex items-start gap-1 transition-opacity">
                          <button className="p-1 hover:bg-gray-200 rounded" title="React">
                            <EmojiIcon />
                          </button>
                          <button className="p-1 hover:bg-gray-200 rounded" title="More">
                            <MoreIcon />
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
        <div className="bg-white border-t border-gray-200 p-4">
          <form onSubmit={handleSendMessage} className="relative">
            <div className="flex items-end gap-2 bg-gray-100 rounded-lg p-2">
              <button
                type="button"
                className="p-2 hover:bg-gray-200 rounded-lg transition-colors text-gray-500"
                title="Attach file"
              >
                <AttachIcon />
              </button>
              <input
                ref={inputRef}
                type="text"
                value={newMessage}
                onChange={(e) => setNewMessage(e.target.value)}
                placeholder={`Message ${currentChannel?.name ? '#' + currentChannel.name : 'channel'}...`}
                className="flex-1 bg-transparent border-0 focus:ring-0 text-gray-900 placeholder-gray-500 py-2"
              />
              <button
                type="button"
                className="p-2 hover:bg-gray-200 rounded-lg transition-colors text-gray-500"
                title="Add emoji"
              >
                <EmojiIcon />
              </button>
              <button
                type="submit"
                disabled={!newMessage.trim()}
                className={`p-2 rounded-lg transition-colors ${
                  newMessage.trim()
                    ? 'bg-teams-purple text-white hover:bg-teams-purple-600'
                    : 'bg-gray-200 text-gray-400 cursor-not-allowed'
                }`}
                title="Send message"
              >
                <SendIcon />
              </button>
            </div>
            <div className="flex items-center gap-4 mt-2 text-xs text-gray-500">
              <span>Press <kbd className="px-1 py-0.5 bg-gray-200 rounded text-gray-600">Enter</kbd> to send</span>
            </div>
          </form>
        </div>
      </main>
    </div>
  );
}

export default ChatPage;
