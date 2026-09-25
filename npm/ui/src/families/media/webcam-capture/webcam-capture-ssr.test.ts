import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import WebcamCaptureDeviceSelect from "./webcam-capture-device-select.vue";
import WebcamCapturePhoto from "./webcam-capture-photo.vue";
import WebcamCaptureRoot from "./webcam-capture-root.vue";
import WebcamCaptureShutter from "./webcam-capture-shutter.vue";
import WebcamCaptureStartButton from "./webcam-capture-start-button.vue";
import WebcamCaptureStatusMessage from "./webcam-capture-status-message.vue";
import WebcamCaptureStopButton from "./webcam-capture-stop-button.vue";
import WebcamCaptureSwitchCamera from "./webcam-capture-switch-camera.vue";
import WebcamCaptureVideo from "./webcam-capture-video.vue";
import { FakeHost } from "./webcam-capture-test-utils.ts";

function probe(host: FakeHost) {
  return defineComponent({
    name: "WebcamCaptureSsrProbe",
    setup: () => () =>
      h(WebcamCaptureRoot, { host, autoStart: true }, () => [
        h(WebcamCaptureVideo),
        h(WebcamCaptureStartButton),
        h(WebcamCaptureStopButton),
        h(WebcamCaptureSwitchCamera),
        h(WebcamCaptureDeviceSelect),
        h(WebcamCaptureShutter, { countdown: 3 }),
        h(WebcamCapturePhoto),
        h(WebcamCaptureStatusMessage),
      ]),
  });
}

test("renders byte-identical idle markup without requesting the camera on the server", async () => {
  const host = new FakeHost();
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(probe(host))),
    renderToString(createSSRApp(probe(host))),
  ]);
  assert.equal(left, right);
  assert.equal(host.requests.length, 0, "autoStart only runs after mount");
  assert.equal(host.enumerations, 0);
  assert.match(left, /^<div id="vize-v-\d+-webcam-capture" data-vize-ui="webcam-capture-root"/);
  assert.match(left, /data-status="idle"/);
  assert.match(left, /<video muted autoplay playsinline aria-label="Camera preview"/);
  assert.match(left, />Start camera</);
  assert.match(left, /role="status" aria-live="polite"/);
  assert.doesNotMatch(left, /<img/);
});

test("hydrates idle camera markup without warnings or node replacement", async () => {
  const serverHtml = await renderToString(createSSRApp(probe(new FakeHost())));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(probe(new FakeHost()));
  let mounted = false;
  try {
    app.mount(host);
    mounted = true;
    assert.ok(host.firstElementChild === serverRoot);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
