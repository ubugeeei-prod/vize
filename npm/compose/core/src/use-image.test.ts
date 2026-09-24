import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useImage } from "./use-image.ts";
import type { ImageLike } from "./use-image.ts";

class FakeImage extends EventTarget implements ImageLike {
  static created: FakeImage[] = [];
  src = "";
  srcset = "";
  sizes = "";
  alt = "";
  crossOrigin: string | null = null;
  referrerPolicy = "";
  loading = "eager";
  decoding = "auto";
  fetchPriority = "auto";
  width = 0;
  height = 0;

  constructor() {
    super();
    FakeImage.created.push(this);
  }

  fire(type: "load" | "error"): void {
    this.dispatchEvent(new Event(type));
  }
}

function latest(): FakeImage {
  const image = FakeImage.created.at(-1);
  assert.ok(image);
  return image;
}

void test("loads an image with its attributes", async () => {
  FakeImage.created = [];
  const image = useImage(
    {
      src: "/a.png",
      srcset: "/a@2x.png 2x",
      alt: "A",
      crossOrigin: "anonymous",
      fetchPriority: "high",
      width: 10,
    },
    { host: FakeImage },
  );

  assert.equal(image.status.value, "loading");
  const element = latest();
  assert.equal(element.src, "/a.png");
  assert.equal(element.srcset, "/a@2x.png 2x");
  assert.equal(element.crossOrigin, "anonymous");
  assert.equal(element.fetchPriority, "high");
  assert.equal(element.width, 10);

  const result = image.load();
  latest().fire("load");
  assert.equal((await result).status, "loaded");
  assert.equal(image.ready.value, true);
  assert.equal(image.image.value, latest());
});

void test("reports load errors", async () => {
  FakeImage.created = [];
  const image = useImage({ src: "/missing.png" }, { host: FakeImage, immediate: false });
  assert.equal(image.status.value, "idle");
  const result = image.load();
  latest().fire("error");
  const outcome = await result;
  assert.equal(outcome.status, "error");
  assert.equal(image.status.value, "error");
  assert.ok(image.error.value instanceof Event);
});

void test("latest source wins and stale images are cancelled", async () => {
  FakeImage.created = [];
  const src = ref("/one.png");
  const image = useImage(() => ({ src: src.value }), { host: FakeImage });
  const stale = latest();
  src.value = "/two.png";
  await nextTick();

  assert.equal(stale.src, "", "superseded downloads are aborted");
  stale.fire("load");
  assert.equal(image.status.value, "loading");
  latest().fire("load");
  assert.equal(image.status.value, "loaded");
  assert.equal(image.image.value?.src, "/two.png");
});

void test("cancels the pending load with the scope", async () => {
  FakeImage.created = [];
  const scope = effectScope();
  const image = scope.run(() => useImage({ src: "/a.png" }, { host: FakeImage, immediate: false }));
  assert.ok(image);
  const result = image.load();
  scope.stop();
  assert.deepEqual(await result, { status: "cancelled" });
});

void test("reports unsupported without a constructor", async () => {
  const image = useImage({ src: "/a.png" }, { host: null, immediate: false });
  assert.deepEqual(await image.load(), { status: "unsupported" });
});

void test("server rendering creates no image", async () => {
  const state = await renderComposableOnServer(() => {
    const image = useImage({ src: "/a.png" });
    return { status: image.status, ready: image.ready };
  });
  assert.equal(state, '{"status":"idle","ready":false}');
});
