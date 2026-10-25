import assert from "node:assert/strict";
import test from "node:test";
import { performance } from "node:perf_hooks";
import { restorePaletteState } from "../gallery/composables/paletteState.ts";
import {
  createPropsPermalink,
  readPropsPermalink,
  MUSEA_PROPS_QUERY,
  MAX_PROPS_PERMALINK_LENGTH,
  type PropsPermalinkState,
} from "../gallery/utils/propsPermalink.ts";

const artPath = "components/日本語 🎨.art.vue";
const state: PropsPermalinkState = {
  palette: {
    version: 1,
    values: JSON.parse(
      '{"label":"こんにちは 🎨 & = ? # + %","config":{"nested":[false,null,42]},"custom":"share","constructor":"literal","__proto__":{"safe":true},"tone":"danger"}',
    ),
    customProps: [{ name: "custom", control: "text", default_value: "authored" }],
    deletedPaletteProps: ["tone"],
  },
  slots: { default: "<script>literal text</script> & 🎨", footer: "{{ neverEvaluate() }}" },
};

function encodedState(input: unknown = { version: 1, artPath, ...state }): string {
  return JSON.stringify(input);
}

void test("props permalinks preserve base, fragment, query, Unicode and complete palette state", () => {
  const link = new URL(
    createPropsPermalink(
      "https://example.test/docs/gallery/component/probe?theme=dark&theme=light#variant-a",
      artPath,
      state,
    ),
  );
  assert.equal(link.pathname, "/docs/gallery/component/probe");
  assert.equal(link.hash, "#variant-a");
  assert.deepEqual(link.searchParams.getAll("theme"), ["dark", "light"]);
  const parsed = readPropsPermalink(link.searchParams.get(MUSEA_PROPS_QUERY), artPath);
  assert.equal(parsed.error, null);
  assert.deepEqual(parsed.state, state);
  assert.equal(Object.getPrototypeOf(parsed.state!.palette.values), Object.prototype);
  assert.equal(Object.hasOwn(parsed.state!.palette.values, "__proto__"), true);
  const controls = Object.keys(state.palette.values)
    .filter((name) => name !== "custom")
    .map((name) => ({
      name,
      control: "text",
      default_value: "default",
      required: false,
      options: [],
    }));
  const restored = restorePaletteState(controls, parsed.state!.palette);
  assert.deepEqual(restored.values, state.palette.values);
  assert.deepEqual(restored.customProps, state.palette.customProps);
  assert.deepEqual([...restored.deletedPaletteProps], ["tone"]);
});

void test("props URLs are deterministic across object, custom prop and deletion insertion orders", () => {
  const first: PropsPermalinkState = {
    ...state,
    palette: {
      ...state.palette,
      customProps: [
        ...state.palette.customProps,
        { name: "second", control: "number", default_value: 2 },
      ],
      deletedPaletteProps: ["tone", "size"],
    },
  };
  const second = {
    palette: {
      ...first.palette,
      values: Object.fromEntries(Object.entries(first.palette.values).reverse()),
      customProps: [...first.palette.customProps].reverse(),
      deletedPaletteProps: ["size", "tone", "tone"],
    },
    slots: Object.fromEntries(Object.entries(first.slots).reverse()),
  };
  assert.equal(
    createPropsPermalink("https://example.test/", artPath, first),
    createPropsPermalink("https://example.test/", artPath, second),
  );
});

void test("malformed, mismatched, repeated, non-JSON and executable states are refused completely", () => {
  for (const invalid of [
    null,
    [encodedState()],
    "{",
    encodedState({ version: 2, artPath, ...state }),
    encodedState({ version: 1, artPath: "other.art.vue", ...state }),
    encodedState({ version: 1, artPath, ...state, slots: { default: 42 } }),
    encodedState({
      version: 1,
      artPath,
      ...state,
      palette: { ...state.palette, customProps: [null] },
    }),
    encodedState({
      version: 1,
      artPath,
      ...state,
      palette: { ...state.palette, deletedPaletteProps: [false] },
    }),
    encodedState({
      version: 1,
      artPath,
      ...state,
      palette: { ...state.palette, values: { innerHTML: "<img src=x onerror=alert(1)>" } },
    }),
    encodedState({
      version: 1,
      artPath,
      ...state,
      palette: { ...state.palette, values: { onClick: "alert(1)" } },
    }),
    ...["onclick", "on:click", "srcdoc", "innerhtml", "outerHTML"].map((name) =>
      encodedState({
        version: 1,
        artPath,
        ...state,
        palette: { ...state.palette, values: { [name]: "untrusted" } },
      }),
    ),
    ...[
      [{ name: " custom ", control: "text", default_value: "trimmed" }],
      [{ name: "custom", control: "unknown", default_value: "invalid" }],
      [state.palette.customProps[0], state.palette.customProps[0]],
    ].map((customProps) =>
      encodedState({ version: 1, artPath, ...state, palette: { ...state.palette, customProps } }),
    ),
  ]) {
    const parsed = readPropsPermalink(invalid, artPath);
    assert.equal(parsed.state, null, String(invalid));
    assert.ok(parsed.error);
  }
  for (const invalid of [undefined, Infinity, NaN, () => 1, 1n, new Date()]) {
    assert.throws(
      () =>
        createPropsPermalink("https://example.test/", artPath, {
          ...state,
          palette: { ...state.palette, values: { invalid } },
        }),
      /JSON/,
    );
  }
});

void test("permalink parsing and encoding enforce payload, depth and node budgets", () => {
  const long = { ...state, slots: { default: "あ".repeat(MAX_PROPS_PERMALINK_LENGTH) } };
  assert.throws(() => createPropsPermalink("https://example.test/", artPath, long), /too large/);
  assert.equal(
    readPropsPermalink(encodedState({ version: 1, artPath, ...long }), artPath).state,
    null,
  );
  const deep = Array.from({ length: 20 }).reduce<unknown>((child) => ({ child }), null);
  assert.throws(
    () =>
      createPropsPermalink("https://example.test/", artPath, {
        ...state,
        palette: { ...state.palette, values: { deep } },
      }),
    /too complex/,
  );
  const many = {
    ...state,
    palette: { ...state.palette, values: { list: Array.from({ length: 4_100 }, () => 0) } },
  };
  assert.throws(() => createPropsPermalink("https://example.test/", artPath, many), /too complex/);
  assert.equal(
    readPropsPermalink(encodedState({ version: 1, artPath, ...many }), artPath).state,
    null,
  );
});

void test("representative permalink encoding and decoding reports source timing", (context) => {
  const start = performance.now();
  for (let index = 0; index < 1_000; index++) {
    const link = new URL(createPropsPermalink("https://example.test/", artPath, state));
    assert.deepEqual(
      readPropsPermalink(link.searchParams.get(MUSEA_PROPS_QUERY), artPath).state,
      state,
    );
  }
  context.diagnostic(
    JSON.stringify({
      iterations: 1_000,
      elapsedMs: performance.now() - start,
      encodedLength: createPropsPermalink("https://example.test/", artPath, state).length,
    }),
  );
});
