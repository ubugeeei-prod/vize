import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { appendFileSync, mkdirSync, mkdtempSync, realpathSync, writeFileSync } from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

import { FrameDecoder } from "./frames.ts";
import { settlePublication, settleReply } from "./dispatch.ts";
import {
  InstalledAliasError,
  validateInstalledLaunch,
  type Packet,
  type InstalledAliasLaunch,
  type Outcome,
  type Pending,
  type PublicationWait,
} from "./protocol-types.ts";
export type { Packet, InstalledAliasLaunch } from "./protocol-types.ts";
/** No launch resolution, provider fallback, packet filtering, or source compilation. */
export class InstalledAliasSession {
  readonly directory: string;
  readonly packets: Packet[] = [];
  readonly notifications: Packet[] = [];
  readonly outcomes: Outcome[] = [];
  readonly failures: string[] = [];
  private readonly launch: InstalledAliasLaunch;
  private readonly child: ChildProcessWithoutNullStreams;
  private readonly pending = new Map<number, Pending>();
  private readonly publications: PublicationWait[] = [];
  private readonly bytes = { client: 0, server: 0, stderr: 0 };
  private readonly closed: Promise<void>;
  private readonly decoder = new FrameDecoder();
  private nextId = 1;
  private fatal: Error | undefined;
  private parsingFailed = false;
  private didClose = false;
  private exitCode: number | null = null;
  private signal: NodeJS.Signals | null = null;
  private finishing: Promise<ReturnType<InstalledAliasSession["receipt"]>> | undefined;

  private constructor(launch: InstalledAliasLaunch) {
    validateInstalledLaunch(launch);
    this.launch = launch;
    mkdirSync(launch.outputRoot, { recursive: true });
    this.directory = mkdtempSync(path.join(launch.outputRoot, "session-"));
    for (const stream of ["client", "server", "stderr"] as const)
      writeFileSync(this.rawPath(stream), Buffer.alloc(0));
    writeFileSync(path.join(this.directory, "packets.ndjson"), "");
    this.child = spawn(
      launch.nodePath,
      ["--require", launch.custodyHookPath, launch.cliPath, "lsp", "--stdio"],
      {
        cwd: launch.projectRoot,
        env: launch.env,
        stdio: ["pipe", "pipe", "pipe"],
      },
    );
    this.child.stdout.on("data", (chunk: Buffer) => {
      this.capture("server", chunk);
      if (this.parsingFailed) return;
      try {
        this.decoder.push(chunk, (packet) => this.receive(packet));
      } catch (error) {
        this.parsingFailed = true;
        this.fail(error instanceof Error ? error : new Error(String(error)));
      }
    });
    this.child.stderr.on("data", (chunk: Buffer) => this.capture("stderr", chunk));
    this.child.on("error", (error) => this.fail(error));
    this.child.stdin.on("error", (error) => this.fail(error));
    this.closed = new Promise((resolve) =>
      this.child.once("close", (code, signal) => {
        this.didClose = true;
        this.exitCode = code;
        this.signal = signal;
        if (this.decoder.remaining)
          this.fail(new Error(`incomplete server frame: ${this.decoder.remaining} bytes`));
        this.rejectPending(new Error(`LSP process closed: ${code}/${signal}`));
        this.persist();
        resolve();
      }),
    );
    this.persist();
  }

  static async launch(launch: InstalledAliasLaunch): Promise<InstalledAliasSession> {
    const session = new InstalledAliasSession(launch);
    try {
      const response = await session.request("initialize", {
        processId: null,
        rootUri: pathToFileURL(realpathSync(launch.projectRoot)).href,
        capabilities: {},
        initializationOptions: {
          lint: launch.lint ?? false,
          typecheck: true,
          hover: true,
          crossFile: launch.crossFile,
        },
      });
      if (!response.result || typeof response.result !== "object" || Array.isArray(response.result))
        throw new Error("initialize result must be an object");
      session.notify("initialized", {});
      return session;
    } catch (error) {
      session.fail(error instanceof Error ? error : new Error(String(error)));
      await session.shutdown();
      throw new Error(`initialize failed; capture ${session.directory}`, { cause: error });
    }
  }

  request(method: string, params?: unknown): Promise<Packet> {
    return this.ask(method, params, false);
  }
  /** Only explicitly uncontracted observations may retain whole RPC error replies. */
  requestObserved(method: string, params?: unknown): Promise<Packet> {
    return this.ask(method, params, true);
  }
  private ask(method: string, params: unknown, observed: boolean): Promise<Packet> {
    const id = this.nextId++;
    const outcome: Outcome = {
      id,
      method,
      status: "pending",
      ...(observed ? { observed: true } : {}),
    };
    this.outcomes.push(outcome);
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        outcome.status = "timeout";
        outcome.error = `${method} (${id}) timed out after ${this.launch.timeoutMs}ms`;
        this.failures.push(outcome.error);
        this.persist();
        reject(new InstalledAliasError(outcome.error));
      }, this.launch.timeoutMs);
      this.pending.set(id, { resolve, reject, timer, outcome, observed });
      try {
        this.send({ jsonrpc: "2.0", id, method, ...(params === undefined ? {} : { params }) });
      } catch (error) {
        this.fail(error instanceof Error ? error : new Error(String(error)));
      }
    });
  }

  notify(method: string, params?: unknown): void {
    this.send({ jsonrpc: "2.0", method, ...(params === undefined ? {} : { params }) });
  }

  open(uri: string, text: string, version = 1): Promise<Packet> {
    return this.publish(uri, version, "textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version, text },
    });
  }

  change(uri: string, text: string, version: number): Promise<Packet> {
    return this.publish(uri, version, "textDocument/didChange", {
      textDocument: { uri, version },
      contentChanges: [{ text }],
    });
  }

  changeWithPublications(
    uri: string,
    text: string,
    version: number,
    count: number,
  ): Promise<Packet[]> {
    return this.publishMany(
      uri,
      version,
      "textDocument/didChange",
      {
        textDocument: { uri, version },
        contentChanges: [{ text }],
      },
      count,
    );
  }
  private publish(uri: string, version: number, method: string, params: unknown): Promise<Packet> {
    return this.publishMany(uri, version, method, params, 1).then((packets) => packets[0]);
  }
  /** One deadline covers the complete ordered publication sequence, including equal packets. */
  private publishMany(
    uri: string,
    version: number,
    method: string,
    params: unknown,
    count: number,
  ): Promise<Packet[]> {
    if (!Number.isSafeInteger(version) || version < 1) throw new Error("invalid document version");
    if (!Number.isSafeInteger(count) || count < 1) throw new Error("invalid publication count");
    return new Promise((resolve, reject) => {
      const wait: PublicationWait = {
        uri,
        version,
        after: this.notifications.length,
        packets: [],
        count,
        resolve,
        reject,
        timer: setTimeout(() => {
          const index = this.publications.indexOf(wait);
          if (index >= 0) this.publications.splice(index, 1);
          const error = new Error(
            `diagnostics ${uri} version ${version} timed out after ${this.launch.timeoutMs}ms`,
          );
          this.failures.push(error.message);
          this.persist();
          reject(error);
        }, this.launch.timeoutMs),
      };
      this.publications.push(wait);
      try {
        this.notify(method, params);
      } catch (error) {
        this.fail(error instanceof Error ? error : new Error(String(error)));
      }
    });
  }

  private rawPath(stream: keyof InstalledAliasSession["bytes"]): string {
    return path.join(this.directory, `${stream}.raw`);
  }
  private capture(stream: keyof InstalledAliasSession["bytes"], chunk: Buffer): void {
    appendFileSync(this.rawPath(stream), chunk);
    this.bytes[stream] += chunk.length;
  }
  private send(packet: Packet): void {
    if (this.didClose || this.fatal) throw this.fatal ?? new Error("LSP process is closed");
    const body = Buffer.from(JSON.stringify(packet), "utf8");
    const frame = Buffer.concat([
      Buffer.from(`Content-Length: ${body.length}\r\n\r\n`, "ascii"),
      body,
    ]);
    this.capture("client", frame);
    this.child.stdin.write(frame);
  }

  private receive(packet: Packet): void {
    this.packets.push(packet);
    appendFileSync(path.join(this.directory, "packets.ndjson"), `${JSON.stringify(packet)}\n`);
    if (typeof packet.method === "string" && packet.id === undefined) {
      this.notifications.push(packet);
      if (packet.method !== "textDocument/publishDiagnostics") return;
      settlePublication(packet, this.notifications.length, this.publications, this.failures);
      return;
    }
    settleReply(packet, this.pending, this.failures);
  }

  private rejectPending(error: Error): void {
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timer);
      pending.outcome.status = "transport-error";
      pending.outcome.error = error.message;
      pending.reject(error);
    }
    this.pending.clear();
    for (const wait of this.publications.splice(0)) {
      clearTimeout(wait.timer);
      wait.reject(error);
    }
  }
  private fail(error: Error): void {
    this.failures.push(error.message);
    this.fatal ??= error;
    this.rejectPending(error);
    this.persist();
  }
  receipt() {
    return {
      directory: this.directory,
      command: this.launch.nodePath,
      args: ["--require", this.launch.custodyHookPath, this.launch.cliPath, "lsp", "--stdio"],
      cwd: this.launch.projectRoot,
      pid: this.child.pid ?? null,
      closed: this.didClose,
      exitCode: this.exitCode,
      signal: this.signal,
      bytes: { ...this.bytes },
      remainingFrameBytes: this.decoder.remaining,
      packets: this.packets,
      notifications: this.notifications,
      outcomes: this.outcomes,
      failures: this.failures,
      parsingFailed: this.parsingFailed,
      clientCapture: "whole submitted stdin writes",
    };
  }
  private persist(): void {
    writeFileSync(
      path.join(this.directory, "receipt.json"),
      `${JSON.stringify(this.receipt(), null, 2)}\n`,
    );
  }
  shutdown(): Promise<ReturnType<InstalledAliasSession["receipt"]>> {
    this.finishing ??= this.finish();
    return this.finishing;
  }
  private async finish(): Promise<ReturnType<InstalledAliasSession["receipt"]>> {
    if (!this.didClose) {
      try {
        await this.request("shutdown");
      } catch (error) {
        this.failures.push(`shutdown: ${String(error)}`);
      }
      try {
        this.notify("exit");
      } catch (error) {
        this.failures.push(`exit: ${String(error)}`);
      }
      try {
        if (!this.launch.keepStdinOpenAfterExit) this.child.stdin.end();
      } catch (error) {
        this.failures.push(`stdin end: ${String(error)}`);
      }
      const kill = setTimeout(() => {
        this.failures.push("shutdown deadline: SIGKILL");
        this.child.kill("SIGKILL");
      }, this.launch.exitTimeoutMs ?? this.launch.timeoutMs);
      await this.closed;
      clearTimeout(kill);
    }
    this.persist();
    return this.receipt();
  }
}
