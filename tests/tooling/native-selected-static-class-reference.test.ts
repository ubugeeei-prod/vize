import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { environment7502 } from "./support/native-attribute-values-7502-loader.ts";
import { primaryStaticClass } from "./support/native-static-class-primary.ts";
import { checkMap, contexts, hash } from "./support/native-sfc-ssr-reference.ts";

const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native-selected-static-class-vue-3.5.35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const ui = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const vue = createRequire(ui.resolve("vue/package.json"));
function prepared(code: string, target: string, scope?: string) {
  const imports = code.split("\n").filter((line) => line.startsWith("import "));
  const binding = target === "dom" ? "render" : "ssrRender";
  return [
    ...imports,
    "const _sfc_main = {}",
    ";",
    code
      .slice(imports.join("\n").length)
      .trimStart()
      .replace(/^export /, ""),
    `_sfc_main.${binding} = ${binding}`,
    ...(scope ? [`_sfc_main.__scopeId = ${JSON.stringify(scope)}`] : []),
    "export default _sfc_main",
    "",
  ].join("\n");
}
test("the complete primary DOM/SSR packets retain both original code and raw maps", () => {
  assert.equal(pack.schema, "vize.native-selected-static-class-reference");
  assert.equal(pack.fixtures.length, 19);
  assert.equal(
    hash(JSON.stringify(pack.fixtures.slice(0, 16))),
    "a112e394d2b546bb8f91d48aaf4edbac453616494f0bb6cd73826c2a91725286",
  );
  for (const target of ["dom", "ssr"]) {
    assert.equal(vue(`@vue/compiler-${target}/package.json`).version, "3.5.35");
    const fixtures = [...pack.fixtures, ...(target === "ssr" ? [pack.scoped] : [])];
    const rows = fixtures.map((row: any) => ({ ...row, target }));
    const references = primaryStaticClass(rows, pack.options);
    assert.deepEqual(primaryStaticClass(rows, pack.options), references);
    for (const [index, fixture] of fixtures.entries()) {
      assert.equal(hash(fixture.source), fixture.sourceSha256);
      assert.equal(hash(fixture[target].code), fixture[target].codeSha256);
      assert.equal(hash(JSON.stringify(fixture[target].map)), fixture[target].mapSha256);
      assert.equal(references[index].code, fixture[target].code);
      assert.deepEqual(references[index].map, fixture[target].map);
    }
  }
  const css = vue("@vue/compiler-sfc").compileStyle({
    source: pack.scoped.cssSource,
    filename: pack.scoped.filename,
    id: pack.scoped.scopeId,
    scoped: true,
  });
  assert.deepEqual(css.errors, []);
  assert.equal(css.code, pack.scoped.css);
});

const capturePath = process.env.VIZE_NATIVE_SELECTED_STATIC_CLASS_CAPTURE;
const mode = process.env.NODE_ENV === "production" ? "production" : "development";
test(
  "whole public selected class components execute with complete original source anchors",
  {
    skip: !capturePath && process.env.VIZE_NATIVE_SELECTED_STATIC_CLASS_REQUIRE_CAPTURE !== "1",
  },
  async () => {
    assert(capturePath, "hosted selected static class requires every fresh public API output");
    const captures = JSON.parse(fs.readFileSync(capturePath, "utf8"));
    const expected = pack.fixtures.flatMap((fixture: any) =>
      [true, false].flatMap((sourceMap) =>
        ["dom", "ssr"].map((target) => [fixture.id, target, sourceMap]),
      ),
    );
    expected.push(...[true, false].map((sourceMap) => [pack.scoped.id, "scoped-ssr", sourceMap]));
    assert.deepEqual(
      captures.map((row: any) => [row.id, row.target, row.sourceMap]),
      expected,
    );
    const environment = await environment7502(mode);
    const execute = async (code: string, target: string) => {
      const loaded = await environment.load(
        code,
        target === "dom" ? "dom" : "ssr",
        null,
        target !== "dom",
      );
      const executions = [];
      for (const props of contexts) {
        const warnings: string[] = [];
        if (target === "dom") {
          const host = document.body.appendChild(document.createElement("div"));
          const app = loaded.runtime.createApp(loaded.component, props);
          app.config.warnHandler = (message: string) => warnings.push(message);
          try {
            app.mount(host);
            await loaded.runtime.nextTick();
            const tree = environment.hostTree(host);
            app.unmount();
            await loaded.runtime.nextTick();
            assert.deepEqual(environment.hostTree(host), []);
            executions.push({ tree, warnings });
          } finally {
            host.remove();
          }
        } else {
          const app = loaded.runtime.createSSRApp(loaded.component, props);
          app.config.warnHandler = (message: string) => warnings.push(message);
          executions.push({ html: await loaded.renderer.renderToString(app), warnings });
        }
      }
      return executions;
    };
    const runtime = [];
    for (const row of captures) {
      const fixture =
        row.id === pack.scoped.id ? pack.scoped : pack.fixtures.find((f: any) => f.id === row.id);
      assert(fixture);
      assert.equal(row.source, fixture.source);
      const target = row.target === "dom" ? "dom" : "ssr";
      if (target === "dom")
        assert.equal(row.code, prepared(fixture[target].code, target, row.scopeId));
      const compiler = vue(`@vue/compiler-${target}`);
      const options = { ...pack.options, ...(row.scopeId ? { scopeId: row.scopeId } : {}) };
      const officialReference = compiler.compile(fixture.template, options);
      assert.equal(compiler.compile(fixture.template, options).code, officialReference.code);
      assert.deepEqual(compiler.compile(fixture.template, options).map, officialReference.map);
      const officialCode = prepared(officialReference.code, target, row.scopeId);
      if (row.sourceMap) {
        checkMap(row);
        const classAnchors = row.links.filter(
          (link: any) =>
            link.segment &&
            Buffer.from(row.source)
              .subarray(link.authored.start, link.authored.end)
              .toString("utf8")
              .startsWith("class"),
        );
        assert(classAnchors.length > 0, `${row.id}: retain the genuine authored class anchors`);
        assert.throws(
          () =>
            checkMap({
              ...row,
              links: row.links.filter((link: any) => !classAnchors.includes(link)),
            }),
          /complete start anchor/,
        );
        if (row.target === "scoped-ssr") {
          assert.equal(row.css, pack.scoped.css);
          checkMap({ ...row, code: row.css, map: row.cssMap, links: row.cssLinks });
        }
      } else {
        assert.equal(row.map, null);
        assert.deepEqual(row.links, []);
      }
      const native = await execute(row.code, row.target);
      const official = await execute(officialCode, row.target);
      assert.deepEqual(native, official);
      assert.deepEqual(await execute(row.code, row.target), native);
      assert.deepEqual(await execute(officialCode, row.target), official);
      runtime.push({
        id: row.id,
        target: row.target,
        sourceMap: row.sourceMap,
        sourceSha256: hash(row.source),
        codeSha256: hash(row.code),
        mapSha256: hash(JSON.stringify(row.map)),
        linksSha256: hash(JSON.stringify(row.links)),
        officialCode: officialReference.code,
        officialMap: officialReference.map,
        native,
        official,
      });
    }
    if (process.env.VIZE_NATIVE_SELECTED_STATIC_CLASS_RUNTIME_CAPTURE)
      fs.writeFileSync(
        `${process.env.VIZE_NATIVE_SELECTED_STATIC_CLASS_RUNTIME_CAPTURE}.${mode}.json`,
        JSON.stringify(
          {
            runtimeMode: mode,
            outcomes: captures.length,
            nativeExecutions: captures.length * 6,
            officialExecutions: captures.length * 6,
            publicSelectedClassAdmission: true,
            vaporStaticClassAdmission: false,
            completeUpstreamMapParity: false,
            nativePacketReviewed: pack.nativeCapture !== null,
            runtime,
          },
          null,
          2,
        ) + "\n",
      );
    assert.notEqual(pack.nativeCapture, null, "native class packet is unreviewed");
    assert.deepEqual(captures, pack.nativeCapture);
  },
);
