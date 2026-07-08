import * as cp from 'child_process';
import * as rl from 'readline';

interface LSPResponse {
  jsonrpc: string;
  id?: number;
  result?: any;
  error?: { code: number; message: string; data?: any };
  method?: string;
  params?: any;
}

interface LSPClientOptions {
  binaryPath: string;
  args?: string[];
}

export class LodeLSPClient {
  private process: cp.ChildProcess | null = null;
  private lineReader: rl.Interface | null = null;
  private pending = new Map<number, { resolve: (v: any) => void; reject: (e: Error) => void }>();
  private nextId = 1;
  private buffer = '';
  private contentLength = 0;

  constructor(private options: LSPClientOptions) {}

  start(): void {
    if (this.process) return;
    const args = this.options.args ?? ['lsp'];
    this.process = cp.spawn(this.options.binaryPath, args, {
      stdio: ['pipe', 'pipe', 'pipe'],
    });
    this.process.on('exit', (code) => {
      this.process = null;
    });
    this.process.stdout?.on('data', (data: Buffer) => {
      this.handleData(data.toString());
    });
    this.process.stderr?.on('data', (data: Buffer) => {
      console.error(`[lode-lsp stderr] ${data.toString().trim()}`);
    });
  }

  private handleData(chunk: string): void {
    this.buffer += chunk;
    while (this.buffer.length > 0) {
      const headerMatch = this.buffer.match(/^Content-Length: (\d+)\r\n\r\n/);
      if (!headerMatch) break;
      const msgLen = parseInt(headerMatch[1], 10);
      const headerEnd = headerMatch[0].length;
      if (this.buffer.length < headerEnd + msgLen) break;
      const rawMsg = this.buffer.slice(headerEnd, headerEnd + msgLen);
      this.buffer = this.buffer.slice(headerEnd + msgLen);
      try {
        const msg: LSPResponse = JSON.parse(rawMsg);
        this.handleMessage(msg);
      } catch {
        // skip malformed messages
      }
    }
  }

  private handleMessage(msg: LSPResponse): void {
    if (msg.id != null && this.pending.has(msg.id)) {
      const pending = this.pending.get(msg.id)!;
      this.pending.delete(msg.id);
      if (msg.error) {
        pending.reject(new Error(msg.error.message));
      } else {
        pending.resolve(msg.result);
      }
    }
  }

  private send(method: string, params?: any): Promise<any> {
    return new Promise((resolve, reject) => {
      const id = this.nextId++;
      const msg = JSON.stringify({ jsonrpc: '2.0', id, method, params });
      const header = `Content-Length: ${Buffer.byteLength(msg, 'utf-8')}\r\n\r\n`;
      this.process?.stdin?.write(header + msg);
      this.pending.set(id, { resolve, reject });
    });
  }

  async initialize(rootUri?: string): Promise<any> {
    this.start();
    const result = await this.send('initialize', {
      processId: process.pid,
      rootUri: rootUri ?? null,
      capabilities: {
        textDocument: {
          synchronization: { didSave: true },
          diagnostic: { dynamicRegistration: false },
        },
        workspace: { configuration: true },
      },
    });
    await this.send('initialized');
    return result;
  }

  async didOpenTextDocument(uri: string, languageId: string, text: string): Promise<void> {
    await this.send('textDocument/didOpen', {
      textDocument: { uri, languageId, version: 1, text },
    });
  }

  async didSaveTextDocument(uri: string): Promise<void> {
    await this.send('textDocument/didSave', { textDocument: { uri } });
  }

  async shutdown(): Promise<void> {
    try {
      await this.send('shutdown');
      this.send('exit');
    } catch {
      // ignore
    }
    this.process?.kill();
    this.process = null;
  }

  dispose(): void {
    this.shutdown();
  }
}
