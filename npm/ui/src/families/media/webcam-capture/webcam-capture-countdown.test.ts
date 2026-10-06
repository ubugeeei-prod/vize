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
  flush,
  installFakeCanvas,
  installFakeSrcObject,
  installFakeObjectUrls,
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
