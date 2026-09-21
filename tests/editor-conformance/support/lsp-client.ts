// A minimal stdio JSON-RPC client for the protocol-replay driver. It answers
// the server-to-client requests every editor answers (configuration,
// capability registration, progress creation) with what the replayed client
// would answer, and nothing else.
import { type ChildProcessWithoutNullStreams, spawn } from "node:child_process";

export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
type Message = {
  id?: number | string;
  method?: string;
  params?: Json;
  result?: Json;
  error?: Json;
};
type Waiter = { predicate: (message: Message) => boolean; resolve: (message: Message) => void };

export class LspClient {
  private readonly child: ChildProcessWithoutNullStreams;
  private buffer = Buffer.alloc(0);
  private nextId = 0;
  private readonly pending = new Map<number, (message: Message) => void>();
  private readonly notifications: Message[] = [];
  private readonly waiters: Waiter[] = [];
  private readonly exited: Promise<number | null>;
  private readonly answer: (method: string, params: Json) => Json;

  constructor(
    command: string,
    args: string[],
    options: { cwd: string; env: NodeJS.ProcessEnv },
    answer: (method: string, params: Json) => Json,
  ) {
    this.answer = answer;
    this.child = spawn(command, args, { cwd: options.cwd, env: options.env, stdio: "pipe" });
    this.child.stderr.resume();
    this.child.stdout.on("data", (chunk: Buffer) => this.receive(chunk));
    this.exited = new Promise((resolve) => this.child.on("close", (code) => resolve(code)));
  }

  /** `params: undefined` omits the member, as JSON-RPC requires for parameterless calls. */
  request(method: string, params: Json | undefined): Promise<Json> {
    const id = ++this.nextId;
    this.send(
      params === undefined
        ? { jsonrpc: "2.0", id, method }
        : { jsonrpc: "2.0", id, method, params },
    );
    return new Promise((resolve, reject) => {
      this.pending.set(id, (message) =>
        message.error == null
          ? resolve(message.result ?? null)
          : reject(new Error(`${method} failed: ${JSON.stringify(message.error)}`)),
      );
    });
  }

  notify(method: string, params: Json | undefined): void {
    this.send(
      params === undefined ? { jsonrpc: "2.0", method } : { jsonrpc: "2.0", method, params },
    );
  }

  /** Resolves with the first notification (already received or future) that matches. */
  waitForNotification(
    method: string,
    predicate: (params: Json) => boolean,
    timeoutMs: number,
  ): Promise<Json> {
    const matches = (message: Message) =>
      message.method === method && predicate(message.params ?? null);
    const index = this.notifications.findIndex(matches);
    if (index >= 0) return Promise.resolve(this.notifications.splice(index, 1)[0].params ?? null);
    return new Promise((resolve, reject) => {
      const timer = setTimeout(
        () => reject(new Error(`timed out after ${timeoutMs}ms waiting for ${method}`)),
        timeoutMs,
      );
      this.waiters.push({
        predicate: matches,
        resolve: (message) => {
          clearTimeout(timer);
          resolve(message.params ?? null);
        },
      });
    });
  }

  async exit(): Promise<number | null> {
    this.child.stdin.end();
    return this.exited;
  }

  private send(message: Record<string, Json>): void {
    const body = Buffer.from(JSON.stringify(message), "utf8");
    this.child.stdin.write(`Content-Length: ${body.length}\r\n\r\n`);
    this.child.stdin.write(body);
  }

  private receive(chunk: Buffer): void {
    this.buffer = Buffer.concat([this.buffer, chunk]);
    for (;;) {
      const headerEnd = this.buffer.indexOf("\r\n\r\n");
      if (headerEnd < 0) return;
      const length = /content-length:\s*(\d+)/iu.exec(
        this.buffer.subarray(0, headerEnd).toString("ascii"),
      );
      if (!length) throw new Error("server frame without Content-Length");
      const end = headerEnd + 4 + Number(length[1]);
      if (this.buffer.length < end) return;
      const message = JSON.parse(this.buffer.subarray(headerEnd + 4, end).toString("utf8"));
      this.buffer = this.buffer.subarray(end);
      this.dispatch(message as Message);
    }
  }

  private dispatch(message: Message): void {
    if (message.method != null && message.id != null) {
      this.send({
        jsonrpc: "2.0",
        id: message.id,
        result: this.answer(message.method, message.params ?? null),
      });
      return;
    }
    if (message.method != null) {
      const waiter = this.waiters.findIndex(({ predicate }) => predicate(message));
      if (waiter >= 0) this.waiters.splice(waiter, 1)[0].resolve(message);
      else this.notifications.push(message);
      return;
    }
    const settle = this.pending.get(Number(message.id));
    this.pending.delete(Number(message.id));
    settle?.(message);
  }
}
