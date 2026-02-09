import axios, { AxiosError } from 'axios';
import type {
  AuthResponse,
  RegisterRequest,
  LoginRequest,
  User,
  Team,
  Channel,
  Message,
  CreateTeamRequest,
  CreateChannelRequest,
  SendMessageRequest,
  UpdateUserRequest,
  ChangePasswordRequest,
  Call,
  Notification,
  Meeting,
  CreateMeetingRequest,
  UpdateMeetingRequest,
  MeetingResponseStatus,
} from '../types';

const API_BASE_URL = '/api/v1';
const TOKEN_KEY = 'rust_teams_token';
const REFRESH_TOKEN_KEY = 'rust_teams_refresh_token';

class ApiClient {
  private token: string | null = null;

  constructor() {
    this.loadTokens();
    this.setupInterceptors();
  }

  private loadTokens() {
    this.token = localStorage.getItem(TOKEN_KEY);
  }

  private saveTokens(accessToken: string, refreshToken: string) {
    localStorage.setItem(TOKEN_KEY, accessToken);
    localStorage.setItem(REFRESH_TOKEN_KEY, refreshToken);
    this.token = accessToken;
  }

  private clearTokens() {
    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
    this.token = null;
  }

  private setupInterceptors() {
    axios.interceptors.request.use((config) => {
      if (this.token) {
        config.headers.Authorization = `Bearer ${this.token}`;
      }
      return config;
    });

    axios.interceptors.response.use(
      (response) => response,
      async (error: AxiosError) => {
        if (error.response?.status === 401) {
          try {
            await this.refreshToken();
            // Retry the original request
            if (error.config) {
              return axios.request(error.config);
            }
          } catch (refreshError) {
            this.clearTokens();
            window.location.href = '/login';
          }
        }
        return Promise.reject(error);
      }
    );
  }

  // Authentication
  async register(data: RegisterRequest): Promise<AuthResponse> {
    const response = await axios.post<AuthResponse>(`${API_BASE_URL}/auth/register`, data);
    this.saveTokens(response.data.access_token, response.data.refresh_token);
    return response.data;
  }

  async login(data: LoginRequest): Promise<AuthResponse> {
    const response = await axios.post<AuthResponse>(`${API_BASE_URL}/auth/login`, data);
    this.saveTokens(response.data.access_token, response.data.refresh_token);
    return response.data;
  }

  async logout(): Promise<void> {
    try {
      await axios.post(`${API_BASE_URL}/auth/logout`);
    } finally {
      this.clearTokens();
    }
  }

  async refreshToken(): Promise<AuthResponse> {
    const refreshToken = localStorage.getItem(REFRESH_TOKEN_KEY);
    if (!refreshToken) {
      throw new Error('No refresh token available');
    }
    const response = await axios.post<AuthResponse>(`${API_BASE_URL}/auth/refresh`, {
      refresh_token: refreshToken,
    });
    this.saveTokens(response.data.access_token, response.data.refresh_token);
    return response.data;
  }

  // Users
  async getCurrentUser(): Promise<User> {
    const response = await axios.get<User>(`${API_BASE_URL}/users/me`);
    return response.data;
  }

  async updateCurrentUser(data: UpdateUserRequest): Promise<User> {
    const response = await axios.patch<User>(`${API_BASE_URL}/users/me`, data);
    return response.data;
  }

  async changePassword(data: ChangePasswordRequest): Promise<void> {
    await axios.put(`${API_BASE_URL}/users/me/password`, data);
  }

  async searchUsers(query: string): Promise<User[]> {
    const response = await axios.get<User[]>(`${API_BASE_URL}/users/search?q=${query}`);
    return response.data;
  }

  // Teams
  async listTeams(): Promise<Team[]> {
    const response = await axios.get<Team[]>(`${API_BASE_URL}/teams`);
    return response.data;
  }

  async createTeam(data: CreateTeamRequest): Promise<Team> {
    const response = await axios.post<Team>(`${API_BASE_URL}/teams`, data);
    return response.data;
  }

  async getTeam(teamId: string): Promise<Team> {
    const response = await axios.get<Team>(`${API_BASE_URL}/teams/${teamId}`);
    return response.data;
  }

  async updateTeam(teamId: string, data: Partial<CreateTeamRequest>): Promise<Team> {
    const response = await axios.patch<Team>(`${API_BASE_URL}/teams/${teamId}`, data);
    return response.data;
  }

  async deleteTeam(teamId: string): Promise<void> {
    await axios.delete(`${API_BASE_URL}/teams/${teamId}`);
  }

  async listTeamMembers(teamId: string): Promise<any[]> {
    const response = await axios.get(`${API_BASE_URL}/teams/${teamId}/members`);
    return response.data;
  }

  async addTeamMember(teamId: string, userId: string, role?: string): Promise<any> {
    const response = await axios.post(`${API_BASE_URL}/teams/${teamId}/members`, {
      user_id: userId,
      role: role || 'member',
    });
    return response.data;
  }

  async removeTeamMember(teamId: string, userId: string): Promise<void> {
    await axios.delete(`${API_BASE_URL}/teams/${teamId}/members/${userId}`);
  }

  // Channels
  async listChannels(): Promise<Channel[]> {
    const response = await axios.get<Channel[]>(`${API_BASE_URL}/channels`);
    return response.data;
  }

  async listTeamChannels(teamId: string): Promise<Channel[]> {
    const response = await axios.get<Channel[]>(`${API_BASE_URL}/teams/${teamId}/channels`);
    return response.data;
  }

  async createChannel(data: CreateChannelRequest): Promise<Channel> {
    const response = await axios.post<Channel>(`${API_BASE_URL}/channels`, data);
    return response.data;
  }

  async createDmChannel(userId: string): Promise<Channel> {
    const response = await axios.post<Channel>(`${API_BASE_URL}/channels/dm`, { user_id: userId });
    return response.data;
  }

  async getChannel(channelId: string): Promise<Channel> {
    const response = await axios.get<Channel>(`${API_BASE_URL}/channels/${channelId}`);
    return response.data;
  }

  async updateChannel(channelId: string, data: Partial<CreateChannelRequest>): Promise<Channel> {
    const response = await axios.patch<Channel>(`${API_BASE_URL}/channels/${channelId}`, data);
    return response.data;
  }

  async deleteChannel(channelId: string): Promise<void> {
    await axios.delete(`${API_BASE_URL}/channels/${channelId}`);
  }

  async listChannelMembers(channelId: string): Promise<any[]> {
    const response = await axios.get(`${API_BASE_URL}/channels/${channelId}/members`);
    return response.data;
  }

  async markChannelAsRead(channelId: string): Promise<void> {
    await axios.post(`${API_BASE_URL}/channels/${channelId}/read`);
  }

  // Messages
  async listMessages(channelId: string, limit: number = 50): Promise<Message[]> {
    const response = await axios.get<Message[]>(
      `${API_BASE_URL}/channels/${channelId}/messages?limit=${limit}`
    );
    return response.data;
  }

  async sendMessage(channelId: string, data: SendMessageRequest): Promise<Message> {
    const response = await axios.post<Message>(
      `${API_BASE_URL}/channels/${channelId}/messages`,
      data
    );
    return response.data;
  }

  async updateMessage(channelId: string, messageId: string, data: { content: string }): Promise<Message> {
    const response = await axios.patch<Message>(
      `${API_BASE_URL}/channels/${channelId}/messages/${messageId}`,
      data
    );
    return response.data;
  }

  async deleteMessage(channelId: string, messageId: string): Promise<void> {
    await axios.delete(`${API_BASE_URL}/channels/${channelId}/messages/${messageId}`);
  }

  async addReaction(channelId: string, messageId: string, emoji: string): Promise<void> {
    await axios.post(`${API_BASE_URL}/channels/${channelId}/messages/${messageId}/reactions`, {
      emoji,
    });
  }

  async removeReaction(channelId: string, messageId: string, emoji: string): Promise<void> {
    await axios.delete(
      `${API_BASE_URL}/channels/${channelId}/messages/${messageId}/reactions/${emoji}`
    );
  }

  // Calls
  async startCall(data: { channel_id?: string; target_user_id?: string; call_type: 'audio' | 'video' }): Promise<Call> {
    const response = await axios.post<Call>(`${API_BASE_URL}/calls`, data);
    return response.data;
  }

  async startDirectCall(targetUserId: string, callType: 'audio' | 'video'): Promise<Call> {
    const response = await axios.post<Call>(`${API_BASE_URL}/calls`, {
      target_user_id: targetUserId,
      call_type: callType,
    });
    return response.data;
  }

  async getCall(callId: string): Promise<Call> {
    const response = await axios.get<Call>(`${API_BASE_URL}/calls/${callId}`);
    return response.data;
  }

  async joinCall(callId: string): Promise<Call> {
    const response = await axios.post<Call>(`${API_BASE_URL}/calls/${callId}/join`);
    return response.data;
  }

  async leaveCall(callId: string): Promise<void> {
    await axios.post(`${API_BASE_URL}/calls/${callId}/leave`);
  }

  async endCall(callId: string): Promise<void> {
    await axios.post(`${API_BASE_URL}/calls/${callId}/end`);
  }

  async updateCallParticipant(
    callId: string,
    data: { is_muted?: boolean; is_video_enabled?: boolean }
  ): Promise<void> {
    await axios.patch(`${API_BASE_URL}/calls/${callId}/participant`, data);
  }

  // Search
  async searchMessages(query: string): Promise<{ messages: Message[]; total_count: number }> {
    const response = await axios.get(`${API_BASE_URL}/search/messages?q=${query}`);
    return response.data;
  }

  // Notifications
  async listNotifications(): Promise<Notification[]> {
    const response = await axios.get<Notification[]>(`${API_BASE_URL}/notifications`);
    return response.data;
  }

  async markNotificationAsRead(notificationId: string): Promise<void> {
    await axios.post(`${API_BASE_URL}/notifications/${notificationId}/read`);
  }

  async markAllNotificationsAsRead(): Promise<void> {
    await axios.post(`${API_BASE_URL}/notifications/read-all`);
  }

  // Meetings
  async listMeetings(): Promise<Meeting[]> {
    const response = await axios.get<Meeting[]>(`${API_BASE_URL}/meetings`);
    return response.data;
  }

  async getMeeting(meetingId: string): Promise<Meeting> {
    const response = await axios.get<Meeting>(`${API_BASE_URL}/meetings/${meetingId}`);
    return response.data;
  }

  async getCalendarMeetings(startDate: string, endDate: string): Promise<Meeting[]> {
    const response = await axios.get<Meeting[]>(
      `${API_BASE_URL}/meetings/calendar?start_date=${startDate}&end_date=${endDate}`
    );
    return response.data;
  }

  async createMeeting(data: CreateMeetingRequest): Promise<Meeting> {
    const response = await axios.post<Meeting>(`${API_BASE_URL}/meetings`, data);
    return response.data;
  }

  async updateMeeting(meetingId: string, data: UpdateMeetingRequest): Promise<Meeting> {
    const response = await axios.patch<Meeting>(`${API_BASE_URL}/meetings/${meetingId}`, data);
    return response.data;
  }

  async cancelMeeting(meetingId: string): Promise<void> {
    await axios.post(`${API_BASE_URL}/meetings/${meetingId}/cancel`);
  }

  async deleteMeeting(meetingId: string): Promise<void> {
    await axios.delete(`${API_BASE_URL}/meetings/${meetingId}`);
  }

  async inviteToMeeting(meetingId: string, userIds: string[]): Promise<Meeting> {
    const response = await axios.post<Meeting>(`${API_BASE_URL}/meetings/${meetingId}/invite`, {
      user_ids: userIds,
    });
    return response.data;
  }

  async respondToMeeting(meetingId: string, response: MeetingResponseStatus): Promise<Meeting> {
    const resp = await axios.post<Meeting>(`${API_BASE_URL}/meetings/${meetingId}/respond`, {
      response,
    });
    return resp.data;
  }

  async removeFromMeeting(meetingId: string, userId: string): Promise<void> {
    await axios.delete(`${API_BASE_URL}/meetings/${meetingId}/participants/${userId}`);
  }
}

export const apiClient = new ApiClient();
