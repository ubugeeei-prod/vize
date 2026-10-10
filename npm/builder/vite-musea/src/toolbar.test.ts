import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { runInNewContext } from "node:vm";
import { normalizeToolbar, parseToolbarGlobals, resolveToolbarGlobals } from "./toolbar.ts";
import { generatePreviewGlobals } from "./preview/globals.ts";
import { generateGalleryGlobalsScript } from "./gallery/globals.ts";
import { generatePreviewModule, generatePreviewModuleWithProps } from "./preview/index.ts";
import type { ArtFileInfo } from "./types/art.ts";

const toolbar = normalizeToolbar([
  {
    id: "brand",
    title: "Brand",
    type: "select",
    options: ["default", "brand-a"],
    default: "default",
  },
  {
    id: "rtl",
    title: "Direction",
    type: "toggle",
    options: [
      { value: false, label: "LTR" },
      { value: true, label: "RTL" },
    ],
    default: false,
  },
]);

void test("toolbar validates finite options, unique ids and exact defaults once", () => {
  assert.deepEqual(toolbar[0].options, [
    { value: "default", label: "default" },
    { value: "brand-a", label: "brand-a" },
  ]);
  for (const invalid of [
    [{ ...toolbar[0], id: "__proto__" }],
    [toolbar[0], toolbar[0]],
    [{ ...toolbar[0], default: "missing" }],
    [{ ...toolbar[0], type: "toggle" as const, options: ["one"] }],
    [{ ...toolbar[0], options: [{ value: Number.NaN, label: "NaN" }] }],
    [{ ...toolbar[0], options: ["same", "same"] }],
  ])
    assert.throws(() => normalizeToolbar(invalid), /Invalid toolbar control/);
  assert.deepEqual(normalizeToolbar(), []);
});

void test("untrusted URL/storage values retain only declared primitive options", () => {
  assert.deepEqual(resolveToolbarGlobals(toolbar, { brand: "brand-a", rtl: true, unknown: 42 }), {
    brand: "brand-a",
    rtl: true,
  });
  for (const invalid of [
    null,
    [],
    "bad",
    { brand: {}, rtl: "true" },
    Object.create({ brand: "brand-a" }),
  ]) {
    assert.deepEqual(resolveToolbarGlobals(toolbar, invalid), { brand: "default", rtl: false });
  }
  assert.equal(parseToolbarGlobals("{"), undefined);
  assert.equal(parseToolbarGlobals(null), undefined);
});

void test("preview initializes before setup, accepts only parent commands and cleans up", () => {
  const handlers = new Map<string, (event: unknown) => void>();
  const parent = {};
  const ref = { value: {} };
  const window = {
    location: {
      origin: "http://musea.test",
      href:
        "http://musea.test/preview?museaGlobals=" +
        encodeURIComponent(JSON.stringify({ brand: "brand-a", rtl: true })),
    },
    parent,
    addEventListener: (type: string, handler: (event: unknown) => void) =>
      handlers.set(type, handler),
    removeEventListener: (type: string) => handlers.delete(type),
  };
  runInNewContext(generatePreviewGlobals(toolbar), {
    window,
    URL,
    Object,
    JSON,
    __museaCreateGlobalsRef: (value: unknown) => {
      ref.value = value as {};
      return ref;
    },
  });
  assert.deepEqual(ref.value, { brand: "brand-a", rtl: true });
  const payload = { brand: "default", rtl: false };
  const data = { type: "musea:set-globals", payload };
  const message = handlers.get("message")!;
  message({ source: parent, origin: "http://attacker.test", data });
  message({ source: {}, origin: window.location.origin, data });
  assert.deepEqual(ref.value, { brand: "brand-a", rtl: true });
  message({ source: parent, origin: window.location.origin, data });
  assert.deepEqual(ref.value, payload);
  const current = ref.value;
  message({ source: parent, origin: window.location.origin, data });
  assert.equal(ref.value, current, "identical values must not trigger watchers");
  handlers.get("pagehide")!({});
  assert.equal(handlers.has("message"), false);
});

void test("dev/static config safely serializes controls and preview snapshots retain original custody", () => {
  const dangerous = normalizeToolbar([
    { ...toolbar[0], title: "</script><script>alert(1)</script>" },
  ]);
  const window: Record<string, unknown> = {};
  const script = generateGalleryGlobalsScript({ basePath: "/__musea__", toolbar: dangerous });
  assert.equal(script.split("</script>").length, 1);
  runInNewContext(script, { window });
  assert.deepEqual(JSON.parse(JSON.stringify(window.__MUSEA_TOOLBAR__)), dangerous);
  assert.equal(generatePreviewGlobals([]), "");
  const art: ArtFileInfo = {
    path: "/tmp/Toolbar.art.vue",
    metadata: { title: "Toolbar", tags: [], status: "ready" },
    variants: [{ name: "default", template: "<div />", isDefault: true, skipVrt: false }],
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
  };
  for (const [name, code] of [
    ["default", generatePreviewModule(art, "Default", "default", [], "/setup.ts")],
    ["props", generatePreviewModuleWithProps(art, "Default", "default", {}, [], "/setup.ts")],
  ]) {
    const original = readFileSync(
      new URL(
        `../../../../tests/_fixtures/differential/musea/global-toolbar-${name}.js.txt`,
        import.meta.url,
      ),
    );
    assert.equal(
      createHash("sha256").update(original).digest("hex"),
      name === "default"
        ? "47b45a7c2659270ef91292d5c9f8fd6d0e03eaffbbcf434631ff28291c036dfc"
        : "1f58dcc03ceecfa048520564027c72b163a58aaec673eb17a89a5e0ba9d86ab0",
    );
    assert.equal(
      code,
      readFileSync(
        new URL(
          `../../../../tests/_fixtures/differential/musea/preview-prop-snapshots-${name}.js.txt`,
          import.meta.url,
        ),
        "utf8",
      ),
    );
  }
});
