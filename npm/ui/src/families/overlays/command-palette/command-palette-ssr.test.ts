import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { createCommandRouter } from "../../foundations/command/command.ts";
import CommandPaletteDialog from "./command-palette-dialog.vue";
import CommandPaletteEmpty from "./command-palette-empty.vue";
import CommandPaletteGroup from "./command-palette-group.vue";
import CommandPaletteInput from "./command-palette-input.vue";
import CommandPaletteItem from "./command-palette-item.vue";
import CommandPaletteList from "./command-palette-list.vue";
import CommandPaletteRoot from "./command-palette-root.vue";

const Probe = defineComponent({
  name: "CommandPaletteSsrProbe",
  setup() {
    const router = createCommandRouter();
    router.register({ id: "reload", title: "Reload", run: () => undefined });
    return () =>
      h("div", null, [
        h(CommandPaletteRoot, { router, defaultSearch: "re" }, () => [
          h(CommandPaletteInput),
          h(CommandPaletteList, null, () => [
            h(CommandPaletteGroup, { heading: "App" }, () => [
              h(CommandPaletteItem, { command: "reload" }),
              h(CommandPaletteItem, { textValue: "Quit" }),
            ]),
            h(CommandPaletteEmpty),
          ]),
        ]),
        h(CommandPaletteDialog, null, () => "closed dialog"),
      ]);
  },
});

test("renders byte-identical palette markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /data-vize-ui="command-palette-root"/);
  assert.match(html, /role="combobox"/);
  assert.match(html, /aria-controls="vize-v-\d+-command-palette-list"/);
  assert.match(html, /value="re"/);
  assert.match(html, /role="listbox"/);
  assert.match(html, /role="group"/);
  assert.match(html, /Reload/);
  assert.match(html, /data-vize-ui="command-palette-dialog"[^>]*data-state="closed"/);
  assert.doesNotMatch(html, /closed dialog/);
});

test("hydrates without mismatches and activates the first visible option", async () => {
  const serverHtml = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverInput = host.querySelector('[role="combobox"]');
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Probe);
  let mounted = false;

  try {
    app.mount(host);
    mounted = true;
    await nextTick();
    await nextTick();
    const input = host.querySelector('[role="combobox"]');
    assert.ok(input === serverInput);
    assert.deepEqual(diagnostics, []);
    const first = host.querySelector('[role="option"]');
    assert.equal(input?.getAttribute("aria-activedescendant"), first?.id);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
