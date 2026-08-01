import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import test from "node:test";

import { createNuxtLintCheckerVitePlugin } from "./vite.ts";
import type { ResolvedVizeNuxtLintCheckerOptions } from "./options.ts";
import type { NuxtLintCheckerResult, NuxtLintCheckerTask } from "./worker.ts";

const flush = () => new Promise<void>((resolve) => setImmediate(resolve));

function result(overrides: Partial<NuxtLintCheckerResult> = {}): NuxtLintCheckerResult {
  return {
    diagnosticCount: 0,
    hasErrors: false,
    hasWarnings: false,
    output: "",
    ...overrides,
  };
}

function options(
  overrides: Partial<ResolvedVizeNuxtLintCheckerOptions> = {},
): ResolvedVizeNuxtLintCheckerOptions {
  return {
    cache: true,
    emitError: true,
    emitWarning: true,
    exclude: ["**/node_modules/**", "/project/.nuxt"],
    fix: false,
    formatter: "stylish",
    include: ["/project/app/**/*.{js,jsx,ts,tsx,vue}"],
    lintOnStart: true,
    ...overrides,
  };
}

function harness(responses: NuxtLintCheckerResult[] = [result()]) {
  const watcher = new EventEmitter() as EventEmitter & { add(path: string): void; added: string[] };
  watcher.added = [];
  watcher.add = (file) => watcher.added.push(file);
  const httpServer = new EventEmitter();
  const messages: unknown[] = [];
  const errors: string[] = [];
  const warnings: string[] = [];
  const tasks: NuxtLintCheckerTask[] = [];
  let closed = 0;
  const runner = {
    close: async () => {
      closed += 1;
    },
    run: async (task: NuxtLintCheckerTask) => {
      tasks.push(task);
      return responses.shift() ?? result();
    },
  };
  const server = {
    config: {
      logger: {
        error: (message: string) => errors.push(message),
        warn: (message: string) => warnings.push(message),
      },
    },
    httpServer,
    watcher,
    ws: { send: (message: unknown) => messages.push(message) },
  };
  return {
    errors,
    get closed() {
      return closed;
    },
    httpServer,
    messages,
    runner,
    server,
    tasks,
    warnings,
    watcher,
  };
}

void test("Vite addon lints on start in a worker and watches the generated config", async () => {
  const state = harness();
  const plugin = createNuxtLintCheckerVitePlugin(
    {
      configFile: "/project/.nuxt/oxlint.config.json",
      options: options(),
      rootDir: "/project",
    },
    { createRunner: () => state.runner },
  );

  assert.equal(plugin.apply, "serve");
  plugin.configureServer?.(state.server as never);
  await flush();
  assert.deepEqual(state.watcher.added, ["/project/.nuxt/oxlint.config.json"]);
  assert.deepEqual(
    state.tasks.map((task) => task.targets),
    [["/project/app/**/*.{js,jsx,ts,tsx,vue}"]],
  );

  state.httpServer.emit("close");
  await flush();
  assert.equal(state.closed, 1);
});

void test("Vite addon lints changed files with cache and full includes without it", async () => {
  for (const cache of [true, false]) {
    const state = harness([result(), result()]);
    const plugin = createNuxtLintCheckerVitePlugin(
      {
        configFile: "/project/.nuxt/oxlint.config.json",
        options: options({ cache, lintOnStart: false }),
        rootDir: "/project",
      },
      { createRunner: () => state.runner },
    );
    plugin.configureServer?.(state.server as never);
    state.watcher.emit("change", "/project/app/pages/index.vue");
    state.watcher.emit("change", "/project/app/node_modules/ignored.ts");
    await flush();
    assert.deepEqual(
      state.tasks.map((task) => task.targets),
      [cache ? ["/project/app/pages/index.vue"] : ["/project/app/**/*.{js,jsx,ts,tsx,vue}"]],
    );

    state.watcher.emit("change", "/project/.nuxt/oxlint.config.json");
    await flush();
    assert.deepEqual(state.tasks[1].targets, ["/project/app/**/*.{js,jsx,ts,tsx,vue}"]);
  }
});

void test("Vite addon reports terminal diagnostics, opens and clears the overlay", async () => {
  const state = harness([
    result({ diagnosticCount: 1, hasErrors: true, output: "error output\n" }),
    result(),
    result({ diagnosticCount: 1, hasWarnings: true, output: "warning output\n" }),
  ]);
  const plugin = createNuxtLintCheckerVitePlugin(
    {
      configFile: "/project/.nuxt/oxlint.config.json",
      options: options({ lintOnStart: false }),
      rootDir: "/project",
    },
    { createRunner: () => state.runner },
  );
  plugin.configureServer?.(state.server as never);

  state.watcher.emit("change", "/project/app/error.vue");
  await flush();
  state.watcher.emit("change", "/project/app/clean.vue");
  await flush();
  state.watcher.emit("change", "/project/app/warning.vue");
  await flush();

  assert.deepEqual(state.errors, ["error output\n"]);
  assert.deepEqual(state.warnings, ["warning output\n"]);
  assert.deepEqual(
    state.messages.map((message) => (message as { type: string }).type),
    ["error", "update", "error"],
  );
  assert.match(JSON.stringify(state.messages[0]), /error output/u);
  assert.match(JSON.stringify(state.messages[2]), /warning output/u);
});

void test("Vite addon turns worker failures into actionable terminal and overlay errors", async () => {
  const state = harness();
  state.runner.run = async () => {
    throw new Error("oxlint disappeared");
  };
  const plugin = createNuxtLintCheckerVitePlugin(
    {
      configFile: "/project/.nuxt/oxlint.config.json",
      options: options(),
      rootDir: "/project",
    },
    { createRunner: () => state.runner },
  );
  plugin.configureServer?.(state.server as never);
  await flush();

  assert.match(state.errors[0], /Nuxt lint checker failed: oxlint disappeared/u);
  assert.equal((state.messages[0] as { type: string }).type, "error");
});

void test("Vite addon stays silent when shutdown cancels an active pass", async () => {
  const state = harness();
  let rejectRun = (_error: Error): void => {};
  state.runner.run = () =>
    new Promise((_resolve, reject) => {
      rejectRun = reject;
    });
  const plugin = createNuxtLintCheckerVitePlugin(
    {
      configFile: "/project/.nuxt/oxlint.config.json",
      options: options(),
      rootDir: "/project",
    },
    { createRunner: () => state.runner },
  );
  plugin.configureServer?.(state.server as never);
  await flush();
  state.httpServer.emit("close");
  rejectRun(new Error("worker closed"));
  await flush();

  assert.deepEqual(state.errors, []);
  assert.deepEqual(state.messages, []);
});
