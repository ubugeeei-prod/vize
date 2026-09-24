import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick } from "vue";

import { useFps } from "./fps.ts";
import type { AnimationFrameHost } from "./fps.ts";

class FrameHost implements AnimationFrameHost {
  private queue = new Map<number, FrameRequestCallback>();
  private next = 1;

  requestAnimationFrame(callback: FrameRequestCallback): number {
    const id = this.next++;
    this.queue.set(id, callback);
    return id;
  }

  cancelAnimationFrame(handle: number): void {
    this.queue.delete(handle);
  }

  get pending(): number {
    return this.queue.size;
  }

  frame(time: number): void {
    const callbacks = [...this.queue.values()];
    this.queue.clear();
    for (const callback of callbacks) callback(time);
  }
}

void test("averages frame timestamps and cancels with the scope", () => {
  const host = new FrameHost();
  const scope = effectScope();
  const fps = scope.run(() => useFps({ host, every: 2 }));
  assert.ok(fps);
  assert.equal(fps.fps.value, 0);

  host.frame(0);
  host.frame(20);
  host.frame(40);
  assert.equal(fps.fps.value, 50);
  host.frame(50);
  host.frame(60);
  assert.equal(fps.fps.value, 100);

  scope.stop();
  assert.equal(host.pending, 0);
});

void test("pause and resume the frame loop; no frames on the server", async () => {
  const host = new FrameHost();
  const scope = effectScope();
  const fps = scope.run(() => useFps({ host, immediate: false }));
  assert.equal(host.pending, 0);
  fps?.resume();
  await nextTick();
  assert.equal(host.pending, 1);
  fps?.pause();
  await nextTick();
  assert.equal(host.pending, 0);
  scope.stop();

  const server = useFps({ host: () => undefined });
  assert.deepEqual([server.fps.value, server.isSupported.value], [0, false]);
});
