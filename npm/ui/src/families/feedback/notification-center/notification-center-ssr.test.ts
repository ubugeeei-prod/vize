import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import NotificationCenterEmpty from "./notification-center-empty.vue";
import NotificationCenterList from "./notification-center-list.vue";
import NotificationCenterRoot from "./notification-center-root.vue";
import NotificationCenterTrigger from "./notification-center-trigger.vue";

const Probe = defineComponent({
  name: "NotificationCenterSsrProbe",
  setup() {
    return () =>
      h(
        NotificationCenterRoot,
        {
          now: () => 0,
          initial: [
            { id: "a", title: "Build passed", createdAt: 1 },
            { id: "b", title: "Review requested", description: "PR 42", createdAt: 2 },
          ],
        },
        () => [
          h(NotificationCenterTrigger, null, () => "Bell"),
          h(NotificationCenterList, null, () => h(NotificationCenterEmpty)),
        ],
      );
  },
});

test("renders byte-identical feed markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /data-vize-ui="notification-center-root"/);
  assert.match(html, /aria-label="Notifications, 2 unread"/);
  assert.match(html, /role="feed"/);
  assert.match(html, /aria-controls="vize-v-\d+-notification-center-list"/);
  assert.match(html, /aria-posinset="1"[^>]*aria-setsize="2"/);
  assert.match(html, /id="vize-v-\d+-notification-center-item-b-description"/);
});

test("hydrates the feed without diagnostics", async () => {
  const serverHtml = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverFeed = host.querySelector('[role="feed"]');
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
    assert.ok(host.querySelector('[role="feed"]') === serverFeed);
    assert.equal(host.querySelectorAll("article").length, 2);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
