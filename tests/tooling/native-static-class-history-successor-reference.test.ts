import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { environment7502 } from "./support/native-attribute-values-7502-loader.ts";
import { primaryStaticClass } from "./support/native-static-class-primary.ts";
import { checkMap, contexts, hash } from "./support/native-sfc-ssr-reference.ts";

const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../davinci/vize_l4/tests/fixtures/native-static-class-history-successor-vue-3.5.35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const mode = process.env.NODE_ENV === "production" ? "production" : "development";
test("the exact original late class source preserves full independent dev/prod modules and raw maps", () => {
  assert.equal(pack.schema, "vize.native-static-class-history-successor");
  assert.equal(pack.version, 1);
  assert.equal(pack.compilerVersion, "3.5.35");
  assert.equal(
    pack.source,
    "<template>prefix<div data-first='kept' class='a  b'>unvisited</div>tail</template>",
  );
  assert.equal(hash(pack.source), pack.sourceSha256);
  assert.equal(pack.source, `<template>${pack.template}</template>`);
  assert.deepEqual(pack.options, {
    mode: "module",
    hoistStatic: false,
    prefixIdentifiers: true,
    comments: true,
    filename: "NativeClassHistory.vue",
    sourceMap: true,
    bindingMetadata: {},
    cacheHandlers: false,
  });
  for (const target of ["dom", "ssr"])
    for (const runtimeMode of ["development", "production"]) {
      const rows = [{ template: pack.template, target }];
      const [actual] = primaryStaticClass(rows, pack.options, runtimeMode);
      assert.deepEqual(primaryStaticClass(rows, pack.options, runtimeMode), [actual]);
      const reference = pack[target][runtimeMode];
      assert.equal(actual.code, reference.code);
      assert.deepEqual(actual.map, reference.map);
      assert.equal(hash(reference.code), reference.codeSha256);
      assert.equal(hash(JSON.stringify(reference.map)), reference.mapSha256);
    }
});
function component(code: string, target: string) {
  const binding = target === "dom" ? "render" : "ssrRender";
  return (
    code.replace(`export function ${binding}`, `function ${binding}`) +
    `\nexport default { ${binding} }\n`
  );
}
const capturePath = process.env.VIZE_NATIVE_SELECTED_STATIC_CLASS_CAPTURE;
test(
  "all four whole original late header successors qualify full runtime, maps and links",
  {
    skip: !capturePath && process.env.VIZE_NATIVE_SELECTED_STATIC_CLASS_REQUIRE_CAPTURE !== "1",
  },
  async () => {
    assert(capturePath, "mandatory fresh original source capture");
    const captures = JSON.parse(fs.readFileSync(`${capturePath}.late-header.json`, "utf8"));
    assert.deepEqual(
      captures.map((row: any) => [row.target, row.sourceMap]),
      [
        ["dom", true],
        ["ssr", true],
        ["dom", false],
        ["ssr", false],
      ],
    );
    const environment = await environment7502(mode);
    const execute = async (code: string, target: string) => {
      const loaded = await environment.load(code, target, null, target === "ssr");
      const outcomes = [];
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
            outcomes.push({ tree, warnings });
          } finally {
            host.remove();
          }
        } else {
          const app = loaded.runtime.createSSRApp(loaded.component, props);
          app.config.warnHandler = (message: string) => warnings.push(message);
          outcomes.push({ html: await loaded.renderer.renderToString(app), warnings });
        }
      }
      return outcomes;
    };
    const runtime = [];
    for (const row of captures) {
      assert.equal(row.source, pack.source);
      assert.equal(row.filename, pack.options.filename);
      if (row.sourceMap) {
        checkMap(row);
        const anchors = row.links.filter(
          (link: any) =>
            link.segment &&
            Buffer.from(row.source)
              .subarray(link.authored.start, link.authored.end)
              .toString("utf8")
              .startsWith("class"),
        );
        assert(anchors.length > 0);
        assert.throws(
          () =>
            checkMap({ ...row, links: row.links.filter((link: any) => !anchors.includes(link)) }),
          /complete start anchor/,
        );
      } else {
        assert.equal(row.map, null);
        assert.deepEqual(row.links, []);
      }
      const reference = pack[row.target][mode];
      const officialCode = component(reference.code, row.target);
      const native = await execute(row.code, row.target),
        official = await execute(officialCode, row.target);
      assert.deepEqual(native, official);
      assert.deepEqual(await execute(row.code, row.target), native);
      assert.deepEqual(await execute(officialCode, row.target), official);
      runtime.push({
        target: row.target,
        sourceMap: row.sourceMap,
        sourceSha256: hash(row.source),
        codeSha256: hash(row.code),
        mapSha256: hash(JSON.stringify(row.map)),
        linksSha256: hash(JSON.stringify(row.links)),
        officialCode: reference.code,
        officialMap: reference.map,
        native,
        official,
      });
    }
    if (process.env.VIZE_NATIVE_SELECTED_STATIC_CLASS_RUNTIME_CAPTURE)
      fs.writeFileSync(
        `${process.env.VIZE_NATIVE_SELECTED_STATIC_CLASS_RUNTIME_CAPTURE}.late-header.${mode}.json`,
        JSON.stringify(
          {
            mode,
            outcomes: 4,
            nativeExecutions: 24,
            officialExecutions: 24,
            nativePacketReviewed: pack.nativeCapture !== null,
            completeUpstreamMapParity: false,
            runtime,
          },
          null,
          2,
        ) + "\n",
      );
    assert.notEqual(
      pack.nativeCapture,
      null,
      "whole original late class successor remains unreviewed",
    );
    assert.deepEqual(captures, pack.nativeCapture);
  },
);
