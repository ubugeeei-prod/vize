import assert from "node:assert/strict";
import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { LspRequestError } from "./errors.ts";
import { resolveVizeLaunchCommand } from "./launch.ts";
import { root } from "./paths.ts";
import type { JsonRpcId, JsonRpcMessage, LspInitializationOptions } from "./protocol.ts";

export { LspRequestError };

/**
 * Minimal JSON-RPC client for production LSP smoke tests.
 *
 * The smoke suite intentionally talks to `vize lsp` through stdio instead of a
 * mocked transport. This session class owns framing, request bookkeeping,
 * notification backlogs, and shutdown so individual tests can focus on editor
 * behavior rather than protocol mechanics.
 */
export class LspSession {
  private readonly process: ChildProcessWithoutNullStreams;
  private readonly pending = new Map<
    JsonRpcId,
    {
      resolve: (value: unknown) => void;
      reject: (error: Error) => void;
      method: string;
      timeout: NodeJS.Timeout;
    }
  >();
  private readonly notificationBacklog: Array<{ method: string; params: unknown }> = [];
  private readonly notifications: Array<{
    method: string;
    predicate?: (params: unknown) => boolean;
    resolve: (params: unknown) => void;
    reject: (error: Error) => void;
    timeout: NodeJS.Timeout;
  }> = [];
  /** Passive notification observers that do not consume waiter/backlog entries. */
  readonly notificationObservers: Array<(method: string, params: unknown) => void> = [];
  private buffer = Buffer.alloc(0);
  private nextId = 0;
  private stderr = "";

  constructor() {
    const [command, ...args] = resolveVizeLaunchCommand();
    this.process = spawn(command, args, {
      cwd: root,
      stdio: ["pipe", "pipe", "pipe"],
    });

    this.process.stdout.on("data", (chunk: Buffer) => {
      this.buffer = Buffer.concat([this.buffer, chunk]);
      this.drainMessages();
    });

    this.process.stderr.on("data", (chunk: Buffer) => {
      this.stderr += chunk.toString("utf8");
    });

    this.process.on("exit", (code, signal) => {
      const error = new Error(
        `vize lsp exited unexpectedly (code=${code ?? "null"}, signal=${signal ?? "null"})\n${this.stderr}`.trim(),
      );

      for (const pending of this.pending.values()) {
        clearTimeout(pending.timeout);
        pending.reject(error);
      }
      this.pending.clear();

      for (const notification of this.notifications) {
        clearTimeout(notification.timeout);
        notification.reject(error);
      }
      this.notifications.length = 0;
    });
  }

  /** Operating-system process id for budget-enforced latency/RSS measurements. */
  get processId(): number {
    assert.ok(this.process.pid, "vize lsp process id is unavailable");
    return this.process.pid;
  }

  /**
   * Server stderr captured so far. Timeout errors raised by this session
   * already embed it; suites with their own deadlines (for example the churn
   * hard timeout) read it here so a hang failure still carries server logs.
   */
  get stderrText(): string {
    return this.stderr;
  }

  /**
   * Initialize the session with one workspace folder (existing callers) or
   * several (true multi-root sessions, #3240).
   *
   * The LSP spec prefers `workspaceFolders` over the deprecated `rootUri`, so
   * multi-root sessions send `rootUri: null` to prove the server consumes the
   * folder list itself; single-folder sessions keep the historical `rootUri`
   * for parity with editors that still populate both fields.
   */
  async initialize(
    workspaceDir: string | readonly string[],
    initializationOptions: LspInitializationOptions = {
      editor: true,
      typecheck: true,
    },
  ): Promise<unknown> {
    const workspaceDirs = typeof workspaceDir === "string" ? [workspaceDir] : [...workspaceDir];
    assert.ok(workspaceDirs.length > 0, "initialize requires at least one workspace folder");

    const result = await this.request("initialize", {
      processId: process.pid,
      rootUri: workspaceDirs.length === 1 ? pathToFileURL(workspaceDirs[0]).href : null,
      capabilities: {
        textDocument: {
          completion: {
            completionItem: {
              documentationFormat: ["markdown", "plaintext"],
            },
          },
        },
      },
      initializationOptions,
      workspaceFolders: workspaceDirs.map((dir) => ({
        uri: pathToFileURL(dir).href,
        name: path.basename(dir),
      })),
    });

    this.notify("initialized", {});
    return result;
  }

  request(
    method: string,
    params: unknown,
    timeoutMs = 30000,
    following: (id: number) => JsonRpcMessage[] = () => [],
  ): Promise<unknown> {
    const id = ++this.nextId;

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`Timed out waiting for ${method}\n${this.stderr}`.trim()));
      }, timeoutMs);

      this.pending.set(id, { resolve, reject, method, timeout });
      this.send({ jsonrpc: "2.0", id, method, params }, ...following(id));
    });
  }

  notify(method: string, params: unknown): void {
    this.send({ jsonrpc: "2.0", method, params });
  }

  waitForNotification(
    method: string,
    predicate?: (params: unknown) => boolean,
    timeoutMs = 30000,
  ): Promise<unknown> {
    const backlogIndex = this.notificationBacklog.findIndex(
      (notification) =>
        notification.method === method && (predicate == null || predicate(notification.params)),
    );
    if (backlogIndex >= 0) {
      const [{ params }] = this.notificationBacklog.splice(backlogIndex, 1);
      return Promise.resolve(params);
    }

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        const index = this.notifications.findIndex(
          (notification) => notification.resolve === resolve,
        );
        if (index >= 0) {
          this.notifications.splice(index, 1);
        }
        reject(new Error(`Timed out waiting for notification ${method}\n${this.stderr}`.trim()));
      }, timeoutMs);

      this.notifications.push({
        method,
        predicate,
        resolve,
        reject,
        timeout,
      });
    });
  }

  async shutdown(): Promise<void> {
    if (this.process.killed) {
      return;
    }

    try {
      await this.request("shutdown", undefined, 10000);
    } finally {
      const exited = this.waitForExit(5000);
      this.notify("exit", undefined);
      this.process.stdin.end();
      if (!(await exited)) {
        await this.kill().catch(() => undefined);
      }
    }
  }

  /**
   * Abruptly terminate the production server and wait until the child is
   * actually gone. Restart oracles use this instead of racing a fresh server
   * against an old process that has only received a signal.
   */
  async kill(signal: NodeJS.Signals = "SIGKILL"): Promise<void> {
    if (this.process.exitCode != null || this.process.signalCode != null) {
      return;
    }

    const exited = this.waitForExit(5000);
    assert.ok(this.process.kill(signal), `Failed to send ${signal} to vize lsp`);
    assert.ok(await exited, `Timed out waiting for vize lsp to exit after ${signal}`);
  }

  /** Resolve `true` once the child exits, or `false` if it is still alive after `timeoutMs`. */
  private waitForExit(timeoutMs: number): Promise<boolean> {
    if (this.process.exitCode != null || this.process.signalCode != null) {
      return Promise.resolve(true);
    }

    return new Promise((resolve) => {
      const onExit = () => {
        clearTimeout(timeout);
        resolve(true);
      };
      const timeout = setTimeout(() => {
        this.process.off("exit", onExit);
        resolve(false);
      }, timeoutMs);

      this.process.once("exit", onExit);
    });
  }

  private send(...messages: JsonRpcMessage[]): void {
    const frame = messages.map((message) => frameMessage(message)).join("");
    this.process.stdin.write(frame, "utf8");
  }

  private drainMessages(): void {
    while (true) {
      const headerEnd = this.buffer.indexOf("\r\n\r\n");
      if (headerEnd < 0) {
        return;
      }

      const header = this.buffer.subarray(0, headerEnd).toString("utf8");
      const lengthMatch = header.match(/Content-Length:\s*(\d+)/i);
      assert.ok(lengthMatch, `missing Content-Length header: ${header}`);

      const bodyLength = Number(lengthMatch[1]);
      const frameLength = headerEnd + 4 + bodyLength;
      if (this.buffer.length < frameLength) {
        return;
      }

      const body = this.buffer.subarray(headerEnd + 4, frameLength).toString("utf8");
      this.buffer = this.buffer.subarray(frameLength);

      const message = JSON.parse(body) as JsonRpcMessage;
      this.dispatch(message);
    }
  }

  private dispatch(message: JsonRpcMessage): void {
    if (typeof message.id === "number" && message.method == null) {
      const pending = this.pending.get(message.id);
      if (!pending) {
        return;
      }

      clearTimeout(pending.timeout);
      this.pending.delete(message.id);

      if (message.error) {
        pending.reject(new LspRequestError(message.id, pending.method, message.error));
        return;
      }

      pending.resolve(message.result);
      return;
    }

    if (message.method != null && typeof message.id === "number") {
      this.send({
        jsonrpc: "2.0",
        id: message.id,
        error: {
          code: -32601,
          message: `client does not implement ${message.method}`,
        },
      });
      return;
    }

    if (message.method == null) {
      return;
    }

    for (const observer of this.notificationObservers) {
      observer(message.method, message.params);
    }

    const index = this.notifications.findIndex(
      (notification) =>
        notification.method === message.method &&
        (notification.predicate == null || notification.predicate(message.params)),
    );

    if (index < 0) {
      this.notificationBacklog.push({
        method: message.method,
        params: message.params,
      });
      return;
    }

    const [notification] = this.notifications.splice(index, 1);
    clearTimeout(notification.timeout);
    notification.resolve(message.params);
  }
}

function frameMessage(message: JsonRpcMessage): string {
  const payload = JSON.stringify(message);
  return `Content-Length: ${Buffer.byteLength(payload, "utf8")}\r\n\r\n${payload}`;
}
