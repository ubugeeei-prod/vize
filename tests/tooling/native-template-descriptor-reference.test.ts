import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import test from "node:test";

const pack = JSON.parse(
  readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native_template_descriptor_vue_3_5_41.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const fromRoot = createRequire(new URL("../../package.json", import.meta.url));
const fromVue = createRequire(fromRoot.resolve("vue-computed-inlay-oracle/package.json"));
const fromSfc = createRequire(fromVue.resolve("@vue/compiler-sfc/package.json"));
const compiler = fromSfc("@vue/compiler-sfc");

test("original native descriptor controls match independent pinned official Vue", () => {
  assert.equal(pack.compiler.name, "@vue/compiler-sfc");
  assert.equal(pack.compiler.version, "3.5.41");
  assert.equal(fromSfc("@vue/compiler-sfc/package.json").version, pack.compiler.version);
  assert.equal(pack.cases.length, 25);
  assert.equal(new Set(pack.cases.map((row: any) => row.id)).size, 25);
  for (const row of pack.cases) {
    const result = compiler.parse(row.source, { filename: pack.filename, sourceMap: true });
    const block = result.descriptor.template;
    // Official offsets count UTF16 code units; the Rust compatibility facade
    // retains byte offsets. Convert using the untouched original source.
    const byte = (offset: number) => Buffer.byteLength(row.source.slice(0, offset));
    assert.deepEqual(
      {
        errors: result.errors.map((error: any) =>
          typeof error === "string"
            ? { message: error }
            : { ...(error.code === undefined ? {} : { code: error.code }), message: error.message },
        ),
        template: block
          ? {
              content: block.content,
              start: byte(block.loc.start.offset),
              end: byte(block.loc.end.offset),
              attrs: block.attrs,
            }
          : null,
      },
      row.official,
      row.id,
    );
  }
  // A real native boundary frontier: literal unclosed interpolation under
  // v-pre is valid Vue. The current native container refuses it, so this
  // projection must preserve the retained product route rather than lose it.
  const frozen = pack.cases.find((row: any) => row.id === "frozen-unclosed-interpolation");
  assert.equal(frozen.source, "<template><div v-pre>{{</div></template>");
  assert.equal(frozen.native, false);
  assert.deepEqual(frozen.official.errors, []);
  assert.equal(frozen.official.template.content, "<div v-pre>{{</div>");
});
