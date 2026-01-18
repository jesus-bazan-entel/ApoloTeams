import { useEffect, useState, useRef } from 'react';
import { useParams } from 'react-router-dom';
import { useStore } from '../store';
import { apiClient } from '../api/client';
import { wsClient } from '../websocket/client';
import type { Message } from '../types';

function ChatPage() {
  const { channelId } = useParams<{ channelId: string }>();
  const { currentUser, setSelectedChannel, messages, setMessages, addMessage } = useStore();
  const [newMessage, setNewMessage] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (channelId) {
      setSelectedChannel(channelId);
      loadMessages();
    }
  }, [channelId, setSelectedChannel]);

  useEffect(() => {
    // Scroll to bottom when new messages arrive
    if (messagesEndRef.current) {
      messagesEndRef.current.scrollTop = messagesEndRef.current.scrollHeight;
    }
  }, [messages]);

  const loadMessages = async () => {
    if (!channelId) return;
    try {
      const msgs = await apiClient.listMessages(channelId);
      setMessages(channelId, msgs);
    } catch (error) {
      console.error('Failed to load messages:', error);
    }
  };

  const handleSendMessage = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newMessage.trim() || !channelId) return;

    try {
      const message = await apiClient.sendMessage(channelId, { content: newMessage });
      addMessage(channelId, message);
      setNewMessage('');

      // Also send via WebSocket for real-time updates
      wsClient.send({
        type: 'SendMessage',
        payload: { channel_id: channelId, content: newMessage },
      });
    } catch (error) {
      console.error('Failed to send message:', error);
    }
  };

  const currentMessages: Message[] = channelId ? (messages[channelId] || []) : [];

  return (
    <div className="flex h-screen bg-gray-50">
      {/* Sidebar placeholder */}
      <div className="w-64 bg-white border-r border-gray-200 flex flex-col">
        <div className="p-4 border-b border-gray-200">
          <h2 className="text-lg font-semibold">Channels</h2>
        </div>
        <div className="flex-1 overflow-y-auto p-4">
          <div className="text-sm text-gray-500">Channel list will be here</div>
        </div>
      </div>

      {/* Main chat area */}
      <div className="flex-1 flex flex-col">
        {/* Header */}
        <div className="bg-white border-b border-gray-200 px-6 py-4">
          <h1 className="text-xl font-semibold">
            {channelId ? `Channel: ${channelId}` : 'Select a channel'}
          </h1>
        </div>

        {/* Messages */}
        <div className="flex-1 overflow-y-auto p-4">
          {currentMessages.length === 0 ? (
            <div className="flex items-center justify-center h-full text-gray-500">
              No messages yet. Start the conversation!
            </div>
          ) : (
            <div className="space-y-4">
              {currentMessages.map((message) => (
                <div
                  key={message.id}
                  className={`flex ${
                    message.sender.id === currentUser?.id ? 'justify-end' : 'justify-start'
                  }`}
                >
                  <div
                    className={`max-w-xs lg:max-w-md px-4 py-2 rounded-lg ${
                      message.sender.id === currentUser?.id ? 'bg-blue-600 text-white' : 'bg-gray-200 text-gray-900'
                    }`}
                  >
                    {message.sender.id !== currentUser?.id && (
                      <div className="text-xs text-gray-600 mb-1">
                        {message.sender.display_name}
                      </div>
                    )}
                    <div className="text-sm">{message.content}</div>
                    <div className="text-xs text-gray-600 mt-1">
                      {new Date(message.created_at).toLocaleString()}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
          <div ref={messagesEndRef} />
        </div>

        {/* Message input */}
        <div className="bg-white border-t border-gray-200 p-4">
          <form onSubmit={handleSendMessage} className="flex gap-2">
            <input
              type="text"
              value={newMessage}
              onChange={(e) => setNewMessage(e.target.value)}
              placeholder="Type a message..."
              className="flex-1 px-4 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            />
            <button
              type="submit"
              className="px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            >
              Send
            </button>
          </form>
        </div>
      </div>
    </div>
  );
}

export default ChatPage;
