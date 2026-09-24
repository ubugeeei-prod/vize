import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type { ImageRootExpose, ImageSlotState } from "./image.ts";
import ImageContent from "./image-content.vue";
import ImageFallback from "./image-fallback.vue";
import ImagePlaceholder from "./image-placeholder.vue";
import ImageRoot from "./image-root.vue";
import { resolveImageCandidates } from "./image-source.ts";
import { mountInteraction } from "../../../testing/mount.ts";

type IntersectionCallback = (
  entries: readonly { target: Element; isIntersecting: boolean; intersectionRatio: number }[],
) => void;

class FakeIntersectionObserver {
  static instances: FakeIntersectionObserver[] = [];
  readonly observed: Element[] = [];
  disconnected = false;

  constructor(
    readonly callback: IntersectionCallback,
    readonly init: IntersectionObserverInit,
  ) {
    FakeIntersectionObserver.instances.push(this);
  }

  observe(target: Element): void {
    this.observed.push(target);
  }

  unobserve(target: Element): void {
    const index = this.observed.indexOf(target);
    if (index >= 0) this.observed.splice(index, 1);
  }

  disconnect(): void {
    this.disconnected = true;
    this.observed.length = 0;
  }

  intersect(isIntersecting: boolean): void {
    this.callback(
      this.observed.map((target) => ({
        target,
        isIntersecting,
        intersectionRatio: isIntersecting ? 1 : 0,
      })),
    );
  }
}

// happy-dom reports every image as already complete. Model a real pending
// network request so only the hydration test exercises the settled path.
Object.defineProperty(HTMLImageElement.prototype, "complete", {
  configurable: true,
  get: () => false,
});

function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function withFakeIntersectionObserver(run: () => Promise<void>): Promise<void> {
  const previous = globalThis.IntersectionObserver;
  FakeIntersectionObserver.instances = [];
  globalThis.IntersectionObserver =
    FakeIntersectionObserver as unknown as typeof IntersectionObserver;
  try {
    await run();
  } finally {
    globalThis.IntersectionObserver = previous;
  }
}

function mountImage(
  props: Record<string, unknown> = {},
  contentProps: Record<string, unknown> = {},
  placeholderProps: Record<string, unknown> = {},
) {
  return mountInteraction(ImageRoot, {
    props,
    record: ["load", "error", "statusChange"],
    slots: {
      default: (state: ImageSlotState) => [
        h("output", { "data-root-status": state.status }, String(state.candidateIndex)),
        h(ImageContent, { alt: "Mountain lake", ...contentProps }),
        h(ImagePlaceholder, placeholderProps, {
          default: ({ status }: ImageSlotState) => h("span", { "data-skeleton": status }),
        }),
        h(ImageFallback, null, {
          default: ({ candidateCount }: ImageSlotState) =>
            h("span", { "data-fallback-count": String(candidateCount) }, "ML"),
        }),
      ],
    },
  });
}

function image(root: HTMLElement): HTMLImageElement | null {
  return root.querySelector<HTMLImageElement>('[data-vize-ui="image-content"]');
}

test("renders a loading native image with placeholder, native attributes, and no fallback", () => {
  const handle = mountImage(
    { src: "/lake.avif" },
    { srcset: "/lake-2x.avif 2x", sizes: "50vw", width: 640, height: 480, fetchPriority: "high" },
  );
  const root = handle.root();
  const img = image(root);

  assert.equal(root.tagName, "SPAN");
  assert.equal(root.getAttribute("data-vize-ui"), "image-root");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-status"), "loading");
  assert.ok(img);
  assert.equal(img.getAttribute("src"), "/lake.avif");
  assert.equal(img.getAttribute("alt"), "Mountain lake");
  assert.equal(img.getAttribute("srcset"), "/lake-2x.avif 2x");
  assert.equal(img.getAttribute("sizes"), "50vw");
  assert.equal(img.getAttribute("loading"), "lazy");
  assert.equal(img.getAttribute("decoding"), "async");
  assert.equal(img.getAttribute("fetchpriority"), "high");
  assert.equal(img.getAttribute("width"), "640");
  assert.equal(img.getAttribute("height"), "480");
  assert.equal(img.getAttribute("part"), "content");
  assert.equal(img.getAttribute("data-candidate"), "0");
  const placeholder = root.querySelector('[data-vize-ui="image-placeholder"]');
  assert.equal(placeholder?.getAttribute("aria-hidden"), "true");
  assert.equal(
    placeholder?.querySelector("[data-skeleton]")?.getAttribute("data-skeleton"),
    "loading",
  );
  assert.equal(root.querySelector('[data-vize-ui="image-fallback"]'), null);
  handle.unmount();
});

test("load settles the lifecycle, hides the placeholder, and emits load", async () => {
  const handle = mountImage({ src: "/lake.avif" });
  const root = handle.root();
  image(root)?.dispatchEvent(new Event("load"));
  await nextTick();

  assert.equal(root.getAttribute("data-status"), "loaded");
  assert.equal(image(root)?.getAttribute("data-status"), "loaded");
  assert.equal(root.querySelector('[data-vize-ui="image-placeholder"]'), null);
  assert.equal(root.querySelector('[data-vize-ui="image-fallback"]'), null);
  const load = handle.wrapper.emitted("load")?.[0];
  assert.ok(load?.[0] instanceof Event);
  assert.equal(load?.[1], "/lake.avif");
  assert.deepEqual(handle.wrapper.emitted("statusChange"), [["loaded", "loading", "load"]]);
  handle.unmount();
});

test("advances through the candidate chain and renders the fallback after the last failure", async () => {
  const handle = mountImage(
    { src: ["/lake.avif", "javascript:alert(1)", "/lake.jpg", "/lake.avif"] },
    { srcset: "/lake-2x.avif 2x" },
  );
  const root = handle.root();

  assert.equal(root.querySelector("output")?.textContent, "0");
  image(root)?.dispatchEvent(new Event("error"));
  await nextTick();
  assert.equal(image(root)?.getAttribute("src"), "/lake.jpg");
  assert.equal(image(root)?.getAttribute("srcset"), null);
  assert.equal(image(root)?.getAttribute("data-candidate"), "1");
  assert.equal(root.getAttribute("data-status"), "loading");

  image(root)?.dispatchEvent(new Event("error"));
  await nextTick();
  assert.equal(root.getAttribute("data-status"), "error");
  assert.equal(image(root), null);
  const fallback = root.querySelector('[data-vize-ui="image-fallback"]');
  assert.equal(fallback?.textContent, "ML");
  assert.equal(
    fallback?.querySelector("[data-fallback-count]")?.getAttribute("data-fallback-count"),
    "2",
  );
  assert.deepEqual(
    handle.wrapper.emitted("error")?.map((payload) => payload[1]),
    ["/lake.avif", "/lake.jpg"],
  );
  assert.deepEqual(handle.wrapper.emitted("statusChange"), [["error", "loading", "error"]]);
  handle.unmount();
});

test("missing and unsafe sources render the fallback without forwarding a source", () => {
  const missing = mountImage();
  assert.equal(missing.root().getAttribute("data-status"), "error");
  assert.equal(image(missing.root()), null);
  assert.ok(missing.root().querySelector('[data-vize-ui="image-fallback"]'));
  missing.unmount();

  const unsafe = mountImage({ src: ["javascript:alert(1)", "http://insecure.test/a.png", " "] });
  assert.equal(unsafe.root().getAttribute("data-status"), "error");
  assert.equal(unsafe.root().querySelector("img"), null);
  unsafe.unmount();

  const insecure = mountImage({ src: "http://localhost/a.png", allowInsecure: true });
  assert.equal(image(insecure.root())?.getAttribute("src"), "http://localhost/a.png");
  insecure.unmount();
});

test("source replacement and retry restart the chain", async () => {
  const handle = mountImage({ src: "/a.png" });
  const root = handle.root();
  image(root)?.dispatchEvent(new Event("error"));
  await nextTick();
  assert.equal(root.getAttribute("data-status"), "error");

  const exposed = handle.exposes<ImageRootExpose>();
  assert.equal(exposed.retry(), true);
  await nextTick();
  assert.equal(root.getAttribute("data-status"), "loading");
  assert.equal(image(root)?.getAttribute("src"), "/a.png");

  image(root)?.dispatchEvent(new Event("load"));
  await nextTick();
  await handle.wrapper.setProps({ src: ["/a.png"] });
  assert.equal(root.getAttribute("data-status"), "loaded", "equal chains keep the settled state");
  await handle.wrapper.setProps({ src: "/b.png" });
  assert.equal(root.getAttribute("data-status"), "loading");
  assert.equal(image(root)?.getAttribute("src"), "/b.png");
  assert.deepEqual(
    handle.wrapper.emitted("statusChange")?.map((payload) => payload[2]),
    ["error", "retry", "load", "source"],
  );

  await handle.wrapper.setProps({ src: undefined });
  assert.equal(exposed.retry(), false);
  handle.unmount();
});

test("deferred images stay idle until visible and fall back to eager without observers", async () => {
  await withFakeIntersectionObserver(async () => {
    const handle = mountImage({ src: "/deferred.png", defer: true, rootMargin: "50px" });
    const root = handle.root();
    await nextTick();

    assert.equal(root.getAttribute("data-status"), "idle");
    assert.equal(root.getAttribute("data-deferred"), "true");
    assert.equal(image(root)?.hasAttribute("src"), false);
    assert.ok(root.querySelector('[data-vize-ui="image-placeholder"]'));
    const observer = FakeIntersectionObserver.instances[0];
    assert.ok(observer);
    assert.equal(observer.init.rootMargin, "50px");
    assert.ok(observer.observed[0] === root);

    observer.intersect(false);
    await nextTick();
    assert.equal(root.getAttribute("data-status"), "idle");
    observer.intersect(true);
    await nextTick();
    assert.equal(root.getAttribute("data-status"), "loading");
    assert.equal(image(root)?.getAttribute("src"), "/deferred.png");
    assert.equal(observer.observed.length, 0);
    assert.deepEqual(handle.wrapper.emitted("statusChange"), [["loading", "idle", "visible"]]);
    handle.unmount();
  });

  const previous = globalThis.IntersectionObserver;
  Reflect.deleteProperty(globalThis, "IntersectionObserver");
  try {
    const eager = mountImage({ src: "/deferred.png", defer: true });
    await nextTick();
    assert.equal(eager.root().getAttribute("data-status"), "loading");
    eager.unmount();
  } finally {
    globalThis.IntersectionObserver = previous;
  }
});

test("turning defer off attaches the source immediately", async () => {
  await withFakeIntersectionObserver(async () => {
    const handle = mountImage({ src: "/deferred.png", defer: true });
    await nextTick();
    assert.equal(handle.root().getAttribute("data-status"), "idle");
    await handle.wrapper.setProps({ defer: false });
    assert.equal(handle.root().getAttribute("data-status"), "loading");
    handle.unmount();
  });
});

test("delayed placeholders wait before rendering and never render after settling", async () => {
  const handle = mountImage({ src: "/slow.png" }, {}, { delay: 20 });
  const root = handle.root();
  assert.equal(root.querySelector('[data-vize-ui="image-placeholder"]'), null);
  await wait(30);
  await nextTick();
  assert.ok(root.querySelector('[data-vize-ui="image-placeholder"]'));
  image(root)?.dispatchEvent(new Event("load"));
  await nextTick();
  assert.equal(root.querySelector('[data-vize-ui="image-placeholder"]'), null);
  handle.unmount();

  const cached = mountImage({ src: "/cached.png" }, {}, { delay: 20 });
  image(cached.root())?.dispatchEvent(new Event("load"));
  await wait(30);
  await nextTick();
  assert.equal(cached.root().querySelector('[data-vize-ui="image-placeholder"]'), null);
  cached.unmount();
});

test("reads images that settled before hydration attached listeners", async () => {
  const completeDescriptor = Object.getOwnPropertyDescriptor(
    HTMLImageElement.prototype,
    "complete",
  );
  const widthDescriptor = Object.getOwnPropertyDescriptor(
    HTMLImageElement.prototype,
    "naturalWidth",
  );
  const decodeDescriptor = Object.getOwnPropertyDescriptor(HTMLImageElement.prototype, "decode");
  let naturalWidth = 320;
  let decodeResult: Promise<void> = Promise.resolve();
  Object.defineProperty(HTMLImageElement.prototype, "complete", {
    configurable: true,
    get: () => true,
  });
  Object.defineProperty(HTMLImageElement.prototype, "naturalWidth", {
    configurable: true,
    get: () => naturalWidth,
  });
  Object.defineProperty(HTMLImageElement.prototype, "decode", {
    configurable: true,
    value: () => decodeResult,
  });

  try {
    const loaded = mountImage({ src: "/cached.png" });
    await nextTick();
    assert.equal(loaded.root().getAttribute("data-status"), "loaded");
    assert.deepEqual(loaded.wrapper.emitted("load")?.[0], [null, "/cached.png"]);
    loaded.unmount();

    naturalWidth = 0;
    decodeResult = Promise.reject(new Error("EncodingError"));
    decodeResult.catch(() => undefined);
    const broken = mountImage({ src: ["/broken.png"] });
    await wait(0);
    await nextTick();
    assert.equal(broken.root().getAttribute("data-status"), "error");
    assert.deepEqual(broken.wrapper.emitted("error")?.[0], [null, "/broken.png"]);
    broken.unmount();

    decodeResult = Promise.resolve();
    const vector = mountImage({ src: "/vector.svg" });
    await wait(0);
    await nextTick();
    assert.equal(vector.root().getAttribute("data-status"), "loaded");
    vector.unmount();
  } finally {
    for (const [name, descriptor] of [
      ["complete", completeDescriptor],
      ["naturalWidth", widthDescriptor],
      ["decode", decodeDescriptor],
    ] as const) {
      if (descriptor) Object.defineProperty(HTMLImageElement.prototype, name, descriptor);
      else Reflect.deleteProperty(HTMLImageElement.prototype, name);
    }
  }
});

test("exposes typed lifecycle state and live parts", async () => {
  const handle = mountImage({ src: ["/a.png", "/b.png"] });
  const exposed = handle.exposes<ImageRootExpose>();
  assert.equal(exposed.status, "loading");
  assert.equal(exposed.src, "/a.png");
  assert.equal(exposed.candidateIndex, 0);
  assert.equal(exposed.candidateCount, 2);
  assert.ok(exposed.element === handle.root());
  image(handle.root())?.dispatchEvent(new Event("error"));
  await nextTick();
  assert.equal(exposed.src, "/b.png");
  assert.equal(exposed.candidateIndex, 1);
  handle.unmount();
});

test("compound parts require a matching root provider", () => {
  for (const part of [ImageContent, ImageFallback, ImagePlaceholder]) {
    assert.throws(
      () => mountInteraction(part, { props: part === ImageContent ? { alt: "" } : {} }),
      /VIZE_UI_CONTEXT_MISSING: Image requires a matching provider/,
    );
  }
});

test("resolves safe, de-duplicated candidate chains", () => {
  assert.deepEqual(resolveImageCandidates(undefined), []);
  assert.deepEqual(resolveImageCandidates(null), []);
  assert.deepEqual(resolveImageCandidates(" /a.png "), ["/a.png"]);
  assert.deepEqual(
    resolveImageCandidates([
      "/a.png",
      "https://cdn.test/b.webp",
      "blob:https://app.test/1",
      "data:image/png;base64,iVBORw0KGgo=",
      "data:text/html;base64,PGgxPg==",
      "javascript:alert(1)",
      "http://cdn.test/c.png",
      "",
      "/a.png",
    ]),
    [
      "/a.png",
      "https://cdn.test/b.webp",
      "blob:https://app.test/1",
      "data:image/png;base64,iVBORw0KGgo=",
    ],
  );
  assert.deepEqual(resolveImageCandidates("http://cdn.test/c.png", { allowInsecure: true }), [
    "http://cdn.test/c.png",
  ]);
  assert.ok(Object.isFrozen(resolveImageCandidates("/a.png")));
});
