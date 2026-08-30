import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import AnnouncerProvider from "./announcer-provider.vue";
import { announcerContext } from "./announcer.ts";

const AnnouncingIsland = defineComponent({
  name: "AnnouncerSsrIsland",
  setup() {
    const announcer = announcerContext.use();
    announcer.announce("From the island");
    return () => h("p", "Island content");
  },
});

const SsrProbe = defineComponent({
  name: "AnnouncerSsrProbe",
  setup() {
    return () =>
      h(AnnouncerProvider, null, {
        default: () => h(AnnouncerProvider, null, { default: () => h(AnnouncingIsland) }),
      });
  },
});

test("renders byte-identical markup with a single live-region pair across islands", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.equal(html.match(/aria-live="polite"/g)?.length, 1);
  assert.equal(html.match(/aria-live="assertive"/g)?.length, 1);
  assert.match(html, /data-vize-announcer="owner"/);
  assert.match(html, /data-vize-announcer="delegate"/);
  assert.match(html, /Island content/);
  assert.doesNotMatch(html, /From the island/);
});
