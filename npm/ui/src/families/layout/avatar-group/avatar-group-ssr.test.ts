import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Avatar from "../avatar/avatar.vue";
import AvatarGroup from "./avatar-group.vue";

const names = ["Ada", "Grace", "Linus", "Barbara"];

const SsrProbe = defineComponent({
  name: "AvatarGroupSsrProbe",
  setup: () => () =>
    h(
      AvatarGroup,
      { items: names, max: 3, spacing: -6, ariaLabel: "Reviewers" },
      { item: ({ item }: { item: string }) => h(Avatar, { name: item, fallback: item.charAt(0) }) },
    ),
});

test("renders byte-identical avatar group markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /^<ul aria-label="Reviewers" style="--vize-ui-avatar-group-spacing: -6px"/);
  assert.match(left, /data-state="collapsed"/);
  assert.match(left, /role="img" aria-label="2 more"/);
  assert.equal(left.match(/data-vize-ui="avatar-group-item"/g)?.length, 2);
});

test("hydrates avatar group markup without warnings or node replacement", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
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
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
