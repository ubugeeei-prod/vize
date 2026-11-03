/**
 * Stdio peers for `lsp-vs-volar.ts`: an LSP JSON-RPC client and a tsserver
 * protocol client. Both only frame and route messages; timing and semantics
 * live in the editors that use them.
 */

import type { ChildProcessWithoutNullStreams } from "node:child_process";

type Waiter = { resolve: (value: unknown) => void; reject: (error: Error) => void };

/** `Content-Length` framed messages; tsserver counts its trailing newline. */
export class FrameReader {
  private buffer: Buffer = Buffer.alloc(0);
  private readonly onMessage: (message: unknown) => void;

  constructor(onMessage: (message: unknown) => void) {
    this.onMessage = onMessage;
  }

  push(chunk: Buffer): void {
    this.buffer = this.buffer.length ? Buffer.concat([this.buffer, chunk]) : chunk;
    for (;;) {
      const header = this.buffer.indexOf("\r\n\r\n");
      if (header < 0) return;
      const length = /content-length:\s*(\d+)/i.exec(this.buffer.subarray(0, header).toString());
      if (!length)
        throw new Error(
          `frame without Content-Length: ${this.buffer.subarray(0, header).toString()}`,
        );
      const end = header + 4 + Number(length[1]);
      if (this.buffer.length < end) return;
      const body = this.buffer.subarray(header + 4, end).toString("utf8");
      this.buffer = this.buffer.subarray(end);
      this.onMessage(JSON.parse(body));
    }
  }
}

export class RpcError extends Error {
  readonly method: string;
  readonly payload: unknown;

  constructor(method: string, payload: unknown) {
    super(`${method} failed: ${JSON.stringify(payload).slice(0, 2000)}`);
    this.method = method;
    this.payload = payload;
  }
}

type LspMessage = {
  id?: number | string;
  method?: string;
  params?: unknown;
  result?: unknown;
  error?: unknown;
};

/** Client side of an LSP server over stdio. */
export class LspPeer {
  private nextId = 0;
  private readonly pending = new Map<number, Waiter & { method: string }>();
  private readonly notificationHandlers = new Map<string, Array<(params: any) => void>>();
  private readonly requestHandlers = new Map<string, (params: any) => unknown>();
  stderr = "";
  /** Set once the process exits; a session stops at the next step instead of timing out. */
  exit: string | null = null;
  readonly child: ChildProcessWithoutNullStreams;
  readonly name: string;

  constructor(child: ChildProcessWithoutNullStreams, name: string) {
    this.child = child;
    this.name = name;
    const reader = new FrameReader((message) => this.dispatch(message as LspMessage));
    child.stdout.on("data", (chunk: Buffer) => reader.push(chunk));
    child.stderr.on("data", (chunk: Buffer) => {
      this.stderr = (this.stderr + chunk.toString("utf8")).slice(-64 * 1024);
    });
    child.once("exit", (code, signal) => {
      this.exit = `${name} exited (code=${code}, signal=${signal})`;
      const error = new Error(`${this.exit}\n${this.stderr}`);
      for (const waiter of this.pending.values()) waiter.reject(error);
      this.pending.clear();
    });
  }

  request<T = unknown>(method: string, params: unknown, timeoutMs = 120_000): Promise<T> {
    const id = ++this.nextId;
    return new Promise<T>((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`${this.name}: ${method} timed out after ${timeoutMs}ms`));
      }, timeoutMs);
      this.pending.set(id, {
        method,
        resolve: (value) => {
          clearTimeout(timer);
          resolve(value as T);
        },
        reject: (error) => {
          clearTimeout(timer);
          reject(error);
        },
      });
      this.write({ jsonrpc: "2.0", id, method, params });
    });
  }

  notify(method: string, params: unknown): void {
    this.write({ jsonrpc: "2.0", method, params });
  }

  onNotification(method: string, handler: (params: any) => void): void {
    const handlers = this.notificationHandlers.get(method) ?? [];
    handlers.push(handler);
    this.notificationHandlers.set(method, handlers);
  }

  /** Server-to-client requests; unhandled ones answer `null` like a minimal editor. */
  onRequest(method: string, handler: (params: any) => unknown): void {
    this.requestHandlers.set(method, handler);
  }

  private write(message: unknown): void {
    const body = JSON.stringify(message);
    this.child.stdin.write(`Content-Length: ${Buffer.byteLength(body, "utf8")}\r\n\r\n${body}`);
  }

  private dispatch(message: LspMessage): void {
    if (message.method != null && message.id != null) {
      const handler = this.requestHandlers.get(message.method);
      let result: unknown = null;
      if (handler) result = handler(message.params);
      else if (message.method === "workspace/configuration") {
        result = ((message.params as { items?: unknown[] })?.items ?? []).map(() => null);
      }
      this.write({ jsonrpc: "2.0", id: message.id, result });
      return;
    }
    if (message.method != null) {
      for (const handler of this.notificationHandlers.get(message.method) ?? []) {
        handler(message.params);
      }
      return;
    }
    const waiter = this.pending.get(Number(message.id));
    if (!waiter) return;
    this.pending.delete(Number(message.id));
    if (message.error) waiter.reject(new RpcError(waiter.method, message.error));
    else waiter.resolve(message.result ?? null);
  }
}

type TsMessage = {
  seq: number;
  type: "response" | "event";
  command?: string;
  request_seq?: number;
  success?: boolean;
  message?: string;
  body?: any;
  event?: string;
};

/** tsserver's own protocol: newline-delimited requests, framed responses and events. */
export class TsServerPeer {
  private seq = 0;
  private readonly pending = new Map<number, Waiter & { command: string }>();
  private readonly eventHandlers = new Set<(event: string, body: any) => void>();
  stderr = "";
  exit: string | null = null;
  readonly child: ChildProcessWithoutNullStreams;

  constructor(child: ChildProcessWithoutNullStreams) {
    this.child = child;
    const reader = new FrameReader((message) => this.dispatch(message as TsMessage));
    child.stdout.on("data", (chunk: Buffer) => reader.push(chunk));
    child.stderr.on("data", (chunk: Buffer) => {
      this.stderr = (this.stderr + chunk.toString("utf8")).slice(-64 * 1024);
    });
    child.once("exit", (code, signal) => {
      this.exit = `tsserver exited (code=${code}, signal=${signal})`;
      const error = new Error(`${this.exit}\n${this.stderr}`);
      for (const waiter of this.pending.values()) waiter.reject(error);
      this.pending.clear();
    });
  }

  /** Allocates the sequence number first so callers can match `requestCompleted`. */
  send(command: string, args: unknown): { seq: number; response: Promise<any> } {
    const seq = ++this.seq;
    const response = new Promise<any>((resolve, reject) => {
      this.pending.set(seq, { command, resolve, reject });
    });
    this.child.stdin.write(
      `${JSON.stringify({ seq, type: "request", command, arguments: args })}\n`,
    );
    return { seq, response };
  }

  /** The response body, or `null` when tsserver answers "no info" (`success: false`). */
  request(command: string, args: unknown): Promise<any> {
    return this.send(command, args).response;
  }

  /** Returns the unsubscribe function. */
  onEvent(handler: (event: string, body: any) => void): () => void {
    this.eventHandlers.add(handler);
    return () => this.eventHandlers.delete(handler);
  }

  private dispatch(message: TsMessage): void {
    if (message.type === "event") {
      for (const handler of this.eventHandlers) handler(message.event ?? "", message.body);
      return;
    }
    const waiter = this.pending.get(Number(message.request_seq));
    if (!waiter) return;
    this.pending.delete(Number(message.request_seq));
    // A failed tsserver request is an answer ("no info here"), not a transport fault.
    waiter.resolve(message.success ? (message.body ?? null) : null);
  }
}
