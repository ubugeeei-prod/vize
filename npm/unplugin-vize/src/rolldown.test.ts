import { test } from "node:test";
import "./test/setup.ts";
import vize from "./rolldown.ts";
import vizeRollup from "./rollup.ts";
import { packageRoot } from "./test/helpers.ts";

// Rolldown consumes Rollup-format plugins, and a Rollup-format hook may be either
// a bare function or an `{ handler }` object. Normalize so the test works against
// whichever shape the factory produces.
type RollupHook = unknown;

function resolveHook(hook: RollupHook): ((...args: never[]) => unknown) | undefined {
  if (typeof hook === "function") {
    return hook as (...args: never[]) => unknown;
  }
  if (hook && typeof hook === "object" && "handler" in hook) {
    const handler = (hook as { handler?: unknown }).handler;
    return typeof handler === "function" ? (handler as (...args: never[]) => unknown) : undefined;
  }
  return undefined;
}

const BASIC_SFC = `<template><div>Hello from Rolldown</div></template>\n<script setup>const n = 1</script>`;

const TS_SFC =
  `<template><div>Hello from Rolldown</div></template>\n` +
  `<script setup lang="ts">interface Props { msg: string }\nconst n: number = 1</script>`;

void test("rolldown default export is a factory returning the unplugin-vize plugin", (t) => {
  t.assert.strictEqual(typeof vize, "function");

  const plugin = vize({ isProduction: true, root: packageRoot });
  t.assert.ok(plugin && typeof plugin === "object");
  t.assert.strictEqual(plugin.name, "unplugin-vize");
  t.assert.ok(resolveHook(plugin.transform), "expected a transform hook");
});

void test("rolldown and rollup entries are the same underlying factory", (t) => {
  // Both src/rolldown.ts and src/rollup.ts re-export `vizeUnplugin.rollup`, so they
  // produce structurally identical Rollup-format plugin objects. (They are NOT the
  // same function reference because each module re-evaluates the rollup getter.)
  const rolldownPlugin = vize({ isProduction: true, root: packageRoot });
  const rollupPlugin = vizeRollup({ isProduction: true, root: packageRoot });

  t.assert.strictEqual(rolldownPlugin.name, rollupPlugin.name);
  t.assert.strictEqual(rolldownPlugin.name, "unplugin-vize");

  // Assert the hook surface is identical between the two entries.
  t.assert.deepStrictEqual(Object.keys(rolldownPlugin).sort(), Object.keys(rollupPlugin).sort());

  // The Rollup-format object actually present here exposes these hooks (all as
  // functions). Assert the ones that are really there, on both entries.
  for (const hookName of ["transform", "resolveId", "load", "transformInclude", "loadInclude"]) {
    t.assert.ok(
      resolveHook((rolldownPlugin as Record<string, unknown>)[hookName]),
      `rolldown plugin should expose a ${hookName} hook`,
    );
    t.assert.ok(
      resolveHook((rollupPlugin as Record<string, unknown>)[hookName]),
      `rollup plugin should expose a ${hookName} hook`,
    );
  }
});

void test("rolldown transformInclude matches .vue ids and skips plain JS", (t) => {
  const plugin = vize({ isProduction: true, root: packageRoot });
  const transformInclude = resolveHook(plugin.transformInclude);
  t.assert.ok(transformInclude, "expected a transformInclude hook");

  t.assert.strictEqual(transformInclude!.call({} as never, "/proj/App.vue" as never), true);
  t.assert.strictEqual(transformInclude!.call({} as never, "/proj/App.js" as never), false);
});

void test("rolldown transform compiles a basic SFC without leftover bundler runtime", async (t) => {
  const plugin = vize({ isProduction: true, root: packageRoot });
  const transform = resolveHook(plugin.transform);
  t.assert.ok(transform, "expected a transform hook");

  const warnings: string[] = [];
  const result = await transform!.call(
    {
      warn(message: string) {
        warnings.push(message);
      },
    } as never,
    BASIC_SFC as never,
    "/proj/App.vue" as never,
  );

  t.assert.ok(result && typeof result === "object");
  const code = (result as { code: unknown }).code;
  t.assert.strictEqual(typeof code, "string");
  // The rendered template text survives compilation.
  t.assert.match(code as string, /Hello from Rolldown/);
  // Compiled output is a real ESM module that pulls runtime helpers from "vue".
  t.assert.match(code as string, /from "vue"/);
  t.assert.match(code as string, /export default/);
  t.assert.deepStrictEqual(warnings, []);
});

void test("rolldown transform strips TypeScript-only syntax from <script setup lang=ts>", async (t) => {
  const plugin = vize({ isProduction: true, root: packageRoot });
  const transform = resolveHook(plugin.transform);
  t.assert.ok(transform, "expected a transform hook");

  const result = await transform!.call(
    { warn() {} } as never,
    TS_SFC as never,
    "/proj/App.vue" as never,
  );

  t.assert.ok(result && typeof result === "object");
  const code = (result as { code: string }).code;
  t.assert.strictEqual(typeof code, "string");
  t.assert.match(code, /Hello from Rolldown/);
  // strip-types ran: the `interface` declaration and the `: number` type annotation
  // are gone from the emitted JavaScript.
  t.assert.doesNotMatch(code, /\binterface\b/);
  t.assert.doesNotMatch(code, /const n\s*:\s*number/);
});

void test("rolldown transform returns null for non-vue ids", async (t) => {
  const plugin = vize({ isProduction: true, root: packageRoot });
  const transform = resolveHook(plugin.transform);
  t.assert.ok(transform, "expected a transform hook");

  const result = await transform!.call(
    { warn() {} } as never,
    "export const n = 1;" as never,
    "/proj/main.ts" as never,
  );

  // The raw hook in unplugin.ts returns `null` for non-vue ids, but unplugin's
  // Rollup-format wrapper normalizes that "no-op" result to `undefined`. Both are
  // a falsy skip in Rollup semantics; assert the value actually observed here.
  t.assert.strictEqual(result, undefined);
});
