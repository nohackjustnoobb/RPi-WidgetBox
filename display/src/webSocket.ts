interface Message {
  type: string;
  data?: any;
}

class WebSocketClient {
  private socket: WebSocket | null = null;

  constructor(
    private url: string,
    private onmessage: (mesg: Message) => void,
    private onopen: () => void,
    private reconnectInterval = 5000
  ) {
    this.connect();
  }

  private connect() {
    this.socket = new WebSocket(this.url);

    this.socket.onopen = this.onopen.bind(this);

    this.socket.onmessage = (event) => this.onmessage(JSON.parse(event.data));
    this.socket.onerror = (error) => {
      console.error("WebSocket error:", error);
    };

    this.socket.onclose = () => {
      console.log("WebSocket closed, attempting to reconnect...");
      setTimeout(() => this.connect(), this.reconnectInterval);
    };
  }

  send(mesg: Message) {
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify(mesg));
    } else {
      console.warn("WebSocket not open. Message not sent.");
    }
  }
}

export { type Message, WebSocketClient };
