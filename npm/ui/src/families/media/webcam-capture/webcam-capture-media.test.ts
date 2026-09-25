import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { WebcamCaptureError, captureFrame, captureGeometry } from "./webcam-capture-frame.ts";
import {
  createWebcamConstraints,
  normalizeUserMediaError,
  resolveWebcamCaptureMessages,
  stopWebcamStream,
  toWebcamDevices,
  webcamCaptureDefaultMessages,
} from "./webcam-capture-media.ts";
import { FakeStream, FakeTrack, mediaError } from "./webcam-capture-test-utils.ts";

test("normalizes getUserMedia rejections with the useUserMedia code table", () => {
  const table: readonly (readonly [unknown, string])[] = [
    [mediaError("NotAllowedError"), "permission-denied"],
    [mediaError("SecurityError"), "permission-denied"],
    [mediaError("NotFoundError"), "not-found"],
    [mediaError("NotReadableError"), "not-readable"],
    [mediaError("OverconstrainedError"), "overconstrained"],
    [mediaError("AbortError"), "aborted"],
    [new TypeError("bad"), "invalid-constraints"],
    [mediaError("Weird"), "failed"],
    ["string", "failed"],
    [null, "failed"],
  ];
  for (const [error, code] of table) assert.equal(normalizeUserMediaError(error), code);
});

test("builds facing-mode and exact-device constraints", () => {
  assert.deepEqual(createWebcamConstraints({ facingMode: "environment" }), {
    audio: false,
    video: { facingMode: "environment" },
  });
  assert.deepEqual(
    createWebcamConstraints({
      audio: true,
      deviceId: "cam",
      facingMode: "user",
      video: { frameRate: 30 },
    }),
    { audio: true, video: { frameRate: 30, deviceId: { exact: "cam" } } },
  );
});

test("keeps video inputs with positional fallback labels and stops every track", () => {
  const devices = toWebcamDevices(
    [
      { deviceId: "a", groupId: "1", kind: "videoinput", label: "" },
      { deviceId: "m", groupId: "1", kind: "audioinput", label: "Mic" },
      { deviceId: "b", groupId: "2", kind: "videoinput", label: "Rear" },
    ],
    (position) => `#${position}`,
  );
  assert.deepEqual(devices, [
    { deviceId: "a", groupId: "1", label: "#1" },
    { deviceId: "b", groupId: "2", label: "Rear" },
  ]);
  assert.ok(Object.isFrozen(devices));
  const stream = new FakeStream([new FakeTrack(), new FakeTrack("audio")]);
  stopWebcamStream(stream);
  assert.ok(stream.tracks.every((track) => track.stopped));
});

test("merges partial messages over English defaults", () => {
  assert.equal(resolveWebcamCaptureMessages(undefined), webcamCaptureDefaultMessages);
  const merged = resolveWebcamCaptureMessages({ shutter: "Cheese" });
  assert.equal(merged.shutter, "Cheese");
  assert.equal(merged.start, "Start camera");
  assert.equal(merged.countdown(3), "Taking photo in 3");
  assert.equal(merged.error("not-found"), "No camera was found");
});

test("computes centered crops and downscaled output geometry", () => {
  assert.deepEqual(captureGeometry(640, 480), {
    sx: 0,
    sy: 0,
    sw: 640,
    sh: 480,
    width: 640,
    height: 480,
  });
  assert.deepEqual(captureGeometry(640, 480, { aspectRatio: 1 }), {
    sx: 80,
    sy: 0,
    sw: 480,
    sh: 480,
    width: 480,
    height: 480,
  });
  assert.deepEqual(captureGeometry(480, 640, { aspectRatio: 16 / 9 }), {
    sx: 0,
    sy: 185,
    sw: 480,
    sh: 270,
    width: 480,
    height: 270,
  });
  assert.deepEqual(captureGeometry(1920, 1080, { maxWidth: 960 }), {
    sx: 0,
    sy: 0,
    sw: 1920,
    sh: 1080,
    width: 960,
    height: 540,
  });
  assert.deepEqual(captureGeometry(0, 0, { aspectRatio: 1 }), {
    sx: 0,
    sy: 0,
    sw: 0,
    sh: 0,
    width: 0,
    height: 0,
  });
  assert.deepEqual(captureGeometry(100, 50, { aspectRatio: -1, maxWidth: Number.NaN }).width, 100);
});

test("captureFrame refuses videos without a frame", async () => {
  const video = document.createElement("video");
  await assert.rejects(
    () => captureFrame(video),
    (error: unknown) =>
      error instanceof WebcamCaptureError && error.code === "VIZE_UI_WEBCAM_NO_FRAME",
  );
});
