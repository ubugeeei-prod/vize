import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  WebcamCaptureDeviceSelect,
  WebcamCapturePhoto,
  WebcamCaptureRoot,
  WebcamCaptureShutter,
  WebcamCaptureStartButton,
  WebcamCaptureStatusMessage,
  WebcamCaptureStopButton,
  WebcamCaptureSwitchCamera,
  WebcamCaptureVideo,
} from "./webcam-capture.ts";

function camera(id: string) {
  // `host: null` keeps hydration from touching real devices in the harness.
  return h(WebcamCaptureRoot, { id, host: null }, () => [
    h(WebcamCaptureVideo),
    h(WebcamCaptureStartButton),
    h(WebcamCaptureStopButton),
    h(WebcamCaptureSwitchCamera),
    h(WebcamCaptureDeviceSelect),
    h(WebcamCaptureShutter),
    h(WebcamCapturePhoto),
    h(WebcamCaptureStatusMessage),
  ]);
}

function part(host: HTMLElement, name: string): Element {
  const element = host.querySelector(`[data-vize-ui="webcam-capture-${name}"]`);
  assert.ok(element, `${name} must render`);
  return element;
}

function fixture(
  name: string,
  file: string,
  server: RegExp,
  hydrated: (host: HTMLElement) => void,
): RuntimeFixture {
  return {
    name,
    sourceFile: `families/media/webcam-capture/${file}`,
    render: () => camera(name),
    assertServerMarkup(html) {
      assert.match(html, server);
    },
    assertHydratedDom: hydrated,
  };
}

export const webcamCaptureRuntimeFixtures: readonly RuntimeFixture[] = [
  fixture(
    "webcam-capture",
    "webcam-capture-root.vue",
    /<div id="webcam-capture" data-vize-ui="webcam-capture-root" part="root" data-status="idle"/,
    (host) => assert.equal(part(host, "root").getAttribute("data-status"), "idle"),
  ),
  fixture(
    "webcam-capture-video",
    "webcam-capture-video.vue",
    /<video muted autoplay playsinline aria-label="Camera preview"/,
    (host) => assert.ok(part(host, "video") instanceof HTMLVideoElement),
  ),
  fixture(
    "webcam-capture-start-button",
    "webcam-capture-start-button.vue",
    /data-vize-ui="webcam-capture-start-button"[^>]*>(?:<!--\[-->)?Start camera/,
    (host) => assert.ok(part(host, "start-button") instanceof HTMLButtonElement),
  ),
  fixture(
    "webcam-capture-stop-button",
    "webcam-capture-stop-button.vue",
    /<button type="button" disabled aria-controls="webcam-capture-stop-button"/,
    (host) => assert.ok(part(host, "stop-button") instanceof HTMLButtonElement),
  ),
  fixture(
    "webcam-capture-switch-camera",
    "webcam-capture-switch-camera.vue",
    />(?:<!--\[-->)?Switch camera/,
    (host) => assert.ok(part(host, "switch-camera") instanceof HTMLButtonElement),
  ),
  fixture(
    "webcam-capture-device-select",
    "webcam-capture-device-select.vue",
    /<select[^>]*disabled aria-label="Camera"/,
    (host) => assert.ok(part(host, "device-select") instanceof HTMLSelectElement),
  ),
  fixture(
    "webcam-capture-shutter",
    "webcam-capture-shutter.vue",
    /data-vize-ui="webcam-capture-shutter"[^>]*>(?:<!--\[-->)?Take photo/,
    (host) => assert.equal(part(host, "shutter").hasAttribute("disabled"), true),
  ),
  fixture(
    "webcam-capture-photo",
    "webcam-capture-photo.vue",
    /<figure data-vize-ui="webcam-capture-photo" part="photo" data-state="empty"/,
    (host) => assert.equal(part(host, "photo").getAttribute("data-state"), "empty"),
  ),
  fixture(
    "webcam-capture-status-message",
    "webcam-capture-status-message.vue",
    /role="status" aria-live="polite" aria-atomic="true"/,
    (host) => assert.equal(part(host, "status").getAttribute("role"), "status"),
  ),
];
