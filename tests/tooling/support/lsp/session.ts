import assert from "node:assert/strict";
import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { resolveVizeLaunchCommand } from "./launch.ts";
import { root } from "./paths.ts";
import type { JsonRpcId, JsonRpcMessage, LspInitializationOptions } from "./protocol.ts";

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

  async initialize(
    workspaceDir: string,
    initializationOptions: LspInitializationOptions = {
      editor: true,
      typecheck: true,
    },
  ): Promise<unknown> {
    const result = await this.request("initialize", {
      processId: process.pid,
      rootUri: pathToFileURL(workspaceDir).href,
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
      workspaceFolders: [
        {
          uri: pathToFileURL(workspaceDir).href,
          name: path.basename(workspaceDir),
        },
      ],
    });

    this.notify("initialized", {});
    return result;
  }

  request(method: string, params: unknown, timeoutMs = 30000): Promise<unknown> {
    const id = ++this.nextId;

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`Timed out waiting for ${method}\n${this.stderr}`.trim()));
      }, timeoutMs);

      this.pending.set(id, { resolve, reject, method, timeout });
      this.send({ jsonrpc: "2.0", id, method, params });
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
      this.notify("exit", undefined);
      this.process.stdin.end();
      await new Promise<void>((resolve) => {
        const timeout = setTimeout(() => {
          this.process.kill("SIGKILL");
        }, 5000);

        this.process.once("exit", () => {
          clearTimeout(timeout);
          resolve();
        });
      });
    }
  }

  private send(message: JsonRpcMessage): void {
    const payload = JSON.stringify(message);
    const frame = `Content-Length: ${Buffer.byteLength(payload, "utf8")}\r\n\r\n${payload}`;
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
        pending.reject(new Error(`${pending.method}: ${message.error.message}`));
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
