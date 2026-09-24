import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import ColorPickerArea from "./color-picker-area.vue";
import ColorPickerChannelSlider from "./color-picker-channel-slider.vue";
import ColorPickerEyeDropper from "./color-picker-eye-dropper.vue";
import ColorPickerField from "./color-picker-field.vue";
import ColorPickerRoot from "./color-picker-root.vue";
import ColorPickerSwatch from "./color-picker-swatch.vue";
import ColorPickerSwatchGroup from "./color-picker-swatch-group.vue";

const SsrProbe = defineComponent({
  name: "ColorPickerSsrProbe",
  setup: () => () =>
    h(ColorPickerRoot, { defaultValue: "#3366cc", name: "accent" }, () => [
      h(ColorPickerArea),
      h(ColorPickerChannelSlider, { channel: "hue" }),
      h(ColorPickerChannelSlider, { channel: "alpha" }),
      h(ColorPickerSwatchGroup, { ariaLabel: "Presets" }, () => [
        h(ColorPickerSwatch, { value: "#3366cc", label: "Brand" }),
        h(ColorPickerSwatch, { value: "#ff0000", label: "Red" }),
      ]),
      h(ColorPickerField, { ariaLabel: "Hex" }),
      h(ColorPickerEyeDropper, { ariaLabel: "Pick" }),
    ]),
});

test("renders byte-identical color picker markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<div id="vize-v-\d+-color-picker" role="group"/);
  assert.match(html, /--vize-ui-color-picker-color:rgb\(51 102 204\)/);
  assert.match(html, /aria-roledescription="2D slider"[^>]*role="slider"/);
  assert.match(html, /aria-valuetext="Saturation 75%, Brightness 80%"/);
  assert.match(html, /aria-valuetext="Hue 220°"/);
  assert.match(html, /role="radiogroup" aria-label="Presets"/);
  assert.match(
    html,
    /<div(?=[^>]*role="radio")(?=[^>]*aria-checked="true")(?=[^>]*aria-label="Brand")/,
  );
  assert.match(html, /value="#3366cc"[^>]*data-vize-ui="color-picker-field"/);
  assert.match(html, /data-vize-ui="color-picker-eye-dropper"[^>]*data-state="unsupported"/);
  assert.match(html, /type="hidden" name="accent" value="#3366cc"/);
});

test("hydrates generated color picker ids without changing the server contract", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverIds = [...host.querySelectorAll("[id]")].map((node) => node.id);
  assert.ok(serverRoot);
  assert.ok(serverIds.length >= 8);

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
    assert.deepEqual(
      [...host.querySelectorAll("[id]")].map((node) => node.id),
      serverIds,
    );
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
