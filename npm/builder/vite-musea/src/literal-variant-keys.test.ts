import assert from "node:assert/strict";
import test from "node:test";
import { runInThisContext } from "node:vm";
import { generateGalleryGlobalsScript } from "./gallery/globals.ts";

void test("ordinary static preview script bytes stay unchanged", () => {
  const script = generateGalleryGlobalsScript({
    basePath: "/__musea__",
    staticPreviews: {
      "/src/Controls.art.vue": {
        Default: "/preview/default.html",
        "Custom theme": "/preview/custom.html",
      },
    },
  });
  assert.equal(
    script,
    'window.__MUSEA_BASE_PATH__="/__musea__";window.__MUSEA_STATIC__=true;' +
      'window.__MUSEA_STATIC_PREVIEWS__={"/src/Controls.art.vue":{"Default":"/preview/default.html","Custom theme":"/preview/custom.html"}};',
  );
});

void test("literal preview keys survive safe embedding and inherited URLs cannot be returned", async () => {
  const authored = JSON.parse(
    '{"__proto__":"/preview/proto.html","constructor":"/preview/constructor.html","hasOwnProperty":"/preview/own.html","quote\\\"</script>&":"/preview/quote.html"}',
  );
  const windowData: Record<string, unknown> = {};
  const script = generateGalleryGlobalsScript({
    basePath: "/__musea__",
    staticPreviews: { "/src/Keys.art.vue": authored },
  });
  assert.equal(script.includes("</script>"), false);
  const apply = runInThisContext(`(window) => {${script}}`) as (
    window: Record<string, unknown>,
  ) => void;
  apply(windowData);
  const previews = windowData.__MUSEA_STATIC_PREVIEWS__ as Record<string, Record<string, string>>;
  const urls = previews["/src/Keys.art.vue"];
  assert.deepEqual(Object.keys(urls), Object.keys(authored));
  assert.equal(Object.hasOwn(urls, "__proto__"), true);
  assert.equal(Object.getPrototypeOf(urls), Object.prototype);
  assert.deepEqual(urls, authored);

  const previous = Reflect.get(globalThis, "window");
  Reflect.set(globalThis, "window", windowData);
  try {
    const api = await import("../gallery/staticApi.ts");
    for (const name of Object.keys(authored)) {
      assert.equal(api.getStaticPreviewUrl("/src/Keys.art.vue", name), authored[name]);
    }
    assert.equal(api.getStaticPreviewUrl("/src/Keys.art.vue", "toString"), undefined);
    assert.equal(api.getStaticPreviewUrl("toString", "constructor"), undefined);
    assert.equal(api.getStaticPreviewUrl("__proto__", "hasOwnProperty"), undefined);
    assert.equal(api.getStaticPreviewUrl("/src/Missing.art.vue", "__proto__"), undefined);
  } finally {
    if (previous === undefined) Reflect.deleteProperty(globalThis, "window");
    else Reflect.set(globalThis, "window", previous);
  }
});
