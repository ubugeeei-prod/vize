import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Rating from "./rating.vue";

const SsrProbe = defineComponent({
  name: "RatingSsrProbe",
  setup: () => () =>
    h(
      Rating,
      {
        ariaDescribedby: "score-help",
        ariaLabel: "Movie score",
        clearable: true,
        defaultValue: 4,
        dir: "rtl",
        id: "movie-rating",
        name: "score",
        required: true,
      },
      {
        item: ({ value }: { value: number }) => String(value),
        default: ({ value }: { value: number | null }) => `Selected ${value ?? "none"}`,
      },
    ),
});

test("renders byte-identical rating markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<span/);
  assert.match(html, /id="movie-rating"/);
  assert.match(html, /role="radiogroup"/);
  assert.match(html, /dir="rtl"/);
  assert.match(html, /aria-label="Movie score"/);
  assert.match(html, /aria-describedby="score-help"/);
  assert.match(html, /aria-required="true"/);
  assert.match(html, /data-vize-ui="rating"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-state="selected"/);
  assert.match(html, /data-value="4"/);
  assert.match(html, /data-clearable="true"/);
  assert.match(html, /--vize-rating-percent:80%/);
  assert.match(html, /data-vize-ui="rating-control"/);
  assert.match(html, /type="radio"/);
  assert.match(html, /name="score"/);
  assert.match(html, /value="4"/);
  assert.match(html, /checked/);
  assert.match(html, /Selected 4/);
  assert.doesNotMatch(html, /function/);
});

test("hydrates generated rating ids without changing the server contract", async () => {
  const GeneratedIdProbe = defineComponent({
    name: "RatingGeneratedIdProbe",
    setup: () => () =>
      h(Rating, {
        ariaLabel: "Movie score",
        defaultValue: 2,
        name: "score",
      }),
  });
  const serverHtml = await renderToString(createSSRApp(GeneratedIdProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverInputs = [
    ...host.querySelectorAll<HTMLInputElement>("[data-vize-ui='rating-control']"),
  ];
  assert.ok(serverRoot);
  assert.equal(serverInputs.length, 5);
  const serverIds = serverInputs.map((input) => input.id);
  assert.match(serverIds[0] ?? "", /^vize-v-\d+-rating-item-1$/);

  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(GeneratedIdProbe);
  let mounted = false;

  try {
    app.mount(host);
    mounted = true;
    const hydratedInputs = [
      ...host.querySelectorAll<HTMLInputElement>("[data-vize-ui='rating-control']"),
    ];
    assert.ok(host.firstElementChild === serverRoot);
    assert.deepEqual(
      hydratedInputs.map((input) => input.id),
      serverIds,
    );
    assert.equal(hydratedInputs[1]?.checked, true);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});

test("renders repaired invalid server markup without unsafe numeric attributes", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "RatingInvalidSsrProbe",
      setup: () => () =>
        h(Rating, {
          ariaInvalid: true,
          ariaLabel: "Movie score",
          defaultValue: Number.NaN,
          max: 0,
          min: Number.NEGATIVE_INFINITY,
          count: 0,
        }),
    }),
  );

  assert.match(html, /^<span/);
  assert.match(html, /data-state="invalid"/);
  assert.match(html, /data-invalid="true"/);
  assert.match(html, /data-min="1"/);
  assert.match(html, /data-max="1"/);
  assert.match(html, /data-count="1"/);
  assert.doesNotMatch(html, /NaN|Infinity/);
});
