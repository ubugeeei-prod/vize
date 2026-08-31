import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import StatusLight from "./status-light.vue";

const SsrProbe = defineComponent({
  name: "StatusLightSsrProbe",
  setup() {
    return () =>
      h(
        StatusLight,
        {
          ariaDescribedby: "service-status-help",
          ariaLabel: "Service online",
          size: "sm",
          state: "online",
          tone: "success",
        },
        {
          default: () => h("span", { id: "service-status-help" }, "API cluster"),
        },
      );
  },
});

test("renders byte-identical image markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<span/);
  assert.match(html, /role="img"/);
  assert.match(html, /aria-label="Service online"/);
  assert.match(html, /aria-describedby="service-status-help"/);
  assert.match(html, /data-vize-ui="status-light"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-state="online"/);
  assert.match(html, /data-tone="success"/);
  assert.match(html, /data-size="sm"/);
  assert.match(html, /data-aria-state="img"/);
  assert.match(html, /data-decorative="false"/);
  assert.match(html, /API cluster/);
  assert.doesNotMatch(html, /class=|style=|tabindex=|function/);
  assert.doesNotMatch(html, /aria-live|aria-atomic|aria-hidden/);
});

test("hydrates labelled markup without replacing the status-light root", async () => {
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
    const hydrated = host.querySelector<HTMLElement>("[data-vize-ui='status-light']");
    assert.ok(hydrated);
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(hydrated.getAttribute("role"), "img");
    assert.equal(hydrated.getAttribute("data-state"), "online");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});

test("renders server status markup with consumer-owned labels", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "StatusLightStatusSsrProbe",
      setup() {
        return () =>
          h(
            StatusLight,
            {
              ariaLabelledby: "deploy-status-label",
              atomic: false,
              role: "status",
              state: "busy",
              tone: "warning",
            },
            {
              default: () => h("span", { id: "deploy-status-label" }, "Deploy status"),
            },
          );
      },
    }),
  );

  assert.match(html, /^<span/);
  assert.match(html, /role="status"/);
  assert.match(html, /aria-labelledby="deploy-status-label"/);
  assert.match(html, /aria-live="polite"/);
  assert.match(html, /aria-atomic="false"/);
  assert.match(html, /data-state="busy"/);
  assert.match(html, /data-tone="warning"/);
  assert.match(html, /data-aria-state="status"/);
  assert.doesNotMatch(html, /aria-hidden/);
});

test("renders decorative server markup without accessible semantics", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "StatusLightDecorativeSsrProbe",
      setup() {
        return () =>
          h(StatusLight, {
            ariaHidden: true,
            ariaLabel: "Ignored",
            role: "status",
            state: "offline",
            tone: "danger",
          });
      },
    }),
  );

  assert.match(html, /^<span/);
  assert.match(html, /aria-hidden="true"/);
  assert.match(html, /data-state="offline"/);
  assert.match(html, /data-tone="danger"/);
  assert.match(html, /data-aria-state="decorative"/);
  assert.doesNotMatch(html, /role="status"/);
  assert.doesNotMatch(html, /aria-label=|aria-describedby=|aria-live|aria-atomic/);
});
