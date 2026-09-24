import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import { useScrollSpy } from "./scroll-spy.ts";

const Probe = defineComponent({
  name: "ScrollSpySsrProbe",
  setup() {
    const spy = useScrollSpy({ ids: ["intro", "api"], initialActiveId: "intro" });
    return () =>
      h("nav", { "data-active": spy.activeId.value ?? "none" }, [
        h(
          "a",
          {
            href: "#intro",
            "aria-current": spy.activeId.value === "intro" ? "location" : undefined,
          },
          "Intro",
        ),
        h("a", { href: "#api" }, "API"),
      ]);
  },
});

test("renders the initial active id identically across SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.match(outputs[0] ?? "", /data-active="intro"/);
  assert.match(outputs[0] ?? "", /aria-current="location"/);
});

test("hydrates with the initial active id before any measurement", async () => {
  const html = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Probe);
  try {
    app.mount(host);
    assert.equal(host.querySelector("nav")?.getAttribute("data-active"), "intro");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
  }
});
