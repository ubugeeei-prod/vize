import assert from "node:assert/strict";
import { before, test } from "node:test";
import { runInNewContext, type Context } from "node:vm";
import { buildNativeApplication } from "../scripts/build.ts";
import type { NativeHostDiagnostic, NativeViewTree } from "../src/host-contract.ts";

let code = "";
before(async () => {
  const result = await buildNativeApplication(
    process.env.VIZE_NATIVE_HOST_COMPILER_PATH
      ? { compilerPath: process.env.VIZE_NATIVE_HOST_COMPILER_PATH }
      : {},
  );
  code = result.code;
  assert.equal(result.receipt.sources.length, 4);
  assert.ok(result.receipt.gzipBytes <= result.receipt.maximumGzipBytes);
});

async function settle(): Promise<void> {
  await new Promise<void>((resolve) => setImmediate(resolve));
}

function launch(scenario: string) {
  const messages: (NativeViewTree | NativeHostDiagnostic)[] = [];
  const context: Context = {
    console,
    queueMicrotask,
    setTimeout,
    clearTimeout,
    __VIZE_NATIVE_BOOT__: { scenario },
    webkit: {
      messageHandlers: {
        vizeNative: {
          postMessage: (value: NativeViewTree | NativeHostDiagnostic) =>
            messages.push(
              JSON.parse(JSON.stringify(value)) as NativeViewTree | NativeHostDiagnostic,
            ),
        },
      },
    },
  };
  runInNewContext(code, context, { timeout: 5_000 });
  return { messages, context };
}

function lastTree(messages: readonly (NativeViewTree | NativeHostDiagnostic)[]): NativeViewTree {
  const value = messages.at(-1);
  assert.ok(value?.format === "vize.native-view-tree");
  return value;
}

void test("a real Vize-compiled SFC reaches the native tree and handles native activation", async () => {
  const { messages, context } = launch("counter");
  await settle();
  const initial = lastTree(messages);
  assert.equal(initial.nodes.length, 4);
  assert.ok(initial.nodes.some((node) => node.kind === "text" && node.text === "Count: 0"));
  const button = initial.nodes.find((node) => node.kind === "button" && node.label === "Increment");
  assert.ok(button);
  const runtime = context.__VIZE_NATIVE__ as { dispatch(event: unknown): boolean };
  assert.equal(
    runtime.dispatch({ node: button.id, event: "press", sequence: initial.sequence }),
    true,
  );
  await settle();
  const updated = lastTree(messages);
  assert.equal(updated.sequence, initial.sequence + 1);
  assert.ok(updated.nodes.some((node) => node.kind === "text" && node.text === "Count: 1"));
  assert.deepEqual(
    updated.nodes.map(({ id }) => id),
    initial.nodes.map(({ id }) => id),
  );
});

void test("stale and malformed native events cannot mutate the compiled application", async () => {
  const { messages, context } = launch("counter");
  await settle();
  const initial = lastTree(messages);
  const button = initial.nodes.find(({ kind }) => kind === "button")!;
  const runtime = context.__VIZE_NATIVE__ as { dispatch(event: unknown): boolean };
  for (const event of [
    null,
    { node: button.id, event: "press", sequence: initial.sequence - 1 },
    { node: button.id, event: "click", sequence: initial.sequence },
    { node: 10000, event: "press", sequence: initial.sequence },
    { node: button.id, event: "press", sequence: initial.sequence, extra: true },
  ])
    assert.equal(runtime.dispatch(event), false);
  await settle();
  assert.equal(messages.length, 1);
  assert.ok(
    lastTree(messages).nodes.some((node) => node.kind === "text" && node.text === "Count: 0"),
  );
});

void test("unsupported native capabilities refuse before mounting any view", async () => {
  const { messages, context } = launch("unsupported-capability");
  await settle();
  assert.equal(context.__VIZE_NATIVE__, undefined);
  assert.equal(messages.length, 1);
  assert.equal(messages[0]?.format, "vize.native-diagnostic");
  const refusal = messages[0] as NativeHostDiagnostic;
  assert.equal(refusal.code, "missing-capability");
  assert.match(refusal.message, /native\.images/);
});

void test("unknown native development scenarios produce explicit refusals", async () => {
  const { messages, context } = launch("unknown");
  await settle();
  assert.equal(context.__VIZE_NATIVE__, undefined);
  assert.equal((messages[0] as NativeHostDiagnostic).code, "unsupported-native-scenario");
});
