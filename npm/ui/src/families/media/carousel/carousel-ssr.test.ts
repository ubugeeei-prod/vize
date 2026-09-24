import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import CarouselAutoplayToggle from "./carousel-autoplay-toggle.vue";
import CarouselIndicator from "./carousel-indicator.vue";
import CarouselIndicatorGroup from "./carousel-indicator-group.vue";
import CarouselNext from "./carousel-next.vue";
import CarouselPrevious from "./carousel-previous.vue";
import CarouselRoot from "./carousel-root.vue";
import CarouselSlide from "./carousel-slide.vue";
import CarouselViewport from "./carousel-viewport.vue";

const SsrProbe = defineComponent({
  name: "CarouselSsrProbe",
  setup: () => () =>
    h(
      CarouselRoot,
      { slideCount: 3, defaultValue: 1, autoplay: true, ariaLabel: "Highlights" },
      () => [
        h(CarouselAutoplayToggle, null, () => "Stop rotation"),
        h(CarouselPrevious, null, () => "Previous"),
        h(CarouselNext, null, () => "Next"),
        h(CarouselViewport, null, () =>
          [0, 1, 2].map((index) => h(CarouselSlide, { index }, () => `Slide ${index + 1}`)),
        ),
        h(CarouselIndicatorGroup, { ariaLabel: "Choose slide" }, () =>
          [0, 1, 2].map((index) => h(CarouselIndicator, { index })),
        ),
      ],
    ),
});

test("renders byte-identical carousel markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  const html = left;

  assert.match(html, /^<section id="vize-v-\d+-carousel" aria-roledescription="carousel"/);
  assert.match(html, /aria-label="Highlights"/);
  assert.match(html, /data-autoplay="playing"/);
  assert.match(html, /data-index="1"/);
  assert.match(html, /id="vize-v-\d+-carousel-viewport" tabindex="0" aria-live="off"/);
  assert.match(html, /aria-label="2 of 3"/);
  assert.match(html, /id="vize-v-\d+-carousel-slide-1"[^>]*data-state="active"/);
  assert.match(html, /role="tab"[^>]*tabindex="0"[^>]*aria-label="Slide 2" aria-selected="true"/);
  assert.doesNotMatch(html, /inert/);
});

test("hydrates carousel markup without warnings or node replacement", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverSlides = [...host.querySelectorAll('[data-vize-ui="carousel-slide"]')];
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
    const slides = [...host.querySelectorAll('[data-vize-ui="carousel-slide"]')];
    assert.ok(slides.every((slide, index) => slide === serverSlides[index]));
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
