import assert from "node:assert/strict";
import { test } from "node:test";
import { sources } from "@rspack/core";
import { originalPositionFor, TraceMap } from "@jridgewell/trace-mapping";
import { parseSfc } from "@vizejs/native";
import { inlineSrcBlocks } from "./utils.ts";
import { inlineSrcBlock, MappedModule, type SourceMapV3 } from "./source-map.ts";

const script = 'export default { name: "Actual" };';
const template = "<p>Actual template</p>";

void test("src inlining preserves comment decoys and replaces the real script", () => {
  const comment = '<!-- <script src="./decoy.js"></script> -->\r\n';
  const tail = "\r\n<template><p>Tail</p></template>";
  const source = `${comment}<script src="./actual.js"></script>${tail}`;
  assert.equal(
    inlineSrcBlocks(source, script, null),
    `${comment}<script>\n${script}\n</script>${tail}`,
  );
});

void test("src inlining preserves template tags inside script strings", () => {
  const head =
    '<script>const decoy = `<template src="./decoy.html"></template>`; export default { decoy };</script>\r\n';
  const source = `${head}<template src="./actual.html"></template>`;
  assert.equal(
    inlineSrcBlocks(source, null, template),
    `${head}<template>\n${template}\n</template>`,
  );
});

for (const selfClosing of [false, true]) {
  void test(`both external blocks inline without consuming their tail (${selfClosing ? "self-closing" : "paired"})`, () => {
    const head = "<!-- 雪🐱 -->\r\n";
    const tail = "\r\n<style>.tail { color: red }</style>";
    const source =
      head +
      (selfClosing
        ? '<script src="./actual.js" />\r\n<template src="./actual.html" />'
        : '<script src="./actual.js"></script>\r\n<template src="./actual.html"></template>') +
      tail;
    assert.equal(
      inlineSrcBlocks(source, script, template),
      `${head}<script>\n${script}\n</script>\r\n<template>\n${template}\n</template>${tail}`,
    );
  });
}

void test("native header ranges preserve quoted greater-than signs and adjacent attributes", () => {
  const source =
    '<script lang="js"src="./actual.js" data-note="src=\'keep\'>"></script><template><p>Tail</p></template>';
  assert.equal(
    inlineSrcBlocks(source, script, null),
    `<script lang="js" data-note="src='keep'>">\n${script}\n</script><template><p>Tail</p></template>`,
  );
});

for (const tag of ["script", "template"] as const) {
  void test(`removing a first src attribute keeps the ${tag} name separate from its next attribute`, () => {
    const content = tag === "script" ? script : template;
    const lang = tag === "script" ? "js" : "html";
    const source = `<${tag} src="./actual"lang="${lang}"></${tag}>`;
    assert.equal(
      inlineSrcBlocks(
        source,
        tag === "script" ? content : null,
        tag === "template" ? content : null,
      ),
      `<${tag} lang="${lang}">\n${content}\n</${tag}>`,
    );
  });
}

void test("without a real src block, supplied contents do not rewrite literal tags", () => {
  const source =
    '<script>export default { decoy: `<template src="./decoy.html"></template>` };</script><template><p>Tail</p></template>';
  assert.equal(inlineSrcBlocks(source, script, template), source);
});

function position(code: string, marker: string) {
  const index = code.indexOf(marker);
  assert.notEqual(index, -1);
  const prefix = code.slice(0, index);
  return { line: prefix.split("\n").length, column: index - prefix.lastIndexOf("\n") - 1 };
}

function trace(module: MappedModule, marker: string) {
  assert.ok(module.map);
  return originalPositionFor(
    new TraceMap(JSON.stringify(module.map)),
    position(module.code, marker),
  );
}

void test("successive native byte ranges retain external and surviving SFC mappings", () => {
  const filename = "/project/App 雪.vue";
  const source =
    '<!-- 🐱 雪 -->\r\n<template src="./actual.html" />\r\n<script src="./actual.js" />\r\n<style>.tail { color: red }</style>';
  const map = new sources.OriginalSource(source, filename).map({ columns: true }) as SourceMapV3;
  const original = new MappedModule(source, map);
  const module = new MappedModule(source, map);
  inlineSrcBlock(module, parseSfc, "script", script, {
    filename: "/project/actual.js",
    sourceFilename: filename,
  });
  inlineSrcBlock(module, parseSfc, "template", template, {
    filename: "/project/actual.html",
    sourceFilename: filename,
  });
  assert.equal(
    module.code,
    `<!-- 🐱 雪 -->\r\n<template>\n${template}\n</template>\r\n<script>\n${script}\n</script>\r\n<style>.tail { color: red }</style>`,
  );
  assert.deepEqual(trace(module, "name:"), {
    source: "/project/actual.js",
    ...position(script, "name:"),
    name: null,
  });
  assert.deepEqual(trace(module, "<p>"), {
    source: "/project/actual.html",
    line: 1,
    column: 0,
    name: null,
  });
  assert.deepEqual(trace(module, ".tail"), trace(original, ".tail"));
  assert.ok(module.map!.sourcesContent?.includes(source));
  assert.ok(module.map!.sourcesContent?.includes(script));
  assert.ok(module.map!.sourcesContent?.includes(template));
});
