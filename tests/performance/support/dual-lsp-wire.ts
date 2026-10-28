import assert from "node:assert/strict";
import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { performance } from "node:perf_hooks";

type JsonRpcMessage = {
  jsonrpc: "2.0";
  id?: number;
  method?: string;
  params?: unknown;
  result?: unknown;
  error?: { code: number; message: string; data?: unknown };
};

export type WireLaunch = {
  name: string;
  command: string;
  args: string[];
  cwd: string;
  env?: NodeJS.ProcessEnv;
};

export type ClientOperation = {
  kind: "request" | "notification";
  method: string;
  params: unknown;
};

export type WireNotification = {
  sequence: number;
  receivedAtMs: number;
  method: string;
  params: unknown;
};

export class BoundedNotificationHistory {
  private readonly entries: WireNotification[] = [];
  private readonly limit: number;
  private highestDroppedSequence = 0;

  constructor(limit = 200) {
    assert.ok(
      Number.isSafeInteger(limit) && limit > 0,
      "notification history limit must be positive",
    );
    this.limit = limit;
  }

  push(notification: WireNotification): void {
    const latest = this.entries.at(-1);
    assert.ok(
      latest == null || notification.sequence > latest.sequence,
      "notification sequences must increase",
    );
    this.entries.push(notification);
    if (this.entries.length <= this.limit) return;
    const dropped = this.entries.shift();
    assert.ok(dropped);
    this.highestDroppedSequence = dropped.sequence;
  }

  find(
    method: string,
    afterSequence: number,
    predicate: (params: unknown) => boolean,
  ): WireNotification | undefined {
    this.assertCursorRetained(afterSequence);
    return this.entries.find(
      (item) => item.sequence > afterSequence && item.method === method && predicate(item.params),
    );
  }

  latest(method: string, afterSequence: number): WireNotification | undefined {
    this.assertCursorRetained(afterSequence);
    return this.entries.findLast((item) => item.sequence > afterSequence && item.method === method);
  }

  since(method: string, afterSequence: number): readonly WireNotification[] {
    this.assertCursorRetained(afterSequence);
    return this.entries.filter((item) => item.sequence > afterSequence && item.method === method);
  }

  recent(count: number): readonly WireNotification[] {
    return this.entries.slice(-count);
  }

  private assertCursorRetained(afterSequence: number): void {
    assert.ok(
      afterSequence >= this.highestDroppedSequence,
      `notification cursor ${afterSequence} predates evicted sequence ${this.highestDroppedSequence}`,
    );
  }
}

type NotificationObserver = (notification: WireNotification) => void | Promise<void>;

const defaultTimeoutMs = 60_000;
const stderrLimit = 64 * 1024;

export function supportsLspMethod(capabilities: Record<string, unknown>, method: string): boolean {
  const capability =
    method === "textDocument/completion"
      ? "completionProvider"
      : method === "textDocument/hover"
        ? "hoverProvider"
        : method === "textDocument/definition"
          ? "definitionProvider"
          : method === "textDocument/references"
            ? "referencesProvider"
            : method === "textDocument/documentSymbol"
              ? "documentSymbolProvider"
              : method === "textDocument/rename" || method === "textDocument/prepareRename"
                ? "renameProvider"
                : null;
  if (capability == null) return false;
  const advertised = capabilities[capability];
  if (method === "textDocument/prepareRename") {
    return (
      advertised != null &&
      typeof advertised === "object" &&
      (advertised as { prepareProvider?: boolean }).prepareProvider === true
    );
  }
  return advertised != null && advertised !== false;
}

/** One strict JSON-RPC stdio transport shared by every measured server process. */
export class DualLspWire {
  readonly launch: WireLaunch;
  private readonly child: ChildProcessWithoutNullStreams;
  private readonly pending = new Map<
    number,
    {
      method: string;
      resolve: (value: unknown) => void;
      reject: (error: Error) => void;
      timer: NodeJS.Timeout;
    }
  >();
  private readonly notifications = new BoundedNotificationHistory();
  private readonly notificationObservers: NotificationObserver[] = [];
  private readonly operations: ClientOperation[] = [];
  private buffer = Buffer.alloc(0);
  private nextId = 0;
  private notificationSequence = 0;
  private shuttingDown = false;
  private stderr = "";
  private capabilities: Record<string, unknown> = {};

  constructor(launch: WireLaunch) {
    this.launch = launch;
    this.child = spawn(launch.command, launch.args, {
      cwd: launch.cwd,
      env: { ...process.env, ...launch.env },
      stdio: ["pipe", "pipe", "pipe"],
    });
    this.child.stdout.on("data", (chunk: Buffer) => {
      this.buffer = Buffer.concat([this.buffer, chunk]);
      this.drain();
    });
    this.child.stderr.on("data", (chunk: Buffer) => {
      this.stderr = (this.stderr + chunk.toString("utf8")).slice(-stderrLimit);
    });
    this.child.on("error", (error) => this.rejectAll(error));
    this.child.on("exit", (code, signal) => {
      if (this.shuttingDown && code === 0 && signal == null) return;
      this.rejectAll(
        new Error(
          `${launch.name} exited (code=${code ?? "null"}, signal=${signal ?? "null"})\n${this.stderr}`.trim(),
        ),
      );
    });
  }

  get processId(): number {
    assert.ok(this.child.pid, `${this.launch.name} has no process id`);
    return this.child.pid;
  }

  get notificationCursor(): number {
    return this.notificationSequence;
  }

  get stderrText(): string {
    return this.stderr;
  }

  clientOperations(): readonly ClientOperation[] {
    return this.operations;
  }

  addNotificationObserver(observer: NotificationObserver): void {
    this.notificationObservers.push(observer);
  }

  async initialize(params: unknown, track = true): Promise<unknown> {
    const result = (await this.request("initialize", params, defaultTimeoutMs, track)) as {
      capabilities?: Record<string, unknown>;
    } | null;
    this.capabilities = result?.capabilities ?? {};
    this.notify("initialized", {}, track);
    return result;
  }

  supports(method: string): boolean {
    return supportsLspMethod(this.capabilities, method);
  }

  request(
    method: string,
    params: unknown,
    timeoutMs = defaultTimeoutMs,
    track = true,
  ): Promise<unknown> {
    const id = ++this.nextId;
    if (track) this.operations.push({ kind: "request", method, params });
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(
          new Error(`${this.launch.name} timed out waiting for ${method}\n${this.stderr}`.trim()),
        );
      }, timeoutMs);
      this.pending.set(id, { method, resolve, reject, timer });
      this.send({ jsonrpc: "2.0", id, method, params });
    });
  }

  notify(method: string, params: unknown, track = true): void {
    if (track) this.operations.push({ kind: "notification", method, params });
    this.send({ jsonrpc: "2.0", method, params });
  }

  async waitForNotification(
    method: string,
    afterSequence: number,
    predicate: (params: unknown) => boolean,
    timeoutMs = defaultTimeoutMs,
  ): Promise<WireNotification> {
    const existing = this.notifications.find(method, afterSequence, predicate);
    if (existing) return existing;
    return await new Promise((resolve, reject) => {
      let observer: NotificationObserver;
      const timer = setTimeout(() => {
        const index = this.notificationObservers.indexOf(observer);
        if (index >= 0) this.notificationObservers.splice(index, 1);
        reject(
          new Error(
            `${this.launch.name} timed out waiting for ${method} after sequence ${afterSequence}; ` +
              `recent=${JSON.stringify(this.notifications.recent(10))}\n${this.stderr}`.trim(),
          ),
        );
      }, timeoutMs);
      observer = (item) => {
        if (item.sequence <= afterSequence || item.method !== method || !predicate(item.params))
          return;
        clearTimeout(timer);
        const index = this.notificationObservers.indexOf(observer);
        if (index >= 0) this.notificationObservers.splice(index, 1);
        resolve(item);
      };
      this.notificationObservers.push(observer);
    });
  }

  latestNotification(method: string, afterSequence = 0): WireNotification | undefined {
    return this.notifications.latest(method, afterSequence);
  }

  notificationsSince(method: string, afterSequence: number): readonly WireNotification[] {
    return this.notifications.since(method, afterSequence);
  }

  async observeQuiet(
    method: string,
    afterSequence: number,
    quietMs: number,
    timeoutMs: number,
  ): Promise<void> {
    await new Promise<void>((resolve, reject) => {
      let observer: NotificationObserver;
      let quietTimer: NodeJS.Timeout | undefined;
      let hardTimer: NodeJS.Timeout | undefined;
      const finish = (error?: Error) => {
        if (quietTimer) clearTimeout(quietTimer);
        if (hardTimer) clearTimeout(hardTimer);
        const index = this.notificationObservers.indexOf(observer);
        if (index >= 0) this.notificationObservers.splice(index, 1);
        if (error) reject(error);
        else resolve();
      };
      const arm = () => {
        clearTimeout(quietTimer);
        quietTimer = setTimeout(() => finish(), quietMs);
      };
      observer = (item) => {
        if (item.sequence > afterSequence && item.method === method) arm();
      };
      this.notificationObservers.push(observer);
      hardTimer = setTimeout(
        () => finish(new Error(`${this.launch.name} ${method} never became quiet`)),
        timeoutMs,
      );
      arm();
    });
  }

  async shutdown(): Promise<void> {
    if (this.child.exitCode != null || this.child.signalCode != null) return;
    this.shuttingDown = true;
    try {
      await this.request("shutdown", null, 10_000, false);
      this.notify("exit", null, false);
      this.child.stdin.end();
      await this.waitForExit(5_000);
    } catch (error) {
      this.child.kill("SIGKILL");
      await this.waitForExit(5_000).catch(() => undefined);
      throw error;
    }
  }

  async kill(): Promise<void> {
    if (this.child.exitCode != null || this.child.signalCode != null) return;
    this.shuttingDown = true;
    this.child.kill("SIGKILL");
    await this.waitForExit(5_000).catch(() => undefined);
  }

  private drain(): void {
    while (true) {
      const headerEnd = this.buffer.indexOf("\r\n\r\n");
      if (headerEnd < 0) return;
      const header = this.buffer.subarray(0, headerEnd).toString("ascii");
      const match = /(?:^|\r\n)Content-Length:\s*(\d+)/i.exec(header);
      if (!match)
        throw new Error(`${this.launch.name} emitted an LSP frame without Content-Length`);
      const length = Number(match[1]);
      const bodyStart = headerEnd + 4;
      if (this.buffer.length < bodyStart + length) return;
      const body = this.buffer.subarray(bodyStart, bodyStart + length).toString("utf8");
      this.buffer = this.buffer.subarray(bodyStart + length);
      this.dispatch(JSON.parse(body) as JsonRpcMessage);
    }
  }

  private dispatch(message: JsonRpcMessage): void {
    if (
      message.id != null &&
      ("result" in message || message.error != null) &&
      message.method == null
    ) {
      const pending = this.pending.get(message.id);
      if (!pending) return;
      clearTimeout(pending.timer);
      this.pending.delete(message.id);
      if (message.error) {
        pending.reject(
          new Error(`${this.launch.name} ${pending.method}: ${JSON.stringify(message.error)}`),
        );
      } else {
        pending.resolve(message.result);
      }
      return;
    }
    if (message.method == null) return;
    if (message.id != null) {
      void this.answerServerRequest(message);
      return;
    }
    const notification = {
      sequence: ++this.notificationSequence,
      receivedAtMs: performance.now(),
      method: message.method,
      params: message.params,
    };
    this.notifications.push(notification);
    const observers = this.notificationObservers.slice();
    for (const observer of observers) {
      void Promise.resolve(observer(notification)).catch((error) => this.rejectAll(error as Error));
    }
  }

  private async answerServerRequest(message: JsonRpcMessage): Promise<void> {
    let result: unknown = null;
    if (message.method === "workspace/configuration") {
      const items =
        (message.params as { items?: Array<{ section?: string }> } | undefined)?.items ?? [];
      result = items.map((item) =>
        item.section === "formattingOptions" ? { insertSpaces: true, tabSize: 2 } : null,
      );
    } else if (message.method === "workspace/workspaceFolders") {
      result = null;
    } else if (message.method === "workspace/applyEdit") {
      result = { applied: false, failureReason: "benchmark never accepts server-initiated edits" };
    }
    this.send({ jsonrpc: "2.0", id: message.id, result });
  }

  private send(message: JsonRpcMessage): void {
    const body = JSON.stringify(message);
    this.child.stdin.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
  }

  private rejectAll(error: Error): void {
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timer);
      pending.reject(error);
    }
    this.pending.clear();
  }

  private async waitForExit(timeoutMs: number): Promise<void> {
    if (this.child.exitCode != null || this.child.signalCode != null) return;
    await new Promise<void>((resolve, reject) => {
      const timer = setTimeout(
        () => reject(new Error(`${this.launch.name} did not exit`)),
        timeoutMs,
      );
      this.child.once("exit", () => {
        clearTimeout(timer);
        resolve();
      });
    });
  }
}
