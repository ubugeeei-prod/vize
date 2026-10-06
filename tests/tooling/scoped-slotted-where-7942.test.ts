import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import { compileCss, compileSfc } from "../../npm/native/index.js";
import { scopeCssForPipeline } from "../../npm/builder/vite/src/utils/css.ts";

const root = new URL(
  "../_fixtures/differential/compiler/scoped-slotted-where-7942/",
  import.meta.url,
);
const read = (name: string) => fs.readFileSync(new URL(name, root), "utf8");
type Case = {
  id: string;
  source: string;
  pipeline: string;
  sha256: string;
  pipelineSha256: string;
};
const pack = JSON.parse(read("cases.json")) as {
  schema: string;
  scopeId: string;
  cases: Case[];
  originalSfcSha256: string;
  sourceIssueBodySha256: string;
  originalPipelineSha256: string;
};
const digest = (text: string) => createHash("sha256").update(text).digest("hex");

await test("all eight full original and anchor CSS controls cross the real native and Vite boundaries", () => {
  assert.equal(pack.schema, "vize.scoped-slotted-where.originals");
  assert.equal(pack.cases.length, 8);
  assert.equal(new Set(pack.cases.map((entry) => entry.id)).size, 8);
  for (const entry of pack.cases) {
    const source = read(entry.source);
    const expected = read(entry.pipeline);
    assert.equal(digest(expected), entry.pipelineSha256);
    assert.equal(digest(source), entry.sha256);
    assert.equal(scopeCssForPipeline(source, pack.scopeId), expected);
    for (const minify of [false, true]) {
      const reference = compileCss(expected, { minify });
      assert.deepEqual(reference.errors, []);
      assert.deepEqual(reference.warnings, []);
      const actual = compileCss(source, { scoped: true, scopeId: pack.scopeId, minify });
      assert.deepEqual(actual, reference, `${entry.id} minify=${minify}`);
    }
  }
});

await test("original two-rule whole SFC keeps full public results and additive authored maps", () => {
  const source = read("App.vue");
  assert.equal(digest(source), pack.originalSfcSha256);
  assert.equal(digest(read("App.pipeline.css")), pack.originalPipelineSha256);
  const originalBody = read("issue.md");
  assert.equal(digest(originalBody), pack.sourceIssueBodySha256);
  assert.equal(originalBody.split("```vue\n")[1].split("```")[0], source);
  for (const inlineTemplate of [false, true]) {
    for (const styleTrim of [false, true]) {
      const options = { filename: "App.vue", scopeId: pack.scopeId, inlineTemplate, styleTrim };
      const plain = compileSfc(source, options);
      assert.deepEqual(plain.errors, []);
      assert.deepEqual(plain.warnings, []);
      assert.equal(
        plain.css,
        styleTrim ? read("App.pipeline.css").trim() : read("App.pipeline.css"),
      );
      assert.equal(plain.hasScoped, true);
      assert.equal(plain.styles.length, 1);
      assert.equal(plain.styles[0].content, source.split("<style scoped>")[1].split("</style>")[0]);
      assert.deepEqual(compileSfc(source, options), plain);
      const mapped = compileSfc(source, { ...options, sourceMap: true });
      assert.equal(typeof mapped.map, "string");
      const map = JSON.parse(mapped.map!);
      assert.equal(map.version, 3);
      assert.deepEqual(map.sources, ["App.vue"]);
      assert.deepEqual(map.sourcesContent, [source]);
      assert.deepEqual({ ...mapped, map: plain.map }, plain);
    }
  }
});
