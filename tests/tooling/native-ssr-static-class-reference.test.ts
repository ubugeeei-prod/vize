import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { checkMap, directModule, hash, renderModule } from "./support/native-sfc-ssr-reference.ts";

const pack = JSON.parse(
  fs.readFileSync(
    new URL("../../davinci/vize_l4/tests/fixtures/native-ssr-vue-3.5.35.json", import.meta.url),
    "utf8",
  ),
);
const classIds = [
  "class-root",
  "class-root-props",
  "class-nested",
  "class-fragment",
  "class-root-bare",
  "class-root-empty",
  "class-nested-bare",
  "class-nested-empty",
  "class-entities",
  "class-unquoted",
  "class-unicode-trim",
  "class-unicode-inner",
  "class-void",
  "class-comment-root",
];
function prepared(code: string) {
  const imports = code.split("\n").filter((line) => line.startsWith("import "));
  const declaration = code
    .slice(imports.join("\n").length)
    .trimStart()
    .replace(/^export /, "");
  return [
    ...imports,
    "const __sfc__ = {}",
    ";",
    declaration,
    "__sfc__.ssrRender = ssrRender",
    "export default __sfc__",
    "",
  ].join("\n");
}

test("static class rows append to byte-identical original eleven SSR references", () => {
  assert.equal(
    hash(JSON.stringify(pack.fixtures.slice(0, 11))),
    "e27f219e6f39898bf16f05ee10edef95c65231af193b6e557c5ba102665267a4",
  );
  assert.deepEqual(
    pack.fixtures.slice(11).map((row: any) => row.id),
    classIds,
  );
});

const capturePath = process.env.VIZE_L4_SSR_NATIVE_CAPTURE;
const runtimeMode = process.env.NODE_ENV === "production" ? "production" : "development";
test(
  "source-built static class components retain every original map anchor and actual SSR fallthrough",
  { skip: !capturePath && process.env.VIZE_L4_SSR_REQUIRE_NATIVE !== "1" },
  async () => {
    assert(capturePath, "hosted static class SSR requires the complete fresh native capture");
    const captures = JSON.parse(fs.readFileSync(capturePath, "utf8"));
    assert.deepEqual(
      captures.map((row: any) => row.id),
      pack.fixtures.map((row: any) => row.id),
    );
    const runtime = [];
    for (const [index, fixture] of pack.fixtures.entries()) {
      const capture = captures[index];
      assert.equal(capture.source, fixture.source);
      assert.equal(capture.code, fixture.code);
      assert.equal(capture.sfcCode, prepared(fixture.code));
      const component = {
        source: capture.source,
        filename: capture.filename,
        code: capture.sfcCode,
        map: capture.sfcMap,
        links: capture.sfcLinks,
      };
      checkMap(capture);
      checkMap(component);
      const native = await renderModule(capture.sfcCode);
      const official = await renderModule(prepared(fixture.referenceCode));
      assert.deepEqual(native, official);
      const direct = await directModule(capture.sfcCode);
      assert.deepEqual(
        direct,
        native.map((row) => row.html),
      );
      if (fixture.id === "class-unicode-trim") {
        assert.equal(native[0].html, '<div class="雪🌸\u0085"></div>');
      }
      if (fixture.id === "class-nested-bare") {
        assert.equal(native[0].html, "<div><span class></span></div>");
      }
      if (fixture.id === "class-root") {
        assert.equal(native[0].html, '<div class="a b"></div>');
        assert.equal(native[1].html.includes('class="a b root active"'), true);
        const missing = { ...component, links: [] };
        assert.throws(() => checkMap(missing), /complete start anchor/);
      }
      runtime.push({
        id: capture.id,
        sourceSha256: hash(capture.source),
        codeSha256: hash(capture.sfcCode),
        mapSha256: hash(JSON.stringify(capture.sfcMap)),
        linksSha256: hash(JSON.stringify(capture.sfcLinks)),
        referenceCodeSha256: fixture.referenceCodeSha256,
        native,
        direct,
        official,
      });
    }
    const runtimePath = process.env.VIZE_L4_SSR_RUNTIME_CAPTURE;
    if (runtimePath) {
      fs.writeFileSync(
        `${runtimePath}.static-class.${runtimeMode}.json`,
        JSON.stringify(
          {
            modules: captures.length,
            classModules: classIds.length,
            runtimeMode,
            nativeWholeExecutions: captures.length * 3,
            nativeDirectExecutions: captures.length * 3,
            officialWholeExecutions: captures.length * 3,
            completeUpstreamMapParity: false,
            selectedSfcClassAdmission: false,
            runtime,
          },
          null,
          2,
        ) + "\n",
      );
    }
  },
);
