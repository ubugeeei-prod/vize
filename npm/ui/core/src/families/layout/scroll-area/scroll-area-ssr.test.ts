import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import ScrollArea from "./scroll-area.vue";

const SsrProbe = defineComponent({
  name: "ScrollAreaSsrProbe",
  setup() {
    return () =>
      h(
        ScrollArea,
        {
          ariaDescribedby: "release-scroll-help",
          ariaLabelledby: "release-scroll-title",
          as: "section",
          blockSize: 280,
          dir: "rtl",
          focusable: true,
          inlineSize: "min(100%, 36rem)",
          maxBlockSize: "65vh",
          orientation: "both",
          overscrollBehavior: "contain",
          scrollBehavior: "smooth",
          scrollbarGutter: "stable both-edges",
          scrollbarWidth: "thin",
        },
        {
          default: () => [
            h("h2", { id: "release-scroll-title" }, "Release log"),
            h("p", { id: "release-scroll-help" }, "Scrollable updates"),
          ],
        },
      );
  },
});

test("renders byte-identical labelled scroll area markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<section/);
  assert.match(html, /dir="rtl"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="scroll-area"/);
  assert.match(html, /data-state="scrollable"/);
  assert.match(html, /data-orientation="both"/);
  assert.match(html, /data-dir="rtl"/);
  assert.match(html, /data-focusable="true"/);
  assert.match(html, /data-overscroll-behavior="contain"/);
  assert.match(html, /data-scroll-behavior="smooth"/);
  assert.match(html, /data-scrollbar-gutter="stable both-edges"/);
  assert.match(html, /data-scrollbar-width="thin"/);
  assert.match(html, /--vize-ui-scroll-area-block-size:280px/);
  assert.match(html, /--vize-ui-scroll-area-inline-size:min\(100%, 36rem\)/);
  assert.match(html, /--vize-ui-scroll-area-max-block-size:65vh/);
  assert.match(html, /--vize-ui-scroll-area-max-inline-size:none/);
  assert.match(html, /--vize-ui-scroll-area-overflow-x:auto/);
  assert.match(html, /--vize-ui-scroll-area-overflow-y:auto/);
  assert.match(html, /--vize-ui-scroll-area-overscroll-behavior:contain/);
  assert.match(html, /--vize-ui-scroll-area-scroll-behavior:smooth/);
  assert.match(html, /--vize-ui-scroll-area-scrollbar-gutter:stable both-edges/);
  assert.match(html, /--vize-ui-scroll-area-scrollbar-width:thin/);
  assert.match(html, /<div[^>]*part="viewport"/);
  assert.match(html, /data-vize-ui="scroll-area-viewport"/);
  assert.match(html, /role="region"/);
  assert.match(html, /tabindex="0"/);
  assert.match(html, /aria-labelledby="release-scroll-title"/);
  assert.match(html, /aria-describedby="release-scroll-help"/);
  assert.match(html, /data-overflow-x="auto"/);
  assert.match(html, /data-overflow-y="auto"/);
  assert.match(
    html,
    /<h2 id="release-scroll-title">Release log<\/h2><p id="release-scroll-help">Scrollable updates<\/p>/,
  );
  assert.doesNotMatch(html, /id="vize/);
  assert.doesNotMatch(html, /aria-hidden=|aria-live=/);
});

test("omits optional ARIA and focus attributes from default SSR markup", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "ScrollAreaDefaultSsrProbe",
      setup() {
        return () =>
          h(ScrollArea, null, {
            default: () => [h("p", "Native scroll container")],
          });
      },
    }),
  );

  assert.match(html, /^<div/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="scroll-area"/);
  assert.match(html, /data-state="scrollable"/);
  assert.match(html, /data-orientation="vertical"/);
  assert.match(html, /data-dir="ltr"/);
  assert.match(html, /data-focusable="false"/);
  assert.match(html, /--vize-ui-scroll-area-overflow-x:hidden/);
  assert.match(html, /--vize-ui-scroll-area-overflow-y:auto/);
  assert.match(html, /<div[^>]*data-vize-ui="scroll-area-viewport"/);
  assert.match(html, /<p>Native scroll container<\/p>/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
  assert.doesNotMatch(html, /aria-label=/);
  assert.doesNotMatch(html, /aria-labelledby=/);
  assert.doesNotMatch(html, /aria-describedby=/);
  assert.doesNotMatch(html, /id=/);
});
