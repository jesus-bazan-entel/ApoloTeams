// Shared types from backend
export interface User {
  id: string;
  email: string;
  username: string;
  display_name: string;
  avatar_url?: string;
  status: UserStatus;
  status_message?: string;
  last_seen?: string;
  created_at: string;
}

export type UserStatus = 'online' | 'away' | 'busy' | 'offline';

export interface Team {
  id: string;
  name: string;
  description?: string;
  avatar_url?: string;
  owner_id: string;
  member_count: number;
  created_at: string;
}

export interface Channel {
  id: string;
  team_id?: string;
  name: string;
  description?: string;
  channel_type: ChannelType;
  member_count: number;
  unread_count: number;
  last_message?: Message;
  created_at: string;
}

export type ChannelType = 'public' | 'private' | 'direct_message';

export interface Message {
  id: string;
  channel_id: string;
  sender: User;
  content: string;
  message_type: MessageType;
  reply_to?: Message;
  reactions: Reaction[];
  attachments: FileAttachment[];
  edited: boolean;
  created_at: string;
  updated_at: string;
}

export type MessageType = 'text' | 'image' | 'file' | 'system';

export interface Reaction {
  emoji: string;
  count: number;
  users: string[];
  reacted_by_me: boolean;
}

export interface FileAttachment {
  id: string;
  filename: string;
  file_size: number;
  mime_type: string;
  download_url: string;
  created_at: string;
}

export interface Call {
  id: string;
  channel_id: string;
  initiator: User;
  call_type: CallType;
  status: CallStatus;
  participants: CallParticipant[];
  started_at: string;
  ended_at?: string;
}

export type CallType = 'audio' | 'video';
export type CallStatus = 'ringing' | 'ongoing' | 'ended';

export interface CallParticipant {
  user: User;
  joined_at: string;
  is_muted: boolean;
  is_video_enabled: boolean;
}

export interface Notification {
  id: string;
  title: string;
  body: string;
  notification_type: string;
  reference_id?: string;
  read: boolean;
  created_at: string;
}

// API Request/Response types
export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
  user: User;
}

export interface RegisterRequest {
  email: string;
  username: string;
  display_name: string;
  password: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface CreateTeamRequest {
  name: string;
  description?: string;
  avatar_url?: string;
}

export interface CreateChannelRequest {
  team_id?: string;
  name: string;
  description?: string;
  channel_type?: ChannelType;
  member_ids?: string[];
}

export interface SendMessageRequest {
  content: string;
  message_type?: MessageType;
  reply_to_id?: string;
  attachment_ids?: string[];
}

export interface UpdateUserRequest {
  display_name?: string;
  avatar_url?: string;
  status?: UserStatus;
  status_message?: string;
}

export interface ChangePasswordRequest {
  current_password: string;
  new_password: string;
}

// WebSocket message types
export type WebSocketMessage =
  | { type: 'Authenticate'; payload: { token: string } }
  | { type: 'JoinChannel'; payload: { channel_id: string } }
  | { type: 'LeaveChannel'; payload: { channel_id: string } }
  | { type: 'SendMessage'; payload: { channel_id: string; content: string; reply_to_id?: string } }
  | { type: 'StartTyping'; payload: { channel_id: string } }
  | { type: 'StopTyping'; payload: { channel_id: string } }
  | { type: 'UpdateStatus'; payload: { status: UserStatus; status_message?: string } }
  | { type: 'Authenticated'; payload: { user: User } }
  | { type: 'Error'; payload: { code: string; message: string } }
  | { type: 'NewMessage'; payload: { message: Message } }
  | { type: 'MessageUpdated'; payload: { message: Message } }
  | { type: 'MessageDeleted'; payload: { channel_id: string; message_id: string } }
  | { type: 'UserTyping'; payload: { channel_id: string; user: User } }
  | { type: 'UserStoppedTyping'; payload: { channel_id: string; user_id: string } }
  | { type: 'UserStatusChanged'; payload: { user_id: string; status: UserStatus; status_message?: string } }
  | { type: 'UserJoinedChannel'; payload: { channel_id: string; user: User } }
  | { type: 'UserLeftChannel'; payload: { channel_id: string; user_id: string } }
  | { type: 'CallStarted'; payload: { call: Call } }
  | { type: 'CallEnded'; payload: { call_id: string } }
  | { type: 'ParticipantJoined'; payload: { call_id: string; participant: CallParticipant } }
  | { type: 'ParticipantLeft'; payload: { call_id: string; user_id: string } }
  | { type: 'Notification'; payload: { notification: Notification } }
  | { type: 'WebRTCOffer'; payload: { call_id: string; from_user_id: string; sdp: string } }
  | { type: 'WebRTCAnswer'; payload: { call_id: string; from_user_id: string; sdp: string } }
  | { type: 'WebRTCIceCandidate'; payload: { call_id: string; from_user_id: string; candidate: string } };
