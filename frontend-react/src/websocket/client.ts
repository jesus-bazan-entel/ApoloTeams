import type { WebSocketMessage } from '../types';

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 3000;
  private messageHandlers: Map<string, (data: any) => void> = new Map();
  private currentToken: string | null = null;
  private authenticated = false;
  private pendingMessages: WebSocketMessage[] = [];
  private joinedChannels: Set<string> = new Set();

  constructor(private url: string) {}

  connect(token: string): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      return;
    }

    this.currentToken = token;
    this.authenticated = false;
    this.ws = new WebSocket(this.url);

    this.ws.onopen = () => {
      console.log('WebSocket connected, authenticating...');
      this.reconnectAttempts = 0;

      // Send authentication message directly (bypass queue)
      this.sendDirect({
        type: 'Authenticate',
        payload: { token }
      });
    };

    this.ws.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);

        // Handle authentication completion internally
        if (message.type === 'Authenticated') {
          this.authenticated = true;
          console.log('[WS] Authenticated. Tracked channels:', [...this.joinedChannels]);
          console.log('[WS] Pending messages:', this.pendingMessages.length);

          // Re-join all previously tracked channels
          for (const channelId of this.joinedChannels) {
            console.log('[WS] Re-joining channel:', channelId);
            this.sendDirect({
              type: 'JoinChannel',
              payload: { channel_id: channelId }
            });
          }

          // Flush pending messages, skipping all JoinChannel/LeaveChannel since
          // joinedChannels already represents the desired state and re-join handled it.
          // This prevents React.StrictMode's double-fire cleanup LeaveChannel from
          // unsubscribing us right after re-join.
          const pending = this.pendingMessages;
          this.pendingMessages = [];
          for (const msg of pending) {
            if (msg.type === 'JoinChannel' || msg.type === 'LeaveChannel') {
              console.log('[WS] Skipping queued channel msg during flush:', msg.type);
              continue;
            }
            console.log('[WS] Flushing pending:', msg.type);
            this.sendDirect(msg);
          }
        }

        this.handleMessage(message);
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    this.ws.onclose = () => {
      console.log('WebSocket disconnected');
      this.authenticated = false;
      this.attemptReconnect();
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
  }

  private attemptReconnect(): void {
    if (this.reconnectAttempts < this.maxReconnectAttempts && this.currentToken) {
      this.reconnectAttempts++;
      setTimeout(() => {
        console.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})`);
        this.connect(this.currentToken!);
      }, this.reconnectDelay);
    }
  }

  private handleMessage(message: WebSocketMessage): void {
    const handler = this.messageHandlers.get(message.type);
    if (handler) {
      console.log('[WS] Handling message:', message.type);
      handler(message.payload);
    } else {
      console.warn('[WS] No handler for message type:', message.type, message);
    }
  }

  on<T extends WebSocketMessage['type']>(
    type: T,
    handler: (data: Extract<WebSocketMessage, { type: T }>['payload']) => void
  ): void {
    this.messageHandlers.set(type, handler);
  }

  off(type: WebSocketMessage['type']): void {
    this.messageHandlers.delete(type);
  }

  /** Send directly to the WebSocket without queueing */
  private sendDirect(message: WebSocketMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  send(message: WebSocketMessage): void {
    // Track channel subscriptions so we can re-join after reconnect
    if (message.type === 'JoinChannel') {
      this.joinedChannels.add(message.payload.channel_id);
    } else if (message.type === 'LeaveChannel') {
      this.joinedChannels.delete(message.payload.channel_id);
    }

    if (this.ws?.readyState === WebSocket.OPEN && this.authenticated) {
      console.log('[WS] Sending:', message.type, 'authenticated:', this.authenticated);
      this.ws.send(JSON.stringify(message));
    } else {
      console.log('[WS] Queuing (not ready):', message.type, 'wsState:', this.ws?.readyState, 'auth:', this.authenticated);
      this.pendingMessages.push(message);
    }
  }

  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.authenticated = false;
    this.pendingMessages = [];
  }

  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN && this.authenticated;
  }
}

// Use relative URL to leverage Vite's proxy in development
const wsUrl = `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/ws`;
export const wsClient = new WebSocketClient(wsUrl);
