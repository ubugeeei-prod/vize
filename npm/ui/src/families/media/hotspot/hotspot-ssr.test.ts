import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import HotspotArea from "./hotspot-area.vue";
import HotspotContent from "./hotspot-content.vue";
import HotspotImage from "./hotspot-image.vue";
import HotspotMarker from "./hotspot-marker.vue";
import HotspotRoot from "./hotspot-root.vue";

const SsrProbe = defineComponent({
  name: "HotspotSsrProbe",
  setup: () => () =>
    h(HotspotRoot, { defaultActive: "lamp" }, () => [
      h(HotspotImage, { src: "/room.jpg", alt: "Living room" }),
      h(HotspotMarker, { id: "lamp", x: 25, y: 40, label: "Floor lamp" }, () =>
        h(HotspotContent, { portalDisabled: true }, () => "Brass floor lamp"),
      ),
      h(HotspotMarker, { id: "sofa", x: 60, y: 70, label: "Sofa" }, () =>
        h(HotspotContent, { portalDisabled: true }, () => "Linen sofa"),
      ),
      h(HotspotArea, {
        id: "window",
        label: "Window",
        shape: { type: "rect", x: 70, y: 5, width: 25, height: 30 },
      }),
    ]),
});

test("renders byte-identical hotspot markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(
    left,
    /^<div id="vize-v-\d+-hotspot" data-vize-ui="hotspot-root" part="root" data-state="open" data-active="lamp"/,
  );
  assert.match(left, /--vize-ui-hotspot-x:25%;--vize-ui-hotspot-y:40%/);
  assert.match(left, /id="vize-v-\d+-hotspot-marker-lamp"/);
  assert.match(left, /aria-label="Floor lamp"/);
  assert.match(left, /Brass floor lamp/);
  assert.match(left, /viewBox="0 0 100 100"/);
  assert.match(left, /role="button" tabindex="0" aria-label="Window" aria-pressed="false"/);
});

test("hydrates hotspot markup without warnings or node replacement", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverMarkers = [...host.querySelectorAll('[data-vize-ui="hotspot-marker"]')];
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);
  let mounted = false;
  try {
    app.mount(host);
    mounted = true;
    assert.ok(host.firstElementChild === serverRoot);
    const markers = [...host.querySelectorAll('[data-vize-ui="hotspot-marker"]')];
    assert.ok(markers.every((marker, index) => marker === serverMarkers[index]));
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
