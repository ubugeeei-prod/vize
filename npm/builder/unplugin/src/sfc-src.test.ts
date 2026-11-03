import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parseSfc } from "@vizejs/native";
import {
  assertSfcStyleSrcImports,
  inlineSfcSrcBlocks,
  listSfcSrcRequests,
  type SfcSrcResolution,
} from "../../shared/sfc-src.ts";

void test("#11456 src blocks are inlined at their parsed positions", (t) => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-src-blocks-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.mkdirSync(path.join(root, "styles"));
  fs.writeFileSync(path.join(root, "a.css"), ".a{color:red}");
  fs.writeFileSync(path.join(root, "styles", "b.scss"), "$c: blue; .b{color:$c}");
  fs.writeFileSync(path.join(root, "t.html"), "<p>é</p>");
  fs.writeFileSync(path.join(root, "s.js"), "export default { name: 'S' }");
  const file = path.join(root, "Foo.vue");
  const at = (name: string) => path.join(root, name);

  const cases: Array<{
    id: string;
    source: string;
    inlined: string;
    dependencies: string[];
    /** Style block index -> the file it came from (#11522). */
    styleSrcs: [number, string][];
  }> = [
    {
      id: "scoped",
      source: '<template><b/></template>\n<style scoped src="./a.css"></style>',
      inlined: "<template><b/></template>\n<style scoped>\n.a{color:red}\n</style>",
      dependencies: [at("a.css")],
      styleSrcs: [[0, at("a.css")]],
    },
    {
      id: "self-closing, lang, module, single quotes and uppercase",
      source: "<style SRC='./styles/b.scss' lang=\"scss\" module />\n<style>.z{}</style>",
      inlined: '<style lang="scss" module>\n$c: blue; .b{color:$c}\n</style>\n<style>.z{}</style>',
      dependencies: [at("styles/b.scss")],
      styleSrcs: [[0, at("styles/b.scss")]],
    },
    {
      id: "script, template and two styles keep source order and UTF-8 offsets",
      source:
        '<script src="./s.js"></script>\n<template src="./t.html"></template>\n<style src="./a.css"></style>\n<style>.é{}</style>\n<style scoped src="./a.css"></style>',
      inlined:
        "<script>\nexport default { name: 'S' }\n</script>\n<template>\n<p>é</p>\n</template>\n<style>\n.a{color:red}\n</style>\n<style>.é{}</style>\n<style scoped>\n.a{color:red}\n</style>",
      dependencies: [at("t.html"), at("s.js"), at("a.css"), at("a.css")],
      // The inline middle block is index 1.
      styleSrcs: [
        [0, at("a.css")],
        [2, at("a.css")],
      ],
    },
  ];
  for (const item of cases) {
    assert.deepEqual(
      inlineSfcSrcBlocks(parseSfc, file, item.source),
      {
        source: item.inlined,
        dependencies: item.dependencies,
        styleSrcs: new Map(item.styleSrcs),
        imports: [],
      },
      item.id,
    );
  }

  // Not src blocks, or malformed (left to the compiler): unchanged, nothing read.
  for (const source of [
    '<template><img src="./missing.png"></template><style>.a{}</style>',
    '<template><b/></template><style data-src="./missing.css">.a{}</style>',
    "<script setup>const s = '<style src=\"./missing.css\"></style>'</script>",
    '<template><b/></template><!-- <style src="./missing.css"></style> -->',
    '<template><b/></template>\n<style src="./missing.css">',
  ]) {
    assert.deepEqual(
      inlineSfcSrcBlocks(parseSfc, file, source),
      { source, dependencies: [], styleSrcs: new Map(), imports: [] },
      source,
    );
  }

  assert.throws(
    () => inlineSfcSrcBlocks(parseSfc, file, '<style scoped src="./missing.css"></style>'),
    /<style src="\.\/missing\.css"> not found \(resolved: .*missing\.css\)/,
  );
  assert.throws(
    () => inlineSfcSrcBlocks(parseSfc, file, '<style src=""></style>'),
    /<style src=""> not found/,
  );
});

void test("#11482 src module requests come from the bundler's resolutions", (t) => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-src-request-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.mkdirSync(path.join(root, "src"));
  fs.writeFileSync(path.join(root, "n.css"), "html{line-height:1.15}");
  fs.writeFileSync(path.join(root, "src", "near.css"), ".near{color:olive}");
  const file = path.join(root, "src", "Foo.vue");
  const inline = (src: string, resolutions?: Map<string, SfcSrcResolution>) =>
    inlineSfcSrcBlocks(parseSfc, file, `<style src="${src}"></style>`, resolutions);
  const resolutions = new Map([
    ["fake-normalize/n.css", { file: path.join(root, "n.css") }],
    [path.join(root, "n.css"), { file: path.join(root, "n.css") }],
  ]);

  assert.deepEqual(inline("fake-normalize/n.css", resolutions), {
    source: "<style>\nhtml{line-height:1.15}\n</style>",
    dependencies: [path.join(root, "n.css")],
    styleSrcs: new Map([[0, path.join(root, "n.css")]]),
    imports: [],
  });
  // Absolute paths are module requests too.
  assert.match(inline(path.join(root, "n.css"), resolutions).source, /line-height/);
  // Relative paths never need the bundler.
  assert.match(inline("./near.css").source, /color:olive/);

  // Negative controls: a request the bundler did not resolve fails the build,
  // including a bare file name next to the SFC, as with @vitejs/plugin-vue.
  for (const src of ["fake-normalize/n.css", "near.css", "missing/x.css"]) {
    assert.throws(
      () => inline(src, new Map()),
      new RegExp(
        `<style src="${src.replace(/[./]/g, "\\$&")}"> not found: the bundler resolved no file`,
      ),
      src,
    );
  }
  assert.deepEqual(
    listSfcSrcRequests(parseSfc, file, '<style src="a.css"></style><style src="./b.css" />'),
    ["a.css"],
  );
});

void test("#11494 an external src module stays an import", () => {
  const file = "/project/src/Foo.vue";
  const external = new Map([["sibling-pkg/x.css", { external: true } as const]]);
  const ext = (source: string) =>
    inlineSfcSrcBlocks(
      parseSfc,
      file,
      source,
      new Map([...external, ["sibling-pkg/x.js", { external: true }]]),
    );

  // A style is left for the output to import; a script imports the module.
  for (const source of [
    '<template><b/></template>\n<style src="sibling-pkg/x.css"></style>',
    '<template><b/></template>\n<style lang="scss" src="sibling-pkg/x.css" />',
  ]) {
    assert.deepEqual(ext(source), {
      source,
      dependencies: [],
      styleSrcs: new Map(),
      imports: ["sibling-pkg/x.css"],
    });
  }
  assert.deepEqual(ext('<script lang="ts" src="sibling-pkg/x.js"></script>'), {
    source:
      '<script lang="ts">\nimport __vize_src_main__ from "sibling-pkg/x.js";\nexport * from "sibling-pkg/x.js";\nexport default __vize_src_main__;\n</script>',
    dependencies: [],
    styleSrcs: new Map(),
    imports: [],
  });

  // Negative controls: blocks compiled into the component need the file.
  for (const [source, reason] of [
    ['<style scoped src="sibling-pkg/x.css"></style>', "a scoped style needs its contents"],
    ['<style module src="sibling-pkg/x.css"></style>', "a CSS module needs its contents"],
    ['<template src="sibling-pkg/x.css"></template>', "the template is compiled into"],
    ['<script setup src="sibling-pkg/x.js"></script>', "<script setup> is compiled into"],
  ]) {
    assert.throws(
      () => ext(source),
      (error: Error) => {
        assert.match(error.message, /resolved to an external module/, source);
        assert.ok(error.message.includes(reason), error.message);
        return true;
      },
    );
  }

  // Only an external style may stay uninlined.
  assertSfcStyleSrcImports(
    file,
    [{ src: "sibling-pkg/x.css" }, { src: null }],
    ["sibling-pkg/x.css"],
  );
  assert.throws(
    () => assertSfcStyleSrcImports(file, [{ src: "./a.css" }], ["sibling-pkg/x.css"]),
    /<style src="\.\/a\.css"> could not be inlined/,
  );
});
