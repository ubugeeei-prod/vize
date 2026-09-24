import assert from "node:assert/strict";

import { afterEach, test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type {
  ScrubberPreviewRootExpose,
  ScrubberPreviewSlotState,
  ScrubberPreviewThumbnailSlotState,
  ScrubberPreviewTrackExpose,
} from "./scrubber-preview.ts";
import { captureVideoFrame, ScrubberPreviewError } from "./scrubber-preview-capture.ts";
import ScrubberPreviewRoot from "./scrubber-preview-root.vue";
import { installFakeCapture } from "./scrubber-preview-test-utils.ts";
import type { FakeCaptureEnvironment } from "./scrubber-preview-test-utils.ts";
import ScrubberPreviewThumbnail from "./scrubber-preview-thumbnail.vue";
import ScrubberPreviewTime from "./scrubber-preview-time.vue";
import ScrubberPreviewTrack from "./scrubber-preview-track.vue";
import { mountInteraction } from "../../../testing/mount.ts";

let capture: FakeCaptureEnvironment | null = null;

afterEach(() => {
  capture?.restore();
  capture = null;
});

function wait(ms = 0): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function settle(): Promise<void> {
  for (let round = 0; round < 6; round++) {
    await Promise.resolve();
    await nextTick();
  }
}

function stubRect(element: Element, left = 100, width = 200): void {
  element.getBoundingClientRect = () =>
    ({
      x: left,
      y: 0,
      left,
      top: 0,
      right: left + width,
      bottom: 10,
      width,
      height: 10,
      toJSON: () => ({}),
    }) satisfies DOMRect;
}

function pointer(type: string, init: PointerEventInit = {}): PointerEvent {
  return new PointerEvent(type, {
    bubbles: true,
    cancelable: true,
    pointerId: 1,
    pointerType: "mouse",
    button: 0,
    ...init,
  });
}

function mountPreview(
  props: Record<string, unknown> = {},
  thumbnailSlot?: (state: ScrubberPreviewThumbnailSlotState) => unknown,
) {
  const handle = mountInteraction(ScrubberPreviewRoot, {
    props: { duration: 100, ...props },
    record: ["update:time", "seek"],
    slots: {
      default: (state: ScrubberPreviewSlotState) => [
        h("output", { "data-root-time": String(state.time) }),
        h(ScrubberPreviewTrack, null, () => h("div", { "data-seek-bar": "" })),
        h(
          ScrubberPreviewThumbnail,
          null,
          thumbnailSlot === undefined ? undefined : { default: thumbnailSlot },
        ),
        h(ScrubberPreviewTime),
      ],
    },
  });
  const root = handle.root();
  const track = root.querySelector<HTMLDivElement>('[data-vize-ui="scrubber-preview-track"]');
  const thumbnail = root.querySelector<HTMLDivElement>(
    '[data-vize-ui="scrubber-preview-thumbnail"]',
  );
  const time = root.querySelector<HTMLSpanElement>('[data-vize-ui="scrubber-preview-time"]');
  assert.ok(track && thumbnail && time);
  stubRect(track);
  return { handle, root, track, thumbnail, time };
}

test("renders an idle preview with hidden, decorative thumbnail and time parts", () => {
  const { handle, root, thumbnail, time } = mountPreview();
  assert.equal(root.getAttribute("data-vize-ui"), "scrubber-preview-root");
  assert.equal(root.getAttribute("data-state"), "idle");
  assert.equal(root.getAttribute("data-kind"), "none");
  assert.equal(root.style.getPropertyValue("--vize-ui-scrubber-preview-ratio"), "0");
  assert.equal(thumbnail.hidden, true);
  assert.equal(thumbnail.getAttribute("aria-hidden"), "true");
  assert.equal(thumbnail.getAttribute("data-status"), "idle");
  assert.equal(time.getAttribute("aria-hidden"), "true");
  assert.equal(time.textContent, "");
  handle.unmount();
});

test("hovering the track previews the time under the pointer and leaving hides it", async () => {
  const { handle, root, track, thumbnail, time } = mountPreview();
  track.dispatchEvent(pointer("pointerenter", { clientX: 150 }));
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "active");
  assert.equal(root.style.getPropertyValue("--vize-ui-scrubber-preview-ratio"), "0.25");
  assert.equal(root.querySelector("output")?.getAttribute("data-root-time"), "25");
  assert.equal(time.textContent, "0:25");
  assert.equal(thumbnail.hidden, false);
  track.dispatchEvent(pointer("pointermove", { clientX: 290 }));
  await nextTick();
  assert.equal(time.textContent, "1:35");
  track.dispatchEvent(pointer("pointerleave"));
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "idle");
  assert.deepEqual(handle.wrapper.emitted("update:time"), [[25], [95], [null]]);
  handle.unmount();
});

test("clicking or scrubbing the track emits seek on release; touch hides on lift", async () => {
  const { handle, root, track } = mountPreview({ dir: "rtl" });
  track.dispatchEvent(pointer("pointerdown", { clientX: 250 }));
  await nextTick();
  assert.equal(track.getAttribute("data-scrubbing"), "true");
  assert.equal(handle.wrapper.findComponent(ScrubberPreviewTrack).vm.$el, track);
  track.dispatchEvent(pointer("pointerleave"));
  await nextTick();
  assert.equal(
    root.getAttribute("data-state"),
    "active",
    "leaving while scrubbing keeps the preview",
  );
  track.dispatchEvent(pointer("pointermove", { clientX: 280 }));
  track.dispatchEvent(pointer("pointerup", { clientX: 280 }));
  await nextTick();
  const seek = handle.wrapper.emitted("seek")?.[0];
  assert.ok(seek);
  assert.equal(Math.round(Number(seek[0])), 10, "RTL maps the right edge to zero");
  assert.ok(seek[1] instanceof PointerEvent);
  assert.equal(track.getAttribute("data-scrubbing"), null);

  track.dispatchEvent(pointer("pointerdown", { pointerId: 2, pointerType: "touch", clientX: 150 }));
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "active");
  track.dispatchEvent(
    pointer("pointercancel", { pointerId: 2, pointerType: "touch", clientX: 150 }),
  );
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "idle");
  assert.equal(handle.wrapper.emitted("seek")?.length, 1, "cancel never seeks");

  track.dispatchEvent(pointer("pointerdown", { button: 2, clientX: 150 }));
  assert.equal(track.getAttribute("data-scrubbing"), null, "secondary buttons are ignored");
  const exposed = handle.wrapper.findComponent(ScrubberPreviewTrack)
    .vm as unknown as ScrubberPreviewTrackExpose;
  assert.equal(exposed.scrubbing, false);
  handle.unmount();
});

test("controlled time, setTime, and disabled roots", async () => {
  const { handle, root, track, time } = mountPreview({ time: 30 });
  assert.equal(root.getAttribute("data-state"), "active");
  assert.equal(time.textContent, "0:30");
  track.dispatchEvent(pointer("pointermove", { clientX: 200 }));
  await nextTick();
  assert.deepEqual(handle.wrapper.emitted("update:time"), [[50]]);
  assert.equal(time.textContent, "0:30", "the parent owns the preview time");
  await handle.wrapper.setProps({ time: 500 });
  assert.equal(time.textContent, "1:40", "times clamp to the duration");

  const uncontrolled = mountPreview({ duration: 7200 });
  const exposed = uncontrolled.handle.exposes<ScrubberPreviewRootExpose>();
  assert.equal(exposed.setTime(3723), true);
  assert.equal(exposed.setTime(3723), false);
  await nextTick();
  assert.equal(uncontrolled.time.textContent, "1:02:03");
  assert.equal(exposed.ratio, 3723 / 7200);
  assert.equal(exposed.active, true);
  assert.equal(exposed.setTime(null), true);
  assert.equal(exposed.time, null);
  uncontrolled.handle.unmount();

  await handle.wrapper.setProps({ disabled: true, time: undefined });
  assert.equal(root.getAttribute("data-disabled"), "true");
  track.dispatchEvent(pointer("pointerdown", { clientX: 150 }));
  track.dispatchEvent(pointer("pointerup", { clientX: 150 }));
  assert.equal(handle.wrapper.emitted("seek"), undefined);
  assert.equal(handle.exposes<ScrubberPreviewRootExpose>().setTime(5), false);
  handle.unmount();
});

test("sprite thumbnails publish background geometry for the previewed frame", async () => {
  const sprite = {
    src: "/thumbs/sheet.jpg",
    columns: 5,
    rows: 5,
    interval: 2,
    width: 160,
    height: 90,
  };
  const { handle, root, track, thumbnail } = mountPreview({ sprite });
  assert.equal(root.getAttribute("data-kind"), "sprite");
  track.dispatchEvent(pointer("pointermove", { clientX: 114 }));
  await nextTick();
  assert.equal(thumbnail.getAttribute("data-status"), "ready");
  assert.equal(
    thumbnail.style.getPropertyValue("--vize-ui-scrubber-preview-image"),
    'url("/thumbs/sheet.jpg")',
  );
  assert.equal(thumbnail.style.getPropertyValue("--vize-ui-scrubber-preview-x"), "-480px");
  assert.equal(thumbnail.style.getPropertyValue("--vize-ui-scrubber-preview-y"), "0px");
  assert.equal(thumbnail.style.getPropertyValue("--vize-ui-scrubber-preview-width"), "160px");
  assert.equal(thumbnail.querySelector("img"), null, "sprites are CSS backgrounds");
  handle.unmount();

  const unsafe = mountPreview({ sprite: { ...sprite, src: "javascript:alert(1)" }, time: 3 });
  assert.equal(unsafe.thumbnail.getAttribute("data-status"), "error");
  assert.equal(unsafe.thumbnail.style.getPropertyValue("--vize-ui-scrubber-preview-image"), "");
  unsafe.handle.unmount();
});

test("WebVTT thumbnails resolve cues and expose frames to slots", async () => {
  const thumbnails = "WEBVTT\n\n00:00.000 --> 00:10.000\nthumbs.jpg#xywh=0,90,160,90\n";
  const { handle, thumbnail } = mountPreview(
    { thumbnails, thumbnailsBaseUrl: "https://cdn.test/v/", time: 4 },
    (state) =>
      h("i", {
        "data-kind": state.kind,
        "data-src": state.frame?.src,
        "data-y": String(state.frame?.region?.y),
      }),
  );
  const slot = thumbnail.querySelector("i");
  assert.equal(slot?.getAttribute("data-kind"), "vtt");
  assert.equal(slot?.getAttribute("data-src"), "https://cdn.test/v/thumbs.jpg");
  assert.equal(slot?.getAttribute("data-y"), "90");
  assert.equal(thumbnail.style.getPropertyValue("--vize-ui-scrubber-preview-y"), "-90px");
  await handle.wrapper.setProps({ time: 50 });
  assert.equal(thumbnail.getAttribute("data-status"), "idle", "no cue covers 50s");
  handle.unmount();

  const parsed = mountPreview({
    thumbnails: [{ start: 0, end: 5, src: "/still.jpg", region: null }],
    time: 1,
  });
  assert.equal(parsed.thumbnail.getAttribute("data-status"), "ready");
  assert.equal(parsed.thumbnail.style.getPropertyValue("--vize-ui-scrubber-preview-x"), "");
  parsed.handle.unmount();
});

test("captured thumbnails are generated once per interval, cached, and revoked", async () => {
  capture = installFakeCapture();
  const { handle, track, thumbnail } = mountPreview({
    videoSrc: "/movie.mp4",
    captureInterval: 5,
    cacheSize: 2,
    captureWidth: 320,
  });
  assert.equal(thumbnail.getAttribute("data-kind"), "capture");
  track.dispatchEvent(pointer("pointermove", { clientX: 114 }));
  await nextTick();
  assert.equal(thumbnail.getAttribute("data-status"), "loading");
  await settle();
  assert.equal(thumbnail.getAttribute("data-status"), "ready");
  assert.deepEqual(capture.drawn, [5], "7s snaps to the 5s capture slot");
  const image = thumbnail.querySelector("img");
  assert.equal(image?.getAttribute("src"), "blob:https://app.test/frame-1");
  assert.equal(image?.getAttribute("alt"), "");

  track.dispatchEvent(pointer("pointermove", { clientX: 116 }));
  await settle();
  assert.deepEqual(capture.drawn, [5], "the same slot is served from the cache");

  for (const clientX of [140, 160]) {
    track.dispatchEvent(pointer("pointermove", { clientX }));
    await settle();
  }
  assert.deepEqual(capture.drawn, [5, 20, 30]);
  assert.deepEqual(capture.revoked, ["blob:https://app.test/frame-1"], "LRU eviction revokes");
  handle.unmount();
  assert.deepEqual(capture.revoked.length, 3, "unmount revokes every cached frame");
});

test("rapid capture requests keep only the latest pending frame", async () => {
  capture = installFakeCapture(false);
  const { handle, track, thumbnail } = mountPreview({ videoSrc: "/movie.mp4" });
  for (const clientX of [110, 120, 130, 140]) {
    track.dispatchEvent(pointer("pointermove", { clientX }));
    await nextTick();
  }
  capture.settle();
  await settle();
  capture.settle();
  await settle();
  assert.deepEqual(capture.drawn, [5, 20], "the 10s and 15s requests are skipped");
  assert.equal(
    thumbnail.querySelector("img")?.getAttribute("src"),
    "blob:https://app.test/frame-2",
  );
  handle.unmount();
});

test("capture failures surface typed errors", async () => {
  capture = installFakeCapture();
  capture.fail.seek = true;
  const errors: ScrubberPreviewError[] = [];
  const handle = mountInteraction(ScrubberPreviewRoot, {
    props: { duration: 100, videoSrc: "/movie.mp4", time: 12 },
    slots: {
      default: () =>
        h(ScrubberPreviewThumbnail, {
          onError: (error: ScrubberPreviewError) => errors.push(error),
        }),
    },
  });
  await handle.wrapper.setProps({ time: 40 });
  await settle();
  const thumbnail = handle.root().querySelector('[data-vize-ui="scrubber-preview-thumbnail"]');
  assert.equal(thumbnail?.getAttribute("data-status"), "error");
  assert.equal(errors[0]?.code, "VIZE_UI_SCRUBBER_PREVIEW_LOAD_FAILED");
  handle.unmount();

  const unsafe = mountPreview({ videoSrc: "javascript:alert(1)" });
  assert.equal(unsafe.root.getAttribute("data-kind"), "none");
  unsafe.handle.unmount();
});

test("captureVideoFrame waits for metadata, seeks, scales, and encodes", async () => {
  capture = installFakeCapture();
  capture.readyState = 0;
  const video = document.createElement("video");
  const pending = captureVideoFrame(video, 500, { width: 320, type: "image/webp" });
  await wait();
  capture.readyState = 4;
  video.dispatchEvent(new Event("loadedmetadata"));
  const blob = await pending;
  assert.equal(blob.type, "image/webp");
  assert.equal(await blob.text(), "320x180", "height keeps the 16:9 aspect ratio");
  assert.deepEqual(capture.drawn, [120], "times clamp to the duration");

  const dataUrl = await captureVideoFrame(video, 120, {
    output: "data-url",
    height: 90,
    width: 90,
  });
  assert.equal(dataUrl, `data:image/jpeg;base64,${btoa("90x90")}`);
  assert.deepEqual(capture.drawn, [120, 120], "no seek is needed for the current time");
});

test("captureVideoFrame reports unsupported, tainted, and failed encodes", async () => {
  capture = installFakeCapture();
  const video = document.createElement("video");
  const codeOf = async (run: () => Promise<unknown>) => {
    try {
      await run();
    } catch (error) {
      assert.ok(error instanceof ScrubberPreviewError);
      assert.match(error.message, new RegExp(`^${error.code}: `));
      return error.code;
    }
    return "resolved";
  };
  capture.fail.context = true;
  assert.equal(
    await codeOf(() => captureVideoFrame(video, 1)),
    "VIZE_UI_SCRUBBER_PREVIEW_UNSUPPORTED",
  );
  capture.fail.context = false;
  capture.fail.draw = true;
  assert.equal(
    await codeOf(() => captureVideoFrame(video, 2)),
    "VIZE_UI_SCRUBBER_PREVIEW_CAPTURE_FAILED",
  );
  capture.fail.draw = false;
  capture.fail.encode = "throw";
  assert.equal(await codeOf(() => captureVideoFrame(video, 3)), "VIZE_UI_SCRUBBER_PREVIEW_TAINTED");
  capture.fail.encode = "null";
  assert.equal(
    await codeOf(() => captureVideoFrame(video, 4)),
    "VIZE_UI_SCRUBBER_PREVIEW_CAPTURE_FAILED",
  );
});

test("compound parts require a matching root provider", () => {
  for (const part of [ScrubberPreviewTrack, ScrubberPreviewThumbnail, ScrubberPreviewTime]) {
    assert.throws(
      () => mountInteraction(part),
      /VIZE_UI_CONTEXT_MISSING: ScrubberPreview requires a matching provider/,
    );
  }
});
