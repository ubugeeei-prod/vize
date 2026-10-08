import assert from "node:assert/strict";
import { test } from "node:test";
import { restoreNuxtClientManifestSourceIds } from "./client-manifest.ts";

void test("original SFC manifest graph keeps every payload and rewires only owned source edges", () => {
  const manifest = {
    "entry.js": {
      file: "entry.hash.js",
      src: "entry.js",
      isEntry: true,
      imports: ["shared.js", "components/ReportTable.vue?vue&vize"],
      dynamicImports: ["pages/index.vue?vue&vize", "pages/reports.vue?vue&vize", "foreign.vue?raw"],
    },
    "pages/index.vue?vue&vize": { file: "index.hash.js", src: "pages/index.vue?vue&vize" },
    "pages/reports.vue?vue&vize": {
      file: "reports.hash.js",
      src: "pages/reports.vue?vue&vize",
      imports: ["components/ReportTable.vue?vue&vize", "shared.js"],
      css: ["reports.css"],
      assets: ["chart.svg"],
      name: "reports",
      isDynamicEntry: true,
    },
    "components/ReportTable.vue?vue&vize": {
      file: "table.hash.js",
      src: "components/ReportTable.vue?vue&vize",
      css: ["table.css"],
    },
    "shared.js": { file: "shared.hash.js", dynamicImports: ["pages/reports.vue?vue&vize"] },
    "foreign.vue?raw": { file: "raw.hash.js", src: "foreign.vue?raw" },
    "reports.css": { file: "reports.css", resourceType: "style", prefetch: true },
    "table.css": { file: "table.css", resourceType: "style", prefetch: true },
    "chart.svg": { file: "chart.svg", resourceType: "image", prefetch: true },
  };
  const originalTable = manifest["components/ReportTable.vue?vue&vize"];
  restoreNuxtClientManifestSourceIds(manifest);
  assert.deepEqual(manifest, {
    "entry.js": {
      file: "entry.hash.js",
      src: "entry.js",
      isEntry: true,
      imports: ["shared.js", "components/ReportTable.vue"],
      dynamicImports: ["pages/index.vue", "pages/reports.vue", "foreign.vue?raw"],
    },
    "pages/index.vue": { file: "index.hash.js", src: "pages/index.vue" },
    "pages/reports.vue": {
      file: "reports.hash.js",
      src: "pages/reports.vue",
      imports: ["components/ReportTable.vue", "shared.js"],
      css: ["reports.css"],
      assets: ["chart.svg"],
      name: "reports",
      isDynamicEntry: true,
    },
    "components/ReportTable.vue": {
      file: "table.hash.js",
      src: "components/ReportTable.vue",
      css: ["table.css"],
    },
    "shared.js": { file: "shared.hash.js", dynamicImports: ["pages/reports.vue"] },
    "foreign.vue?raw": { file: "raw.hash.js", src: "foreign.vue?raw" },
    "reports.css": { file: "reports.css", resourceType: "style", prefetch: true },
    "table.css": { file: "table.css", resourceType: "style", prefetch: true },
    "chart.svg": { file: "chart.svg", resourceType: "image", prefetch: true },
  });
  assert.equal(
    Object.getOwnPropertyDescriptor(manifest, "components/ReportTable.vue")?.value,
    originalTable,
  );
  const once = JSON.stringify(manifest);
  restoreNuxtClientManifestSourceIds(manifest);
  assert.equal(JSON.stringify(manifest), once);
});

void test("foreign requests, unowned src and absent graph rows remain byte exact", () => {
  const ids = [
    "Page.vue",
    "Page.vue?vue",
    "Page.vue?raw",
    "Page.vue?vue&vize&type=style",
    "Page.vue?vue&vize&raw",
    "Page.vue?vue&vize&worker",
    "Page.vue?vue&vize&lang.ts",
    "Page.vue?vue&vize-ssr",
    "Page.vue.ts?vue&vize",
    "Page.vue?notvue&vize",
    "Page.vue?vue&vize=yes",
    "Page.vue?vue&vize&",
    "Page.vue?raw&fake=Nested.vue?vue&vize",
    "Page.vue?vue&vize&url=Other.vue?vue&vize",
    "Page.ts?vue&vize",
  ];
  const manifest = Object.fromEntries(ids.map((id) => [id, { src: id, file: id + ".js" }]));
  manifest["Unowned.vue?vue&vize"] = { src: "different.vue", file: "unowned.js" };
  manifest["entry.js"] = { src: "entry.js", file: "entry.js" };
  Object.assign(manifest["entry.js"], { imports: ["Absent.vue?vue&vize"], dynamicImports: ids });
  const original = JSON.stringify(manifest);
  restoreNuxtClientManifestSourceIds(manifest);
  assert.equal(JSON.stringify(manifest), original);
});

void test("source collisions fail before mutating any original row or dependency", () => {
  const manifest = {
    "first.vue?vue&vize": { src: "first.vue?vue&vize", file: "first.js" },
    "entry.js": { file: "entry.js", dynamicImports: ["first.vue?vue&vize"] },
    "Page.vue": { src: "Page.vue", file: "stock.js" },
    "Page.vue?vue&vize": { src: "Page.vue?vue&vize", file: "vize.js" },
  };
  const original = JSON.stringify(manifest);
  assert.throws(() => restoreNuxtClientManifestSourceIds(manifest), {
    message: "@vizejs/nuxt: conflicting client manifest source Page.vue",
  });
  assert.equal(JSON.stringify(manifest), original);
});
