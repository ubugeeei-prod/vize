import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../davinci/vize_l4/tests/fixtures/native-selected-ssr-vue-3.5.35.json",
      import.meta.url,
    ),
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
const fromMagic = createRequire(fromSfc.resolve("magic-string/package.json"));
const codec = fromMagic("@jridgewell/sourcemap-codec");
const core = fromVue("vue");
const renderer = fromVue("@vue/server-renderer");
const url = (source: string) =>
  `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
const runtimeUrl = (specifier: string, names: string[]) =>
  url(
    `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve(specifier)).href)};\n` +
      names.map((name) => `export const ${name} = runtime.${name};`).join("\n"),
  );
const server = runtimeUrl("@vue/server-renderer", ["ssrRenderAttrs"]);
const vue = runtimeUrl("vue", ["mergeProps"]);
const load = (code: string) =>
  import(
    url(
      code
        .replace('from "@vue/server-renderer"', `from ${JSON.stringify(server)}`)
        .replace('from "vue"', `from ${JSON.stringify(vue)}`),
    )
  );
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
const lineEndingHtml =
  '<div id="r\r\ns\rt\nu\u2028v\u2029w"><p title="a\nb\nc\nd\u2028e\u2029f"></p><!--a\nb\nc\nd\u2028e\u2029f--></div>';

function grouped(code: string) {
  const imports = code.split("\n").filter((line) => line.startsWith("import "));
  return imports.length
    ? [...imports]
        .sort((a, b) => Number(a.includes('from "vue"')) - Number(b.includes('from "vue"')))
        .join("\n") + code.slice(imports.join("\n").length)
    : code;
}
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
async function direct(code: string) {
  const loaded = await load(code);
  return contexts.map((attrs) => {
    const chunks: string[] = [];
    const forbidden = new Proxy(Object.create(null), {
      get(_, name) {
        throw Error(`static SSR read ${String(name)}`);
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
    return chunks.join("");
  });
}
async function whole(code: string, component: boolean) {
  const loaded = await load(code);
  const output: string[] = [];
  for (const attrs of contexts) {
    const app = core.createSSRApp(
      component ? loaded.default : { ssrRender: loaded.ssrRender },
      attrs,
    );
    const warnings: string[] = [];
    app.config.warnHandler = (message: string) => warnings.push(message);
    output.push(await renderer.renderToString(app));
    assert.deepEqual(warnings, [], "scriptless bounded SSR module emits no Vue warnings");
  }
  return output;
}

function anchors(map: any, code: string, source: string): number[][] {
  assert.equal(map.version, 3);
  assert.equal(map.file, pack.options.filename);
  assert.deepEqual(map.sources, [pack.options.filename]);
  assert.deepEqual(map.sourcesContent, [source]);
  assert(Array.isArray(map.names));
  assert(map.names.every((name: unknown) => typeof name === "string"));
  const generated = code.split(/\r\n|[\r\n\u2028\u2029]/);
  const authored = source.split(/\r\n|[\r\n\u2028\u2029]/);
  const decoded: number[][][] = codec.decode(map.mappings);
  assert.equal(codec.encode(decoded), map.mappings);
  return decoded.flatMap((segments, line) => {
    let previous = -1;
    return segments.map((segment) => {
      assert([4, 5].includes(segment.length));
      assert(segment.every(Number.isSafeInteger));
      assert(
        line < generated.length &&
          segment[0] >= 0 &&
          segment[0] >= previous &&
          segment[0] <= generated[line].length,
      );
      previous = segment[0];
      assert.equal(segment[1], 0);
      assert(segment[2] >= 0 && segment[2] < authored.length);
      assert(segment[3] >= 0 && segment[3] <= authored[segment[2]].length);
      if (segment.length === 5) assert(segment[4] >= 0 && segment[4] < map.names.length);
      return [line, ...segment];
    });
  });
}

test("nine complete pinned SSR references retain original SFC inputs and full primary outputs", async () => {
  assert.equal(pack.schema, "vize.native-selected-ssr-reference");
  assert.equal(pack.version, 1);
  assert.deepEqual(pack.compiler, { name: "@vue/compiler-ssr", version: "3.5.35" });
  assert.equal(pack.fixtures.length, 9);
  assert.equal(new Set(pack.fixtures.map((row: any) => row.id)).size, 9);
  assert.equal(fromSfc("@vue/compiler-ssr/package.json").version, "3.5.35");
  assert.equal(fromVue("@vue/server-renderer/package.json").version, "3.5.35");
  assert.equal(fromVue("vue/package.json").version, "3.5.35");
  assert.deepEqual(pack.options, {
    mode: "module",
    ssrRuntimeModuleName: "@vue/server-renderer",
    comments: true,
    filename: "NativeSelectedSsr.vue",
    sourceMap: true,
  });
  for (const row of pack.fixtures) {
    assert.equal(row.source, `<template>${row.template}</template>`);
    assert.equal(hash(row.source), row.sourceSha256);
    assert.equal(hash(row.template), row.templateSha256);
    assert.equal(hash(row.referenceCode), row.referenceCodeSha256);
    assert.equal(hash(JSON.stringify(row.referenceMap)), row.referenceMapSha256);
    assert.equal(hash(row.code), row.codeSha256);
    assert.equal(grouped(row.referenceCode), row.code);
    for (let repeat = 0; repeat < 2; repeat++) {
      const measured = compiler.compile(row.template, pack.options);
      assert.equal(measured.code, row.referenceCode);
      assert.deepEqual(measured.map, row.referenceMap);
    }
    const html = await direct(row.code);
    assert.deepEqual(html, await direct(row.referenceCode));
    if (row.id === "line-endings") assert.equal(html[0], lineEndingHtml);
    assert.deepEqual(await whole(prepared(row.code), true), await whole(row.referenceCode, false));
  }
});

const capturePath = process.env.VIZE_L4_SELECTED_SSR_NATIVE_CAPTURE;
test(
  "fresh original receipts execute complete modules, whole components and every map anchor",
  { skip: !capturePath && process.env.VIZE_L4_SSR_REQUIRE_NATIVE !== "1" },
  async () => {
    assert(capturePath, "hosted SSR requires fresh source-built original completion receipts");
    const capture = JSON.parse(fs.readFileSync(capturePath, "utf8"));
    assert.equal(capture.custody, "original_completed_template");
    assert.deepEqual(
      capture.modules.map((row: any) => row.id),
      pack.fixtures.map((row: any) => row.id),
    );
    const runtime = [];
    for (const [index, row] of pack.fixtures.entries()) {
      const actual = capture.modules[index];
      assert.equal(actual.outcome, "complete_original_module");
      assert.equal(actual.source, row.source);
      assert.equal(actual.template, row.template);
      assert.equal(actual.code, row.code);
      assert.equal(actual.sfcCode, prepared(row.code));
      const base = anchors(actual.map, actual.code, row.source);
      const component = anchors(actual.sfcMap, actual.sfcCode, row.source);
      assert.equal(base.length > 0, row.template.length > 0);
      assert.deepEqual(actual.sfcMap.names, actual.map.names);
      assert.deepEqual(
        component,
        base.map(([line, ...segment]) => [line + 1, ...segment]),
      );
      const html = await direct(actual.code);
      if (row.id === "line-endings") {
        assert.equal(html[0], lineEndingHtml);
        for (const token of ["id=", "title=", "<!--"]) {
          const prefix = row.source.slice(0, row.source.indexOf(token));
          const lines = prefix.split(/\r\n|[\r\n\u2028\u2029]/);
          assert(
            base.some(
              (anchor) => anchor[3] === lines.length - 1 && anchor[4] === lines.at(-1).length,
            ),
            `original ${token} anchor follows every standard source line terminator`,
          );
        }
      }
      assert.deepEqual(html, await direct(actual.sfcCode));
      assert.deepEqual(html, await direct(row.referenceCode));
      const componentHtml = await whole(actual.sfcCode, true);
      assert.deepEqual(componentHtml, await whole(row.referenceCode, false));
      runtime.push({
        id: row.id,
        sourceSha256: row.sourceSha256,
        codeSha256: hash(actual.code),
        mapSha256: hash(JSON.stringify(actual.map)),
        sfcCodeSha256: hash(actual.sfcCode),
        sfcMapSha256: hash(JSON.stringify(actual.sfcMap)),
        referenceCodeSha256: row.referenceCodeSha256,
        html,
        componentHtml,
      });
    }
    assert.deepEqual(
      capture.refusals.map((row: any) => row.id),
      ["search-root", "search-nested", "global-style", "scoped-style"],
    );
    for (const [index, row] of capture.refusals.entries()) {
      const expected = [
        "<template><search/></template>",
        "<template><div><search/></div></template>",
        "<template><div/></template><style></style>",
        "<style scoped>div{color:red}</style><template><div/></template>",
      ][index];
      assert.equal(row.source, expected);
      if (index < 2) {
        assert.equal(row.outcome, "original_role_refusal");
        assert.equal(row.reason, "ElementSemantics");
        assert.equal(row.node, index);
        assert.equal(
          Buffer.from(row.source).subarray(row.span.start, row.span.end).toString(),
          "<search/>",
        );
      } else {
        assert.equal(row.outcome, "original_style_refusal");
        assert.equal(row.reason, "StyledSource");
        assert.deepEqual(row.span, { start: 0, end: Buffer.byteLength(row.source) });
      }
    }
    if (process.env.VIZE_L4_SELECTED_SSR_RUNTIME_CAPTURE)
      fs.writeFileSync(
        process.env.VIZE_L4_SELECTED_SSR_RUNTIME_CAPTURE,
        JSON.stringify(
          {
            custody: capture.custody,
            modules: 9,
            originalRefusals: 4,
            nativeExecutions: 27,
            upstreamExecutions: 27,
            preparedExecutions: 27,
            preparedVueExecutions: 27,
            upstreamVueExecutions: 27,
            completeUpstreamMapParity: false,
            runtime,
            refusals: capture.refusals,
          },
          null,
          2,
        ) + "\n",
      );
  },
);
