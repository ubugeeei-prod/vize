import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Skeleton from "./skeleton.vue";

const SsrProbe = defineComponent({
  name: "SkeletonSsrProbe",
  setup() {
    return () =>
      h(
        Skeleton,
        {
          ariaLabel: "Loading profile",
          as: "section",
          blockSize: "2rem",
          inlineSize: "12rem",
        },
        {
          default: () => "Loading",
        },
      );
  },
});

test("renders byte-identical status skeleton markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /^<section/);
  assert.match(html, /role="status"/);
  assert.match(html, /aria-label="Loading profile"/);
  assert.match(html, /data-vize-ui="skeleton"/);
  assert.match(html, /data-state="loading"/);
  assert.match(html, /data-aria-state="status"/);
  assert.match(html, /--vize-ui-skeleton-block-size:2rem/);
  assert.match(html, /--vize-ui-skeleton-inline-size:12rem/);
  assert.match(html, /Loading/);
});

test("renders decorative server markup without status ARIA", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "SkeletonDecorativeSsrProbe",
      setup() {
        return () => h(Skeleton, { ariaLabel: "Ignored", ariaHidden: true, visible: false });
      },
    }),
  );

  assert.match(html, /^<div/);
  assert.match(html, /hidden/);
  assert.match(html, /aria-hidden="true"/);
  assert.match(html, /data-state="hidden"/);
  assert.match(html, /data-visible="false"/);
  assert.match(html, /data-aria-state="decorative"/);
  assert.doesNotMatch(html, /role="status"/);
  assert.doesNotMatch(html, /aria-label=/);
});
