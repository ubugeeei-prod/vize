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
const fromMagic = createRequire(fromSfc.resolve("magic-string/package.json"));
const mapCodec = fromMagic("@jridgewell/sourcemap-codec");
const url = (source: string) =>
  `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
const runtimeUrl = (specifier: string, names: string[]) =>
  url(
    `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve(specifier)).href)};\n` +
      names.map((name) => `export const ${name} = runtime.${name};`).join("\n"),
  );
const server = runtimeUrl("@vue/server-renderer", ["ssrRenderAttrs", "ssrRenderComponent"]);
const vue = runtimeUrl("vue", ["mergeProps", "resolveComponent"]);

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

function mapAnchors(map: any, code: string, source: string): number[][] {
  assert.equal(map.version, 3);
  assert.equal(map.file, pack.options.filename);
  assert(Array.isArray(map.names));
  assert(map.names.every((name: unknown) => typeof name === "string"));
  const generated = code.split(/\r\n|[\r\n\u2028\u2029]/);
  const authored = source.split(/\r\n|[\r\n\u2028\u2029]/);
  const decoded: number[][][] = mapCodec.decode(map.mappings);
  assert.equal(mapCodec.encode(decoded), map.mappings);
  const anchors = decoded.flatMap((segments, line) => {
    let previous = -1;
    return segments.map((segment) => {
      assert([4, 5].includes(segment.length));
      assert(segment.every(Number.isSafeInteger));
      assert(line < generated.length);
      assert(segment[0] >= 0 && segment[0] >= previous && segment[0] <= generated[line].length);
      previous = segment[0];
      assert.equal(segment[1], 0);
      assert(segment[2] >= 0 && segment[2] < authored.length);
      assert(segment[3] >= 0 && segment[3] <= authored[segment[2]].length);
      if (segment.length === 5) assert(segment[4] >= 0 && segment[4] < map.names.length);
      return [line, ...segment];
    });
  });
  assert.equal(anchors.length > 0, source.length > 0);
  return anchors;
}

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
      const templateAnchors = mapAnchors(capture.map, capture.code, fixture.source);
      const componentAnchors = mapAnchors(capture.sfcMap, capture.sfcCode, fixture.source);
      const functionLine = (code: string) =>
        code.split("\n").findIndex((line) => /^(export )?function ssrRender\(/.test(line));
      const shift = functionLine(capture.sfcCode) - functionLine(capture.code);
      assert.equal(shift, 1);
      assert.deepEqual(capture.sfcMap.names, capture.map.names);
      assert.deepEqual(
        componentAnchors,
        templateAnchors.map(([line, ...segment]) => [line + shift, ...segment]),
        `${fixture.id}: every prepared-module anchor retains its original UTF-16 position`,
      );
      const imports = fixture.code.split("\n").filter((line: string) => line.startsWith("import "));
      const declaration = fixture.code.slice(imports.join("\n").length).trimStart();
      assert.equal(
        capture.sfcCode,
        [
          ...imports,
          "const __sfc__ = {}",
          ";",
          declaration.replace(/^export /, ""),
          "__sfc__.ssrRender = ssrRender",
          "export default __sfc__",
          "",
        ].join("\n"),
        `${fixture.id}: complete prepared component module`,
      );
      const native = await execute(capture.code);
      const sfc = await execute(capture.sfcCode);
      assert.deepEqual(sfc, native, `${fixture.id}: whole prepared component module`);
      assert.deepEqual(native, await execute(fixture.referenceCode), fixture.id);
      runtime.push({
        id: fixture.id,
        codeSha256: hash(capture.code),
        mapSha256: hash(JSON.stringify(capture.map)),
        sfcCodeSha256: hash(capture.sfcCode),
        sfcMapSha256: hash(JSON.stringify(capture.sfcMap)),
        referenceCodeSha256: fixture.referenceCodeSha256,
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

const roles = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../davinci/vize_l4/tests/fixtures/native-ssr-component-roles-vue-3.5.35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const roleRuntime: any[] = [];
test("complete pinned search modules resolve Vue components with actual SSR outcomes", async () => {
  assert.equal(roles.schema, "vize.native-ssr-component-role-reference");
  assert.equal(roles.version, 1);
  assert.equal(roles.compiler.name, "@vue/compiler-ssr");
  assert.equal(roles.compiler.version, "3.5.35");
  assert.deepEqual(roles.options, { ...pack.options, filename: "NativeSsrRole.vue" });
  assert.equal(roles.fixtures.length, 2);
  const core = fromVue("vue");
  const renderer = fromVue("@vue/server-renderer");
  for (const fixture of roles.fixtures) {
    assert.equal(hash(fixture.template), fixture.templateSha256);
    assert.equal(hash(fixture.referenceCode), fixture.referenceCodeSha256);
    assert.equal(hash(JSON.stringify(fixture.referenceMap)), fixture.referenceMapSha256);
    const measured = compiler.compile(fixture.template, roles.options);
    assert.equal(measured.code, fixture.referenceCode);
    assert.deepEqual(measured.map, fixture.referenceMap);
    const loaded = await import(
      url(
        fixture.referenceCode
          .replace('from "@vue/server-renderer"', `from ${JSON.stringify(server)}`)
          .replace('from "vue"', `from ${JSON.stringify(vue)}`),
      )
    );
    const warnings: string[] = [];
    const previousWarn = console.warn;
    let unregistered: string, registered: string;
    console.warn = (...args: unknown[]) => {
      warnings.push(args.map(String).join(" "));
    };
    try {
      const app = core.createSSRApp({ ssrRender: loaded.ssrRender });
      app.config.warnHandler = (message: string) => warnings.push(message);
      unregistered = await renderer.renderToString(app);
      registered = await renderer.renderToString(
        core.createSSRApp({
          components: {
            search: {
              ssrRender(_ctx: unknown, push: (text: string) => void) {
                push("<strong>resolved</strong>");
              },
            },
          },
          ssrRender: loaded.ssrRender,
        }),
      );
    } finally {
      console.warn = previousWarn;
    }
    assert.equal(unregistered, fixture.unregisteredHtml);
    assert.equal(registered, fixture.registeredHtml);
    assert.deepEqual(warnings, fixture.warnings);
    roleRuntime.push({
      id: fixture.id,
      referenceCodeSha256: fixture.referenceCodeSha256,
      unregistered,
      registered,
      warnings,
    });
  }
});

test(
  "fresh original selected search views preserve neutral HTML but refuse Vue SSR",
  { skip: !capturePath && !requireNative },
  () => {
    assert(capturePath, "hosted SSR requires its fresh original selected role refusals");
    const captures = JSON.parse(fs.readFileSync(`${capturePath}.refusals.json`, "utf8"));
    assert.equal(captures.length, 2);
    assert.equal(roleRuntime.length, 2);
    for (const [index, fixture] of roles.fixtures.entries()) {
      const capture = captures[index];
      assert.equal(capture.id, fixture.id);
      assert.equal(capture.source, fixture.source);
      assert.equal(capture.template, fixture.template);
      assert.equal(capture.outcome, "component_role_refusal");
      assert.equal(capture.reason, "ElementSemantics");
      assert.equal(capture.node, fixture.node);
      assert.equal(
        Buffer.from(capture.source).subarray(capture.span.start, capture.span.end).toString(),
        "<search/>",
      );
    }
    if (process.env.VIZE_L4_SSR_RUNTIME_CAPTURE) {
      const runtime = JSON.parse(fs.readFileSync(process.env.VIZE_L4_SSR_RUNTIME_CAPTURE, "utf8"));
      runtime.componentRoleRefusals = captures;
      runtime.componentRoleReferenceExecutions = 4;
      runtime.componentRoleRuntime = roleRuntime;
      fs.writeFileSync(
        process.env.VIZE_L4_SSR_RUNTIME_CAPTURE,
        JSON.stringify(runtime, null, 2) + "\n",
      );
    }
  },
);
