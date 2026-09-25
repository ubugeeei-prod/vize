import assert from "node:assert/strict";

import { afterAll, afterEach, beforeAll, test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type { WebcamCaptureRootExpose, WebcamCaptureSlotState } from "./webcam-capture.ts";
import WebcamCaptureDeviceSelect from "./webcam-capture-device-select.vue";
import WebcamCapturePhoto from "./webcam-capture-photo.vue";
import WebcamCaptureRoot from "./webcam-capture-root.vue";
import WebcamCaptureShutter from "./webcam-capture-shutter.vue";
import WebcamCaptureStartButton from "./webcam-capture-start-button.vue";
import WebcamCaptureStatusMessage from "./webcam-capture-status-message.vue";
import WebcamCaptureStopButton from "./webcam-capture-stop-button.vue";
import WebcamCaptureSwitchCamera from "./webcam-capture-switch-camera.vue";
import WebcamCaptureVideo from "./webcam-capture-video.vue";
import {
  FakeHost,
  FakeStream,
  FakeTrack,
  flush,
  installFakeCanvas,
  installFakeSrcObject,
  installFakeObjectUrls,
  mediaError,
  setVideoSize,
} from "./webcam-capture-test-utils.ts";
import { mountInteraction } from "../../../testing/mount.ts";

const cleanups: (() => void)[] = [];
let restoreSrcObject: () => void = () => undefined;

beforeAll(() => {
  restoreSrcObject = installFakeSrcObject();
});

afterAll(() => restoreSrcObject());

afterEach(() => {
  while (cleanups.length > 0) cleanups.pop()?.();
});

function mountCamera(
  props: Record<string, unknown> = {},
  shutterProps: Record<string, unknown> = {},
) {
  const handle = mountInteraction(WebcamCaptureRoot, {
    props: { id: "camera", ...props },
    record: ["statusChange", "error", "streamChange", "capture", "captureError"],
    slots: {
      default: (state: WebcamCaptureSlotState) => [
        h("output", { "data-slot-status": state.status }, String(state.devices.length)),
        h(WebcamCaptureVideo),
        h(WebcamCaptureStartButton),
        h(WebcamCaptureStopButton),
        h(WebcamCaptureSwitchCamera),
        h(WebcamCaptureDeviceSelect),
        h(WebcamCaptureShutter, shutterProps),
        h(WebcamCapturePhoto, null, { empty: () => h("span", { "data-empty": "true" }) }),
        h(WebcamCaptureStatusMessage),
      ],
    },
  });
  cleanups.push(() => handle.unmount());
  return handle;
}

function part<Element extends HTMLElement>(root: HTMLElement, name: string): Element {
  const element = root.querySelector<Element>(`[data-vize-ui="webcam-capture-${name}"]`);
  assert.ok(element, `${name} must render`);
  return element;
}

function button(root: HTMLElement, name: string): HTMLButtonElement {
  return part<HTMLButtonElement>(root, name);
}

test("renders an idle camera without requesting permission", async () => {
  const host = new FakeHost();
  const handle = mountCamera({ host });
  const root = handle.root();
  await flush();

  assert.equal(root.id, "camera");
  assert.equal(root.getAttribute("data-vize-ui"), "webcam-capture-root");
  assert.equal(root.getAttribute("data-status"), "idle");
  assert.equal(root.getAttribute("data-facing-mode"), "user");
  assert.equal(root.getAttribute("data-mirrored"), "true");
  assert.equal(host.requests.length, 0, "permission is never requested without start");
  const video = part<HTMLVideoElement>(root, "video");
  assert.equal(video.muted, true);
  assert.equal(video.hasAttribute("playsinline"), true);
  assert.equal(video.getAttribute("aria-label"), "Camera preview");
  assert.equal(video.getAttribute("data-mirrored"), "true");
  assert.match(video.getAttribute("style") ?? "", /--vize-ui-webcam-capture-scale-x:\s*-1/);
  assert.equal(button(root, "start-button").textContent, "Start camera");
  assert.equal(button(root, "start-button").disabled, false);
  assert.equal(button(root, "start-button").getAttribute("aria-controls"), "camera");
  assert.equal(button(root, "stop-button").disabled, true);
  assert.equal(button(root, "shutter").disabled, true);
  assert.equal(button(root, "shutter").textContent, "Take photo");
  assert.equal(part(root, "device-select").getAttribute("aria-label"), "Camera");
  assert.equal(part(root, "photo").getAttribute("data-state"), "empty");
  assert.ok(root.querySelector("[data-empty]"));
  const status = handle.getByRole("status");
  assert.equal(status.getAttribute("aria-live"), "polite");
  assert.equal(status.textContent, "");
  assert.equal(host.enumerations, 1, "devices are listed on mount without permission");
});

test("start acquires the camera, binds the preview, and lists devices", async () => {
  const host = new FakeHost();
  host.devices = [
    { deviceId: "front", groupId: "g1", kind: "videoinput", label: "" },
    { deviceId: "mic", groupId: "g1", kind: "audioinput", label: "Mic" },
    { deviceId: "back", groupId: "g2", kind: "videoinput", label: "Back camera" },
  ];
  const stream = new FakeStream();
  const handle = mountCamera({ host, constraints: { width: { ideal: 1280 } } });
  const root = handle.root();

  await handle.click(button(root, "start-button"));
  assert.equal(root.getAttribute("data-status"), "requesting");
  assert.equal(handle.getByRole("status").textContent, "Requesting camera access");
  assert.equal(button(root, "start-button").disabled, true);
  assert.deepEqual(host.requests[0]?.constraints, {
    audio: false,
    video: { width: { ideal: 1280 }, facingMode: "user" },
  });

  host.requests[0]?.resolve(stream);
  await flush();
  assert.equal(root.getAttribute("data-status"), "active");
  assert.equal(handle.getByRole("status").textContent, "Camera on");
  assert.ok(Reflect.get(part(root, "video"), "srcObject") === stream);
  assert.equal(button(root, "stop-button").disabled, false);
  assert.equal(button(root, "shutter").disabled, false);
  const options = [...part<HTMLSelectElement>(root, "device-select").options];
  assert.deepEqual(
    options.map((option) => [option.value, option.textContent]),
    [
      ["front", "Camera 1"],
      ["back", "Back camera"],
    ],
  );
  assert.deepEqual(handle.wrapper.emitted("streamChange"), [[stream]]);
  assert.deepEqual(handle.wrapper.emitted("statusChange"), [
    ["requesting", "idle"],
    ["active", "requesting"],
  ]);
  assert.equal(await handle.exposes<WebcamCaptureRootExpose>().start(), stream);
  assert.equal(host.requests.length, 1, "an active stream is reused");
});

test("acquisition failures are normalized like useUserMedia", async () => {
  const host = new FakeHost();
  const cause = mediaError("NotAllowedError");
  host.auto = { error: cause };
  const handle = mountCamera({ host });
  const root = handle.root();
  await handle.exposes<WebcamCaptureRootExpose>().start();
  await flush();
  assert.equal(root.getAttribute("data-status"), "error");
  assert.deepEqual(handle.wrapper.emitted("error"), [[{ code: "permission-denied", cause }]]);
  assert.equal(handle.exposes<WebcamCaptureRootExpose>().error?.code, "permission-denied");
  assert.equal(handle.getByRole("status").textContent, "Camera permission was denied");
  assert.equal(button(root, "start-button").disabled, false, "errors can be retried");

  const unsupported = mountCamera({ host: null });
  await unsupported.exposes<WebcamCaptureRootExpose>().start();
  await flush();
  assert.equal(unsupported.exposes<WebcamCaptureRootExpose>().error?.code, "unsupported");
  assert.equal(
    unsupported.getByRole("status").textContent,
    "Camera is not supported in this browser",
  );
});

test("stop and ended tracks release owned streams, and late streams are discarded", async () => {
  const host = new FakeHost();
  const first = new FakeStream([new FakeTrack(), new FakeTrack("audio")]);
  const handle = mountCamera({ host, audio: true });
  const root = handle.root();
  const exposed = handle.exposes<WebcamCaptureRootExpose>();

  void exposed.start();
  host.requests[0]?.resolve(first);
  await flush();
  assert.equal(host.requests[0]?.constraints?.audio, true);
  await handle.click(button(root, "stop-button"));
  assert.equal(root.getAttribute("data-status"), "idle");
  assert.ok(first.tracks.every((track) => track.stopped));
  assert.ok(Reflect.get(part(root, "video"), "srcObject") === null);

  const late = new FakeStream();
  void exposed.start();
  exposed.stop();
  host.requests[1]?.resolve(late);
  await flush();
  assert.equal(root.getAttribute("data-status"), "idle");
  assert.equal(late.tracks[0]?.stopped, true, "a stream resolved after stop is released");

  const unplugged = new FakeStream();
  void exposed.start();
  host.requests[2]?.resolve(unplugged);
  await flush();
  assert.equal(root.getAttribute("data-status"), "active");
  unplugged.tracks[0]?.end();
  await flush();
  assert.equal(root.getAttribute("data-status"), "idle");
});

test("switching cameras and selecting devices restart an owned stream", async () => {
  const host = new FakeHost();
  host.auto = { stream: new FakeStream() };
  host.devices = [{ deviceId: "back", groupId: "g", kind: "videoinput", label: "Back" }];
  const handle = mountInteraction(WebcamCaptureRoot, {
    props: { host },
    record: ["update:facingMode", "update:deviceId"],
    slots: {
      default: () => [
        h(WebcamCaptureVideo),
        h(WebcamCaptureSwitchCamera),
        h(WebcamCaptureDeviceSelect),
      ],
    },
  });
  cleanups.push(() => handle.unmount());
  const root = handle.root();
  await handle.exposes<WebcamCaptureRootExpose>().start();
  await flush();

  await handle.click(button(root, "switch-camera"));
  await flush();
  assert.equal(root.getAttribute("data-facing-mode"), "environment");
  assert.equal(root.getAttribute("data-mirrored"), "false");
  assert.deepEqual(handle.wrapper.emitted("update:facingMode"), [["environment"]]);
  const restarted = host.requests.at(-1)?.constraints?.video;
  assert.ok(typeof restarted === "object" && restarted.facingMode === "environment");

  const select = part<HTMLSelectElement>(root, "device-select");
  select.value = "back";
  select.dispatchEvent(new Event("change"));
  await flush();
  assert.deepEqual(handle.wrapper.emitted("update:deviceId"), [["back"]]);
  const selected = host.requests.at(-1)?.constraints?.video;
  assert.ok(
    typeof selected === "object" && JSON.stringify(selected.deviceId) === '{"exact":"back"}',
  );
  assert.equal(root.getAttribute("data-status"), "active");
});

test("external streams are displayed but never acquired or stopped", async () => {
  const host = new FakeHost();
  const stream = new FakeStream();
  const handle = mountCamera({ host, stream });
  const root = handle.root();
  await flush();
  assert.equal(root.getAttribute("data-status"), "active");
  assert.equal(root.getAttribute("data-external"), "true");
  assert.ok(Reflect.get(part(root, "video"), "srcObject") === stream);
  assert.equal(button(root, "start-button").disabled, true);
  assert.equal(button(root, "stop-button").disabled, true);
  assert.equal(button(root, "switch-camera").disabled, true);
  assert.equal(await handle.exposes<WebcamCaptureRootExpose>().start(), undefined);
  handle.exposes<WebcamCaptureRootExpose>().stop();
  assert.equal(host.requests.length, 0);
  assert.equal(stream.tracks[0]?.stopped, false);

  await handle.wrapper.setProps({ stream: null });
  assert.equal(root.getAttribute("data-status"), "idle");
  handle.unmount();
  cleanups.pop();
  assert.equal(stream.tracks[0]?.stopped, false, "consumer tracks survive unmount");
});

test("the shutter captures a mirrored, cropped frame and owns the photo URL", async () => {
  const canvas = installFakeCanvas();
  const urls = installFakeObjectUrls();
  cleanups.push(
    () => canvas.restore(),
    () => urls.restore(),
  );
  const host = new FakeHost();
  host.auto = { stream: new FakeStream() };
  const handle = mountCamera({
    host,
    aspectRatio: 1,
    captureType: "image/jpeg",
    captureQuality: 0.8,
    maxWidth: 240,
  });
  const root = handle.root();
  await handle.exposes<WebcamCaptureRootExpose>().start();
  await flush();
  setVideoSize(part<HTMLVideoElement>(root, "video"), 640, 480);

  await handle.click(button(root, "shutter"));
  await flush();
  assert.deepEqual(canvas.sizes, [{ width: 240, height: 240 }]);
  assert.deepEqual(canvas.calls, [
    "translate(240,0)",
    "scale(-1,1)",
    "drawImage(80,0,480,480,0,0,240,240)",
  ]);
  assert.deepEqual(canvas.blobs, [{ type: "image/jpeg", quality: 0.8 }]);
  const captured = handle.wrapper.emitted("capture")?.[0]?.[0];
  assert.ok(captured && typeof captured === "object" && "blob" in captured);
  const image = part(root, "photo").querySelector("img");
  assert.equal(image?.getAttribute("src"), "blob:test/1");
  assert.equal(image?.getAttribute("alt"), "Captured photo");
  assert.equal(image?.getAttribute("width"), "240");
  assert.equal(part(root, "photo").getAttribute("data-state"), "captured");
  assert.equal(handle.getByRole("status").textContent, "Photo taken");

  await handle.click(button(root, "shutter"));
  await flush();
  assert.deepEqual(urls.revoked, ["blob:test/1"]);
  handle.exposes<WebcamCaptureRootExpose>().clearPhoto();
  await nextTick();
  assert.deepEqual(urls.revoked, ["blob:test/1", "blob:test/2"]);
  assert.equal(part(root, "photo").querySelector("img"), null);
});

test("capture failures emit captureError and a missing frame captures nothing", async () => {
  const canvas = installFakeCanvas({ context: false });
  cleanups.push(() => canvas.restore());
  const host = new FakeHost();
  host.auto = { stream: new FakeStream() };
  const handle = mountCamera({ host });
  const exposed = handle.exposes<WebcamCaptureRootExpose>();
  assert.equal(await exposed.capture(), null, "idle cameras capture nothing");
  await exposed.start();
  await flush();
  assert.equal(await exposed.capture(), null);
  const noFrame = handle.wrapper.emitted("captureError")?.[0]?.[0];
  assert.ok(noFrame instanceof Error && /VIZE_UI_WEBCAM_NO_FRAME/.test(noFrame.message));
  setVideoSize(part<HTMLVideoElement>(handle.root(), "video"), 10, 10);
  assert.equal(await exposed.capture(), null);
  const noCanvas = handle.wrapper.emitted("captureError")?.[1]?.[0];
  assert.ok(noCanvas instanceof Error && /VIZE_UI_WEBCAM_NO_CANVAS/.test(noCanvas.message));
});

test("countdowns announce each second, block the shutter, and cancel on stop", async () => {
  const canvas = installFakeCanvas();
  const urls = installFakeObjectUrls();
  cleanups.push(
    () => canvas.restore(),
    () => urls.restore(),
  );
  const host = new FakeHost();
  host.auto = { stream: new FakeStream() };
  const handle = mountCamera({ host, countdownInterval: 5 }, { countdown: 2 });
  const root = handle.root();
  await handle.exposes<WebcamCaptureRootExpose>().start();
  await flush();
  setVideoSize(part<HTMLVideoElement>(root, "video"), 20, 10);

  await handle.click(button(root, "shutter"));
  assert.equal(handle.getByRole("status").textContent, "Taking photo in 2");
  assert.equal(button(root, "shutter").getAttribute("data-countdown"), "2");
  assert.equal(button(root, "shutter").disabled, true);
  await new Promise((resolve) => setTimeout(resolve, 8));
  await nextTick();
  assert.equal(handle.getByRole("status").textContent, "Taking photo in 1");
  await new Promise((resolve) => setTimeout(resolve, 20));
  await flush();
  assert.equal(handle.wrapper.emitted("capture")?.length, 1);
  assert.equal(button(root, "shutter").hasAttribute("data-countdown"), false);

  await handle.click(button(root, "shutter"));
  handle.exposes<WebcamCaptureRootExpose>().stop();
  await new Promise((resolve) => setTimeout(resolve, 20));
  await flush();
  assert.equal(handle.wrapper.emitted("capture")?.length, 1, "stop cancels the countdown");
});

test("messages localize every default label and announcement", async () => {
  const host = new FakeHost();
  host.auto = { error: mediaError("NotFoundError") };
  const handle = mountCamera({
    host,
    messages: {
      start: "カメラを開始",
      shutter: "撮影",
      preview: "プレビュー",
      error: (code: string) => `エラー: ${code}`,
    },
  });
  const root = handle.root();
  assert.equal(button(root, "start-button").textContent, "カメラを開始");
  assert.equal(button(root, "shutter").textContent, "撮影");
  assert.equal(button(root, "stop-button").textContent, "Stop camera");
  assert.equal(part(root, "video").getAttribute("aria-label"), "プレビュー");
  await handle.exposes<WebcamCaptureRootExpose>().start();
  await flush();
  assert.equal(handle.getByRole("status").textContent, "エラー: not-found");
});

test("autoStart requests on mount and devicechange refreshes the list", async () => {
  const host = new FakeHost();
  host.auto = { stream: new FakeStream() };
  const handle = mountCamera({ host, autoStart: true, mirrored: false });
  await flush();
  assert.equal(handle.root().getAttribute("data-status"), "active");
  assert.equal(handle.root().getAttribute("data-mirrored"), "false");
  const before = host.enumerations;
  host.devices = [{ deviceId: "usb", groupId: "u", kind: "videoinput", label: "USB" }];
  host.dispatchEvent(new Event("devicechange"));
  await flush();
  assert.equal(host.enumerations, before + 1);
  assert.equal(handle.root().querySelector("output")?.textContent, "1");
});

test("unmounting stops owned tracks and revokes the photo URL", async () => {
  const canvas = installFakeCanvas();
  const urls = installFakeObjectUrls();
  cleanups.push(
    () => canvas.restore(),
    () => urls.restore(),
  );
  const host = new FakeHost();
  const stream = new FakeStream();
  host.auto = { stream };
  const handle = mountCamera({ host });
  await handle.exposes<WebcamCaptureRootExpose>().start();
  await flush();
  setVideoSize(part<HTMLVideoElement>(handle.root(), "video"), 4, 4);
  await handle.exposes<WebcamCaptureRootExpose>().capture();
  handle.unmount();
  cleanups.pop();
  assert.equal(stream.tracks[0]?.stopped, true);
  assert.deepEqual(urls.revoked, ["blob:test/1"]);
});

test("buttons honor preventDefault from click listeners", async () => {
  const host = new FakeHost();
  const blocked = (event: MouseEvent) => event.preventDefault();
  const handle = mountInteraction(WebcamCaptureRoot, {
    props: { host },
    slots: {
      default: () => [
        h(WebcamCaptureStartButton, { onClick: blocked }),
        h(WebcamCaptureSwitchCamera, { onClick: blocked }),
      ],
    },
  });
  cleanups.push(() => handle.unmount());
  for (const element of handle.root().querySelectorAll("button")) {
    element.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
  }
  await flush();
  assert.equal(host.requests.length, 0);
  assert.equal(handle.root().getAttribute("data-facing-mode"), "user");
});

test("compound parts require a matching root provider", () => {
  for (const part of [
    WebcamCaptureVideo,
    WebcamCaptureStartButton,
    WebcamCaptureStopButton,
    WebcamCaptureSwitchCamera,
    WebcamCaptureDeviceSelect,
    WebcamCaptureShutter,
    WebcamCapturePhoto,
    WebcamCaptureStatusMessage,
  ]) {
    assert.throws(
      () => mountInteraction(part),
      /VIZE_UI_CONTEXT_MISSING: WebcamCapture requires a matching provider/,
    );
  }
});
