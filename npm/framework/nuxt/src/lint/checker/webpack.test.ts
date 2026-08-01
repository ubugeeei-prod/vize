import assert from "node:assert/strict";
import test from "node:test";

import { createNuxtLintCheckerWebpackPlugin } from "./webpack.ts";
import type { ResolvedVizeNuxtLintCheckerOptions } from "./options.ts";
import type { NuxtLintCheckerResult, NuxtLintCheckerTask } from "./worker.ts";

type SyncHook = (compiler: FakeCompiler) => void;
type AsyncHook = (compilation: FakeCompilation) => Promise<void>;

interface FakeCompilation {
  errors: Error[];
  fileDependencies: Set<string>;
  warnings: Error[];
}

function resolved(
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

function ok(overrides: Partial<NuxtLintCheckerResult> = {}): NuxtLintCheckerResult {
  return {
    diagnosticCount: 0,
    hasErrors: false,
    hasWarnings: false,
    output: "",
    ...overrides,
  };
}

class FakeCompiler {
  modifiedFiles: Set<string> | undefined;
  watchRunHook: SyncHook | undefined;
  afterCompileHook: AsyncHook | undefined;
  watchCloseHook: (() => void) | undefined;
  hooks = {
    watchRun: { tap: (_name: string, hook: SyncHook) => (this.watchRunHook = hook) },
    afterCompile: {
      tapPromise: (_name: string, hook: AsyncHook) => (this.afterCompileHook = hook),
    },
    watchClose: { tap: (_name: string, hook: () => void) => (this.watchCloseHook = hook) },
  };
}

function harness(options: ResolvedVizeNuxtLintCheckerOptions) {
  const compiler = new FakeCompiler();
  const tasks: NuxtLintCheckerTask[] = [];
  const responses: NuxtLintCheckerResult[] = [];
  let closed = 0;
  const runner = {
    close: async () => {
      closed += 1;
    },
    run: async (task: NuxtLintCheckerTask) => {
      tasks.push(task);
      return responses.shift() ?? ok();
    },
  };
  createNuxtLintCheckerWebpackPlugin(
    { configFile: "/project/.nuxt/oxlint.config.json", options, rootDir: "/project" },
    { createRunner: () => runner },
  ).apply(compiler as never);
  return {
    compiler,
    get closed() {
      return closed;
    },
    responses,
    tasks,
  };
}

function compilation(): FakeCompilation {
  return { errors: [], fileDependencies: new Set(), warnings: [] };
}

void test("webpack addon starts worker lint before compilation completes and feeds its overlay", async () => {
  const state = harness(resolved());
  state.responses.push(ok({ diagnosticCount: 2, hasErrors: true, output: "lint failed\n" }));
  state.compiler.watchRunHook?.(state.compiler);
  assert.equal(state.tasks.length, 1, "worker starts at watchRun rather than afterCompile");

  const built = compilation();
  await state.compiler.afterCompileHook?.(built);
  assert.deepEqual(state.tasks[0].targets, ["/project/app/**/*.{js,jsx,ts,tsx,vue}"]);
  assert.equal(built.errors.length, 1);
  assert.match(built.errors[0].message, /lint failed/u);
  assert.deepEqual([...built.fileDependencies], ["/project/.nuxt/oxlint.config.json"]);
});

void test("webpack addon uses modified files with cache and reruns all on config changes", async () => {
  const state = harness(resolved({ lintOnStart: false }));
  state.compiler.modifiedFiles = new Set([
    "/project/app/pages/index.vue",
    "/project/app/node_modules/ignored.ts",
  ]);
  state.compiler.watchRunHook?.(state.compiler);
  await state.compiler.afterCompileHook?.(compilation());
  assert.deepEqual(
    state.tasks.map((task) => task.targets),
    [["/project/app/pages/index.vue"]],
  );

  state.compiler.modifiedFiles = new Set(["/project/.nuxt/oxlint.config.json"]);
  state.compiler.watchRunHook?.(state.compiler);
  await state.compiler.afterCompileHook?.(compilation());
  assert.deepEqual(state.tasks[1].targets, ["/project/app/**/*.{js,jsx,ts,tsx,vue}"]);
});

void test("webpack addon honours lintOnStart false, warning output, and closes its worker", async () => {
  const state = harness(resolved({ lintOnStart: false }));
  state.compiler.modifiedFiles = undefined;
  state.compiler.watchRunHook?.(state.compiler);
  const first = compilation();
  await state.compiler.afterCompileHook?.(first);
  assert.equal(state.tasks.length, 0);

  state.responses.push(ok({ diagnosticCount: 1, hasWarnings: true, output: "lint warning\n" }));
  state.compiler.modifiedFiles = new Set(["/project/app/page.vue"]);
  state.compiler.watchRunHook?.(state.compiler);
  const second = compilation();
  await state.compiler.afterCompileHook?.(second);
  assert.equal(second.warnings.length, 1);
  assert.match(second.warnings[0].message, /lint warning/u);

  state.compiler.watchCloseHook?.();
  await new Promise<void>((resolve) => setImmediate(resolve));
  assert.equal(state.closed, 1);
});
