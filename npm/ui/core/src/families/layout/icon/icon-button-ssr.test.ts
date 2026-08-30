import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Icon from "./icon.vue";
import IconButton from "./icon-button.vue";

const SsrProbe = defineComponent({
  name: "IconButtonSsrProbe",
  setup() {
    return () =>
      h(
        IconButton,
        {
          ariaLabel: "Refresh feed",
          size: "sm",
          tone: "accent",
          variant: "soft",
        },
        {
          default: () =>
            h(
              Icon,
              { ariaHidden: true, size: "sm" },
              {
                default: () => h("path", { d: "M4 12h16" }),
              },
            ),
        },
      );
  },
});

test("renders byte-identical native icon-button markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<button/);
  assert.match(html, /type="button"/);
  assert.match(html, /aria-label="Refresh feed"/);
  assert.match(html, /data-vize-ui="icon-button"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-state="idle"/);
  assert.match(html, /data-size="sm"/);
  assert.match(html, /data-tone="accent"/);
  assert.match(html, /data-variant="soft"/);
  assert.match(html, /data-name="present"/);
  assert.match(html, /data-vize-ui="icon"/);
  assert.match(html, /aria-hidden="true"/);
  assert.doesNotMatch(html, /class=|style=|tabindex=|aria-busy=/);
});

test("hydrates native icon-button markup without warnings or root replacement", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  assert.ok(serverRoot);
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
    const hydrated = host.querySelector("[data-vize-ui='icon-button']");
    assert.ok(hydrated instanceof HTMLButtonElement);
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(hydrated.getAttribute("aria-label"), "Refresh feed");
    assert.equal(hydrated.getAttribute("data-state"), "idle");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});

test("renders non-native server markup with button semantics", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "IconButtonNonNativeSsrProbe",
      setup() {
        return () =>
          h(
            IconButton,
            {
              ariaLabelledby: "pin-label",
              as: "span",
              native: false,
              size: "lg",
              variant: "outline",
            },
            {
              default: () => h("span", { id: "pin-label" }, "Pin"),
            },
          );
      },
    }),
  );

  assert.match(html, /^<span/);
  assert.match(html, /role="button"/);
  assert.match(html, /tabindex="0"/);
  assert.match(html, /aria-labelledby="pin-label"/);
  assert.match(html, /data-vize-ui="icon-button"/);
  assert.match(html, /data-size="lg"/);
  assert.match(html, /data-variant="outline"/);
  assert.doesNotMatch(html, /aria-label=|disabled=|style=|class=/);
});
