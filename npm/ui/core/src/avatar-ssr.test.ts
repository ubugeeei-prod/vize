import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Avatar from "./avatar.vue";

const SsrProbe = defineComponent({
  name: "AvatarSsrProbe",
  setup() {
    return () =>
      h(Avatar, {
        fallback: "AK",
        name: "Aki Kimura",
      });
  },
});

test("renders byte-identical fallback markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<span/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="avatar"/);
  assert.match(html, /data-state="fallback"/);
  assert.match(html, /data-status="none"/);
  assert.match(html, /data-image="missing"/);
  assert.match(html, /data-name="present"/);
  assert.match(html, /data-fallback="present"/);
  assert.match(html, /data-vize-ui="avatar-fallback"/);
  assert.match(html, /AK/);
  assert.doesNotMatch(html, /data-vize-ui="avatar-image"/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /aria-live=/);
  assert.doesNotMatch(html, /style=/);
});

test("renders server image markup with native image attributes", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "AvatarImageSsrProbe",
      setup() {
        return () =>
          h(Avatar, {
            alt: "Aki Kimura",
            crossOrigin: "anonymous",
            decoding: "async",
            fetchPriority: "low",
            loading: "lazy",
            name: "Aki Kimura",
            referrerPolicy: "no-referrer",
            src: "/avatars/aki.png",
            status: "online",
          });
      },
    }),
  );

  assert.match(html, /^<span/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="avatar"/);
  assert.match(html, /data-state="image"/);
  assert.match(html, /data-status="online"/);
  assert.match(html, /data-image="present"/);
  assert.match(html, /data-name="present"/);
  assert.match(html, /data-fallback="missing"/);
  assert.match(html, /<img/);
  assert.match(html, /part="image"/);
  assert.match(html, /data-vize-ui="avatar-image"/);
  assert.match(html, /src="\/avatars\/aki\.png"/);
  assert.match(html, /alt="Aki Kimura"/);
  assert.match(html, /loading="lazy"/);
  assert.match(html, /decoding="async"/);
  assert.match(html, /fetchpriority="low"/);
  assert.match(html, /crossorigin="anonymous"/);
  assert.match(html, /referrerpolicy="no-referrer"/);
  assert.doesNotMatch(html, /data-vize-ui="avatar-fallback"/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /aria-live=/);
  assert.doesNotMatch(html, /style=/);
});
