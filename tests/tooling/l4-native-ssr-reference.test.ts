import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

const pack = JSON.parse(
  fs.readFileSync(
    new URL("../../davinci/vize_l4/tests/fixtures/native-ssr-vue-3.5.35.json", import.meta.url),
    "utf8",
  ),
);
const hash = (text: string) => createHash("sha256").update(text).digest("hex");
const fromUi = createRequire(
  process.env.VIZE_TEST_VUE_PACKAGE ?? new URL("../../npm/ui/package.json", import.meta.url),
);
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const fromSfc = createRequire(fromVue.resolve("@vue/compiler-sfc/package.json"));
const compiler = fromSfc("@vue/compiler-ssr");
const url = (source: string) =>
  `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
const runtimeUrl = (specifier: string, names: string[]) =>
  url(
    `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve(specifier)).href)};\n` +
      names.map((name) => `export const ${name} = runtime.${name};`).join("\n"),
  );
const server = runtimeUrl("@vue/server-renderer", ["ssrRenderAttrs"]);
const vue = runtimeUrl("vue", ["mergeProps"]);

function groupedImports(code: string) {
  const imports = code.split("\n").filter((line) => line.startsWith("import "));
  if (!imports.length) return code;
  return (
    [...imports]
      .sort(
        (left, right) => Number(left.includes('from "vue"')) - Number(right.includes('from "vue"')),
      )
      .join("\n") + code.slice(imports.join("\n").length)
  );
}

test("eleven complete pinned SSR modules preserve full code, maps and selected options", () => {
  assert.equal(pack.schema, "vize.native-ssr-reference");
  assert.equal(pack.version, 1);
  assert.equal(pack.compiler.name, "@vue/compiler-ssr");
  assert.equal(pack.compiler.version, "3.5.35");
  assert.equal(pack.fixtures.length, 11);
  assert.equal(new Set(pack.fixtures.map((row: any) => row.id)).size, 11);
  assert.equal(fromSfc("@vue/compiler-ssr/package.json").version, "3.5.35");
  assert.equal(fromVue("@vue/server-renderer/package.json").version, "3.5.35");
  assert.equal(fromVue("vue/package.json").version, "3.5.35");
  assert.deepEqual(pack.options, {
    mode: "module",
    ssrRuntimeModuleName: "@vue/server-renderer",
    comments: true,
    filename: "NativeSsr.vue",
    sourceMap: true,
  });
  for (const fixture of pack.fixtures) {
    assert.equal(hash(fixture.source), fixture.sourceSha256);
    assert.equal(hash(fixture.referenceCode), fixture.referenceCodeSha256);
    assert.equal(hash(fixture.code), fixture.codeSha256);
    assert.equal(hash(JSON.stringify(fixture.referenceMap)), fixture.referenceMapSha256);
    assert.equal(groupedImports(fixture.referenceCode), fixture.code);
    for (let repeat = 0; repeat < 2; repeat++) {
      const measured = compiler.compile(fixture.source, pack.options);
      assert.equal(measured.code, fixture.referenceCode, fixture.id);
      assert.deepEqual(measured.map, fixture.referenceMap, fixture.id);
    }
  }
});

const contexts = [
  {},
  {
    id: "override",
    class: ["root", { active: true }],
    style: { color: "red" },
    title: '"<&>',
    disabled: true,
    onClick() {
      throw Error("SSR event executed");
    },
  },
  { id: null, title: "${untrusted}\\`", disabled: false, "data-x": "猫&" },
];

async function execute(code: string) {
  const loaded = await import(
    url(
      code
        .replace('from "@vue/server-renderer"', `from ${JSON.stringify(server)}`)
        .replace('from "vue"', `from ${JSON.stringify(vue)}`),
    )
  );
  const outcomes: string[] = [];
  for (const attrs of contexts) {
    const chunks: string[] = [];
    const forbidden = new Proxy(Object.create(null), {
      get(_, property) {
        throw Error(`static SSR read ${String(property)}`);
      },
    });
    assert.equal(
      (loaded.ssrRender ?? loaded.default.ssrRender)(
        forbidden,
        (chunk: string) => chunks.push(chunk),
        forbidden,
        attrs,
      ),
      undefined,
    );
    assert(chunks.every((chunk) => typeof chunk === "string"));
    outcomes.push(chunks.join(""));
  }
  return outcomes;
}

for (const fixture of pack.fixtures) {
  test(`${fixture.id} complete upstream SSR function executes with three fallthrough contexts`, async () => {
    assert.deepEqual(await execute(fixture.code), await execute(fixture.referenceCode));
  });
}

const capturePath = process.env.VIZE_L4_SSR_NATIVE_CAPTURE;
const requireNative = process.env.VIZE_L4_SSR_REQUIRE_NATIVE === "1";
test(
  "fresh source-built native modules execute with complete maps and real SSR helpers",
  { skip: !capturePath && !requireNative },
  async () => {
    assert(capturePath, "hosted native SSR requires its fresh Rust source capture");
    const captures = JSON.parse(fs.readFileSync(capturePath, "utf8"));
    assert.equal(captures.length, pack.fixtures.length);
    assert.deepEqual(
      captures.map((row: any) => row.id),
      pack.fixtures.map((row: any) => row.id),
    );
    const runtime = [];
    for (const [index, fixture] of pack.fixtures.entries()) {
      const capture = captures[index];
      assert.equal(capture.source, fixture.source);
      assert.equal(capture.outcome, "complete_module");
      assert.equal(capture.code, fixture.code);
      assert.deepEqual(capture.map.sources, [pack.options.filename]);
      assert.deepEqual(capture.map.sourcesContent, [fixture.source]);
      if (fixture.source) assert(capture.map.mappings.length > 0);
      assert.deepEqual(capture.sfcMap.sources, [pack.options.filename]);
      assert.deepEqual(capture.sfcMap.sourcesContent, [fixture.source]);
      assert(capture.sfcCode.endsWith("__sfc__.ssrRender = ssrRender\nexport default __sfc__\n"));
      const native = await execute(capture.code);
      const sfc = await execute(capture.sfcCode);
      assert.deepEqual(sfc, native, `${fixture.id}: whole prepared component module`);
      assert.deepEqual(native, await execute(fixture.referenceCode), fixture.id);
      runtime.push({
        id: fixture.id,
        codeSha256: hash(capture.code),
        mapSha256: hash(JSON.stringify(capture.map)),
        html: native,
      });
    }
    if (process.env.VIZE_L4_SSR_RUNTIME_CAPTURE)
      fs.writeFileSync(
        process.env.VIZE_L4_SSR_RUNTIME_CAPTURE,
        JSON.stringify(
          {
            modules: captures.length,
            nativeExecutions: captures.length * contexts.length,
            upstreamExecutions: captures.length * contexts.length,
            preparedSfcExecutions: captures.length * contexts.length,
            completeUpstreamMapParity: false,
            runtime,
          },
          null,
          2,
        ) + "\n",
      );
  },
);
