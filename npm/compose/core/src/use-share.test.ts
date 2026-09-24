import assert from "node:assert/strict";
import { test } from "node:test";
import { ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useShare } from "./use-share.ts";
import type { ShareDataLike, ShareHost } from "./use-share.ts";

function named(name: string): Error {
  const error = new Error(name);
  error.name = name;
  return error;
}

class FakeShare implements ShareHost {
  readonly shared: ShareDataLike[] = [];
  failure: unknown;
  allowFiles = false;

  share(data: ShareDataLike): Promise<void> {
    if (this.failure !== undefined) return Promise.reject(this.failure);
    this.shared.push(data);
    return Promise.resolve();
  }

  canShare(data?: ShareDataLike): boolean {
    return this.allowFiles || data?.files === undefined;
  }
}

void test("merges reactive defaults with overrides", async () => {
  const host = new FakeShare();
  const title = ref("Vize");
  const share = useShare(() => ({ title: title.value, url: "https://vize.dev" }), { host });

  title.value = "Vize docs";
  const result = await share.share({ text: "hello" });
  assert.deepEqual(result, {
    status: "shared",
    data: { title: "Vize docs", url: "https://vize.dev", text: "hello" },
  });
  assert.equal(share.supported.value, true);
  assert.equal(share.sharing.value, false);
});

void test("distinguishes cancellation, activation, invalid data, and failures", async () => {
  const host = new FakeShare();
  const share = useShare({}, { host });

  host.failure = named("AbortError");
  assert.equal((await share.share()).status, "cancelled");
  host.failure = named("NotAllowedError");
  assert.equal((await share.share()).status, "not-allowed");
  host.failure = named("TypeError");
  assert.equal((await share.share()).status, "invalid");
  host.failure = named("OperationError");
  assert.equal((await share.share()).status, "failed");
});

void test("checks canShare before opening the sheet", async () => {
  const host = new FakeShare();
  const share = useShare({}, { host });
  const files = [new File(["x"], "x.txt")];

  assert.equal(share.canShare({ files }), false);
  assert.deepEqual(await share.share({ files }), { status: "invalid", error: undefined });
  assert.equal(host.shared.length, 0);
  host.allowFiles = true;
  assert.equal((await share.share({ files })).status, "shared");
});

void test("tracks the sharing flag while the sheet is open", async () => {
  let finish: (() => void) | undefined;
  const host: ShareHost = {
    share: () =>
      new Promise((resolve) => {
        finish = resolve;
      }),
  };
  const share = useShare({}, { host });
  const pending = share.share({ text: "x" });
  assert.equal(share.sharing.value, true);
  assert.equal(share.canShare(), true, "hosts without canShare accept any data");
  finish?.();
  await pending;
  assert.equal(share.sharing.value, false);
});

void test("reports unsupported without a host", async () => {
  const share = useShare({}, { host: null });
  assert.equal(share.supported.value, false);
  assert.equal(share.canShare(), false);
  assert.deepEqual(await share.share(), { status: "unsupported", error: undefined });
});

void test("server rendering reports no support", async () => {
  const state = await renderComposableOnServer(() => {
    const share = useShare({ title: "x" });
    return { supported: share.supported, sharing: share.sharing };
  });
  assert.equal(state, '{"supported":false,"sharing":false}');
});
