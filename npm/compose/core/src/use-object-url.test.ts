import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, shallowRef } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useObjectUrl } from "./use-object-url.ts";
import type { ObjectUrlHost, ObjectUrlSource } from "./use-object-url.ts";

class FakeUrls implements ObjectUrlHost {
  readonly live = new Set<string>();
  next = 0;

  createObjectURL(_object: ObjectUrlSource): string {
    this.next += 1;
    const url = `blob:fake/${String(this.next)}`;
    this.live.add(url);
    return url;
  }

  revokeObjectURL(url: string): void {
    this.live.delete(url);
  }
}

void test("creates a URL and revokes the previous one on change", async () => {
  const host = new FakeUrls();
  const blob = shallowRef<Blob | null>(new Blob(["a"]));
  const scope = effectScope();
  const url = scope.run(() => useObjectUrl(blob, { host }));
  assert.ok(url);

  assert.equal(url.value, "blob:fake/1");
  blob.value = new Blob(["b"]);
  await nextTick();
  assert.equal(url.value, "blob:fake/2");
  assert.deepEqual([...host.live], ["blob:fake/2"]);

  blob.value = null;
  await nextTick();
  assert.equal(url.value, undefined);
  assert.equal(host.live.size, 0);

  blob.value = new Blob(["c"]);
  await nextTick();
  scope.stop();
  assert.equal(url.value, undefined);
  assert.equal(host.live.size, 0);
});

void test("accepts the URL constructor itself as the host", () => {
  class UrlLike {
    static created = 0;
    static createObjectURL(): string {
      UrlLike.created += 1;
      return "blob:ctor";
    }
    static revokeObjectURL(): void {}
  }
  const url = useObjectUrl(new Blob(["a"]), { host: UrlLike });
  assert.equal(url.value, "blob:ctor");
});

void test("stays undefined without a capability", () => {
  const url = useObjectUrl(new Blob(["a"]), { host: null });
  assert.equal(url.value, undefined);
});

void test("server rendering creates no URL", async () => {
  const state = await renderComposableOnServer(() => ({
    url: useObjectUrl(new Blob(["a"])).value ?? null,
  }));
  assert.equal(state, '{"url":null}');
});
