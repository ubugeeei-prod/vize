import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import {
  themeDensityAttribute,
  themePresetAttribute,
  themeTokens,
  themeTokenVar,
} from "./theme.ts";

/** A consumer that touches the token surface during setup, exactly like real SSR. */
const SsrProbe = defineComponent({
  name: "ThemeSsrProbe",
  setup() {
    const accent = themeTokenVar("color-accent");
    const canvas = themeTokens["color-canvas"];
    return () =>
      h(
        "section",
        {
          [themePresetAttribute]: "atelier",
          [themeDensityAttribute]: "compact",
          "data-accent": accent,
          "data-canvas": canvas,
        },
        "Themed",
      );
  },
});

/** Render with every platform global a server lacks removed. */
async function renderWithoutPlatformGlobals(): Promise<string> {
  const originalDocument = globalThis.document;
  const originalMatchMedia = globalThis.matchMedia;
  // @ts-expect-error simulating a server runtime without a document.
  delete globalThis.document;
  // @ts-expect-error simulating a server runtime without matchMedia.
  delete globalThis.matchMedia;
  try {
    return await renderToString(createSSRApp(SsrProbe));
  } finally {
    globalThis.document = originalDocument;
    globalThis.matchMedia = originalMatchMedia;
  }
}

test("renders byte-identical SSR markup without platform globals", async () => {
  const outputs = [await renderWithoutPlatformGlobals(), await renderWithoutPlatformGlobals()];
  assert.equal(outputs[0], outputs[1]);
  assert.equal(
    outputs[0],
    '<section data-vize-theme="atelier" data-vize-density="compact"' +
      ' data-accent="var(--vize-ui-color-accent)" data-canvas="Canvas">Themed</section>',
  );
});

test("hydrates a themed consumer without replacement or diagnostics", async () => {
  const serverHtml = await renderWithoutPlatformGlobals();
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverTarget = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);

  try {
    app.mount(host);
    assert.equal(host.firstElementChild, serverTarget);
    assert.equal(host.firstElementChild?.getAttribute("data-vize-theme"), "atelier");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
