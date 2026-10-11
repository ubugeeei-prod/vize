import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
import { compileFile } from "./compiler.ts";
import { generateOutputWithMap } from "./utils/index.ts";

const fixture = new URL(
  "../../../../tests/_fixtures/differential/compiler/vuefes-heading-cascade/",
  import.meta.url,
);
const corpus: { files: Record<string, { bytes: number; sha256: string; gitBlob: string }> } =
  JSON.parse(fs.readFileSync(new URL("corpus.json", fixture), "utf8"));
for (const [name, pin] of Object.entries(corpus.files)) {
  const source = fs.readFileSync(new URL(name, fixture));
  assert.equal(source.length, pin.bytes);
  assert.equal(createHash("sha256").update(source).digest("hex"), pin.sha256);
  assert.equal(
    createHash("sha1").update(`blob ${source.length}\0`).update(source).digest("hex"),
    pin.gitBlob,
  );
}
const original = fs.readFileSync(new URL("VFHeading.vue.txt", fixture), "utf8");
const controls = [
  original,
  "<template><p>plain style</p></template><style>p { color: red }</style>",
  `${original}\n<style>.authored-control { color: blue }</style>\n`,
];
for (const [index, source] of controls.entries()) {
  for (const ssr of [false, true]) {
    const filePath = `/src/VFHeading-${index}.vue`;
    const compiled = compileFile(
      filePath,
      new Map(),
      { ssr, vapor: false, sourceMap: true },
      source,
    );
    const output = generateOutputWithMap(compiled, {
      ssr,
      filePath,
      isDev: false,
      isProduction: false,
    });
    assert.equal(compiled.styles?.length, index === 2 ? 2 : 1);
    for (const block of compiled.styles!) {
      assert.ok(
        output.code.includes(`.__vize_style_${block.index}.css?`),
        `${ssr ? "SSR" : "client"} must retain each real style dependency`,
      );
    }
    assert.doesNotMatch(output.code, /document\.createElement\('style'\)/);
    const nativeMap = compiled.map ? JSON.parse(compiled.map) : null;
    assert.equal(
      output.map !== null,
      nativeMap !== null,
      "style imports preserve the compiler's optional source-map contract",
    );
    if (nativeMap) assert.deepEqual(output.map!.sourcesContent, nativeMap.sourcesContent);
    if (index !== 1) assert.ok(compiled.styles![0].content.includes("@media (--mobile)"));
  }
}
