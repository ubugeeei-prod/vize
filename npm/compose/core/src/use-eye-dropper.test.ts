import assert from "node:assert/strict";
import { test } from "node:test";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useEyeDropper } from "./use-eye-dropper.ts";
import type { EyeDropperColor, EyeDropperHost } from "./use-eye-dropper.ts";

function createHost(outcome: () => Promise<EyeDropperColor>): {
  host: EyeDropperHost;
  signals: (AbortSignal | undefined)[];
} {
  const signals: (AbortSignal | undefined)[] = [];
  class FakeEyeDropper {
    open(options?: { signal?: AbortSignal }): Promise<EyeDropperColor> {
      signals.push(options?.signal);
      return outcome();
    }
  }
  return { host: FakeEyeDropper, signals };
}

void test("picks a color with a class-based host", async () => {
  const { host, signals } = createHost(() => Promise.resolve({ sRGBHex: "#ff0000" }));
  const dropper = useEyeDropper({ host, initialValue: "#000000" });

  assert.equal(dropper.supported.value, true);
  assert.equal(dropper.sRGBHex.value, "#000000");
  const controller = new AbortController();
  const pending = dropper.open(controller.signal);
  assert.equal(dropper.picking.value, true);
  assert.deepEqual(await pending, { status: "picked", sRGBHex: "#ff0000" });
  assert.equal(dropper.sRGBHex.value, "#ff0000");
  assert.equal(dropper.picking.value, false);
  assert.equal(signals[0], controller.signal);
});

void test("distinguishes cancellation from failures", async () => {
  const abort = new DOMException("dismissed", "AbortError");
  const cancelled = useEyeDropper({ host: createHost(() => Promise.reject(abort)).host });
  assert.deepEqual(await cancelled.open(), { status: "cancelled", error: abort });

  const failure = new Error("no activation");
  const failed = useEyeDropper({ host: createHost(() => Promise.reject(failure)).host });
  assert.deepEqual(await failed.open(), { status: "failed", error: failure });
  assert.equal(failed.sRGBHex.value, "");
});

void test("reports unsupported without a host", async () => {
  const dropper = useEyeDropper({ host: null });
  assert.equal(dropper.supported.value, false);
  assert.deepEqual(await dropper.open(), { status: "unsupported", error: undefined });
});

void test("server rendering reports no support", async () => {
  const state = await renderComposableOnServer(() => {
    const dropper = useEyeDropper({ initialValue: "#123456" });
    return { supported: dropper.supported, color: dropper.sRGBHex };
  });
  assert.equal(state, '{"supported":false,"color":"#123456"}');
});
