import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { environment7502 } from "./support/native-attribute-values-7502-loader.ts";
import { checkMap, contexts, hash } from "./support/native-sfc-ssr-reference.ts";

const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../davinci/vize_l4/tests/fixtures/native-dom-static-class-vue-3.5.35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const ui = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const vue = createRequire(ui.resolve("vue/package.json"));
const compiler = vue("@vue/compiler-dom");
function prepared(code: string) {
  const imports = code.split("\n").filter((line) => line.startsWith("import "));
  return [
    ...imports,
    "const __sfc__ = {}",
    ";",
    code
      .slice(imports.join("\n").length)
      .trimStart()
      .replace(/^export /, ""),
    "__sfc__.render = render",
    "export default __sfc__",
    "",
  ].join("\n");
}

test("every supplemental class fixture retains whole official DOM code and raw map", () => {
  assert.equal(pack.schema, "vize.native-dom-static-class-reference");
  assert.equal(vue("@vue/compiler-dom/package.json").version, "3.5.35");
  assert.equal(pack.fixtures.length, 16);
  for (const row of pack.fixtures) {
    assert.equal(hash(row.source), row.sourceSha256);
    assert.equal(hash(row.code), row.codeSha256);
    assert.equal(hash(JSON.stringify(row.referenceMap)), row.referenceMapSha256);
    for (let repeat = 0; repeat < 2; repeat++) {
      const actual = compiler.compile(row.source, pack.options);
      assert.equal(actual.code, row.code);
      assert.deepEqual(actual.map, row.referenceMap);
    }
  }
});

const capturePath = process.env.VIZE_L4_DOM_STATIC_CLASS_CAPTURE;
const mode = process.env.NODE_ENV === "production" ? "production" : "development";
test(
  "fresh complete lower File class components mount and render like the official modules",
  {
    skip: !capturePath && process.env.VIZE_L4_DOM_STATIC_CLASS_REQUIRE_CAPTURE !== "1",
  },
  async () => {
    assert(capturePath, "hosted DOM class qualification requires the whole fresh source capture");
    const captures = JSON.parse(fs.readFileSync(capturePath, "utf8"));
    assert.deepEqual(
      captures.map((row: any) => row.id),
      pack.fixtures.map((row: any) => row.id),
    );
    const environment = await environment7502(mode);
    const execute = async (code: string) => {
      const loaded = await environment.load(code, "dom");
      const executions = [];
      for (const props of contexts) {
        const host = document.body.appendChild(document.createElement("div"));
        const warnings: string[] = [];
        const app = loaded.runtime.createApp(loaded.component, props);
        app.config.warnHandler = (message: string) => warnings.push(message);
        try {
          app.mount(host);
          await loaded.runtime.nextTick();
          const tree = environment.hostTree(host);
          app.unmount();
          await loaded.runtime.nextTick();
          assert.deepEqual(environment.hostTree(host), []);
          const ssrApp = environment.ssrRuntime.createSSRApp(loaded.component, props);
          const ssrWarnings: string[] = [];
          ssrApp.config.warnHandler = (message: string) => ssrWarnings.push(message);
          const html = await environment.server.renderToString(ssrApp);
          executions.push({ tree, warnings, html, ssrWarnings });
        } finally {
          host.remove();
        }
      }
      return executions;
    };
    const runtime = [];
    for (const [index, fixture] of pack.fixtures.entries()) {
      const capture = captures[index];
      assert.equal(capture.source, fixture.source);
      assert.equal(capture.code, fixture.code);
      assert.equal(capture.sfcCode, prepared(fixture.code));
      checkMap(capture);
      const component = {
        source: capture.source,
        filename: capture.filename,
        code: capture.sfcCode,
        map: capture.sfcMap,
        links: capture.sfcLinks,
      };
      checkMap(component);
      assert.throws(() => checkMap({ ...component, links: [] }), /complete start anchor/);
      const native = await execute(capture.sfcCode);
      const official = await execute(prepared(fixture.code));
      assert.deepEqual(native, official, fixture.id);
      assert.deepEqual(await execute(capture.sfcCode), native);
      assert.deepEqual(await execute(prepared(fixture.code)), official);
      runtime.push({
        id: fixture.id,
        sourceSha256: hash(capture.source),
        codeSha256: hash(capture.sfcCode),
        mapSha256: hash(JSON.stringify(capture.sfcMap)),
        linksSha256: hash(JSON.stringify(capture.sfcLinks)),
        native,
        official,
      });
    }
    if (process.env.VIZE_L4_DOM_STATIC_CLASS_RUNTIME_CAPTURE)
      fs.writeFileSync(
        `${process.env.VIZE_L4_DOM_STATIC_CLASS_RUNTIME_CAPTURE}.${mode}.json`,
        JSON.stringify(
          {
            runtimeMode: mode,
            modules: captures.length,
            nativeMounts: captures.length * 6,
            officialMounts: captures.length * 6,
            nativeWholeSsrExecutions: captures.length * 6,
            officialWholeSsrExecutions: captures.length * 6,
            selectedSfcClassAdmission: false,
            completeUpstreamMapParity: false,
            runtime,
          },
          null,
          2,
        ) + "\n",
      );
  },
);
