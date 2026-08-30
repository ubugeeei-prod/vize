import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import BlockUI from "./block-ui.vue";

const SsrProbe = defineComponent({
  name: "BlockUISsrProbe",
  setup() {
    return () =>
      h(
        BlockUI,
        {
          announce: "polite",
          blocked: true,
          interaction: "inert",
          label: "Saving profile",
          reason: "saving",
        },
        {
          default: () => "Profile form",
        },
      );
  },
});

test("renders byte-identical blocked markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<section/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="block-ui"/);
  assert.match(html, /data-state="blocked"/);
  assert.match(html, /data-reason="saving"/);
  assert.match(html, /data-interaction="inert"/);
  assert.match(html, /data-announcement="polite"/);
  assert.match(html, /aria-busy="true"/);
  assert.match(html, /\sinert(?:=""|(?=[\s>]))/);
  assert.match(html, /role="status"/);
  assert.match(html, /aria-live="polite"/);
  assert.match(html, /aria-label="Saving profile"/);
  assert.match(html, /Profile form/);
  assert.doesNotMatch(html, /class=/);
  assert.doesNotMatch(html, /style=/);
  assert.doesNotMatch(html, /tabindex=/);
});

test("renders idle server markup without intrinsic busy, inert, or announcement attrs", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "BlockUIIdleSsrProbe",
      setup() {
        return () =>
          h(
            BlockUI,
            {
              "aria-describedby": "updates-help",
              "aria-label": "Updates",
              as: "article",
              role: "region",
            },
            {
              default: () => h("p", { id: "updates-help" }, "Updates are ready"),
            },
          );
      },
    }),
  );

  assert.match(html, /^<article/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="block-ui"/);
  assert.match(html, /data-state="idle"/);
  assert.match(html, /data-reason="loading"/);
  assert.match(html, /data-interaction="none"/);
  assert.match(html, /data-announcement="off"/);
  assert.match(html, /role="region"/);
  assert.match(html, /aria-label="Updates"/);
  assert.match(html, /aria-describedby="updates-help"/);
  assert.match(html, /Updates are ready/);
  assert.doesNotMatch(html, /aria-busy=/);
  assert.doesNotMatch(html, /\sinert(?:=""|(?=[\s>]))/);
  assert.doesNotMatch(html, /aria-live=/);
});
