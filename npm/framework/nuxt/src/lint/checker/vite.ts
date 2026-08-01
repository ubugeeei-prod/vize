import path from "node:path";

import { matchesNuxtLintCheckerFile } from "./filter.ts";
import type { ResolvedVizeNuxtLintCheckerOptions } from "./options.ts";
import {
  NuxtLintCheckerWorker,
  type NuxtLintCheckerResult,
  type NuxtLintCheckerTask,
} from "./worker.ts";

const PLUGIN_NAME = "vize:nuxt-lint-checker";

interface CheckerWatcher {
  add(file: string): void;
  on(event: "add" | "change" | "unlink", listener: (file: string) => void): unknown;
}

interface CheckerServer {
  config: { logger: { error(message: string): void; warn(message: string): void } };
  httpServer?: { once(event: "close", listener: () => void): unknown } | null;
  watcher: CheckerWatcher;
  ws: { send(message: unknown): void };
}

export interface NuxtLintCheckerVitePlugin {
  apply: "serve";
  enforce: "post";
  name: typeof PLUGIN_NAME;
  configureServer(server: CheckerServer): void;
}

export interface NuxtLintCheckerViteOptions {
  configFile: string;
  options: ResolvedVizeNuxtLintCheckerOptions;
  rootDir: string;
}

export interface NuxtLintCheckerRunner {
  close(): void | Promise<void>;
  run(task: NuxtLintCheckerTask): Promise<NuxtLintCheckerResult>;
}

export interface NuxtLintCheckerViteDependencies {
  createRunner?: () => NuxtLintCheckerRunner;
  oxlintEntrypoint?: string;
}

function overlayError(output: string): unknown {
  return {
    type: "error",
    err: {
      message: "Vize lint checker found diagnostics",
      stack: output,
      plugin: PLUGIN_NAME,
    },
  };
}

class NuxtLintChangeCollector {
  private readonly pendingFiles = new Set<string>();
  private pendingFull = false;
  private draining = false;
  private closed = false;
  private overlayVisible = false;
  private readonly config: NuxtLintCheckerViteOptions;
  private readonly runner: NuxtLintCheckerRunner;
  private readonly server: CheckerServer;
  private readonly oxlintEntrypoint: string | undefined;

  constructor(
    config: NuxtLintCheckerViteOptions,
    runner: NuxtLintCheckerRunner,
    server: CheckerServer,
    oxlintEntrypoint: string | undefined,
  ) {
    this.config = config;
    this.runner = runner;
    this.server = server;
    this.oxlintEntrypoint = oxlintEntrypoint;
  }

  full(): void {
    if (this.closed) return;
    this.pendingFull = true;
    this.pendingFiles.clear();
    this.schedule();
  }

  file(file: string): void {
    if (this.closed) return;
    if (path.resolve(file) === path.resolve(this.config.configFile)) {
      this.full();
      return;
    }
    if (!matchesNuxtLintCheckerFile(file, this.config.rootDir, this.config.options)) return;
    if (!this.config.options.cache) {
      this.full();
      return;
    }
    if (!this.pendingFull) this.pendingFiles.add(path.resolve(file));
    this.schedule();
  }

  async close(): Promise<void> {
    if (this.closed) return;
    this.closed = true;
    this.pendingFiles.clear();
    await this.runner.close();
  }

  private schedule(): void {
    if (this.draining) return;
    this.draining = true;
    queueMicrotask(() => void this.drain());
  }

  private async drain(): Promise<void> {
    while (!this.closed && (this.pendingFull || this.pendingFiles.size > 0)) {
      const targets = this.pendingFull
        ? [...this.config.options.include]
        : [...this.pendingFiles].sort();
      this.pendingFull = false;
      this.pendingFiles.clear();
      try {
        const result = await this.runner.run(this.task(targets));
        if (!this.closed) this.report(result);
      } catch (error) {
        if (!this.closed) this.reportFailure(error);
      }
    }
    this.draining = false;
    // A request can land after the loop condition but before the flag reset.
    if (!this.closed && (this.pendingFull || this.pendingFiles.size > 0)) this.schedule();
  }

  private task(targets: string[]): NuxtLintCheckerTask {
    const options = this.config.options;
    return {
      configFile: this.config.configFile,
      cwd: this.config.rootDir,
      emitError: options.emitError,
      emitWarning: options.emitWarning,
      exclude: [...options.exclude],
      fix: options.fix,
      formatter: options.formatter,
      oxlintEntrypoint: this.oxlintEntrypoint,
      targets,
    };
  }

  private report(result: NuxtLintCheckerResult): void {
    if (result.output) {
      if (result.hasErrors) this.server.config.logger.error(result.output);
      else if (result.hasWarnings) this.server.config.logger.warn(result.output);
    }
    if (result.diagnosticCount > 0) {
      this.server.ws.send(overlayError(result.output));
      this.overlayVisible = true;
    } else if (this.overlayVisible) {
      // Vite clears its error overlay whenever an update payload arrives.
      this.server.ws.send({ type: "update", updates: [] });
      this.overlayVisible = false;
    }
  }

  private reportFailure(error: unknown): void {
    const message = `Nuxt lint checker failed: ${error instanceof Error ? error.message : String(error)}`;
    this.server.config.logger.error(message);
    this.server.ws.send(overlayError(message));
    this.overlayVisible = true;
  }
}

/** Create the client-side Vite dev plugin; it has no transform/render hooks. */
export function createNuxtLintCheckerVitePlugin(
  config: NuxtLintCheckerViteOptions,
  dependencies: NuxtLintCheckerViteDependencies = {},
): NuxtLintCheckerVitePlugin {
  return {
    name: PLUGIN_NAME,
    apply: "serve",
    enforce: "post",
    configureServer(server) {
      const runner = dependencies.createRunner?.() ?? new NuxtLintCheckerWorker();
      const collector = new NuxtLintChangeCollector(
        config,
        runner,
        server,
        dependencies.oxlintEntrypoint,
      );
      server.watcher.add(config.configFile);
      const changed = (file: string) => collector.file(file);
      server.watcher.on("add", changed);
      server.watcher.on("change", changed);
      server.watcher.on("unlink", changed);
      server.httpServer?.once("close", () => void collector.close());
      if (config.options.lintOnStart) collector.full();
    },
  };
}
