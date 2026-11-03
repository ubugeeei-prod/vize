// #11522: a `<style src>` file in another directory resolves its relative
// `@import`, `@use` and `url()` from its own directory, as with
// @vitejs/plugin-vue, whose output for this fixture is the expected one.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { build, type Rollup } from "vite";
import { vize } from "./index.ts";

const require = createRequire(import.meta.url);
// A sass stand-in that resolves `@use` from the importing file's `url`, as Sass does.
const { installStandIns } = require("../../shared/test-preprocessors.cjs") as {
  installStandIns: (root: string) => void;
};

const SVG = '<svg xmlns="http://www.w3.org/2000/svg"/>\n';
const DATA_URI =
  /url\("data:image\/svg\+xml,%3csvg%20xmlns='http:\/\/www\.w3\.org\/2000\/svg'\/%3e"\)/;

function project(t: { after: (fn: () => void) => void }, files: Record<string, string>) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-style-src-relative-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const all: Record<string, string> = {
    "entry.js": 'export { default } from "./src/components/Foo.vue";\n',
    "src/styles/b.css": ".b { color: blue; }\n",
    "src/styles/img.svg": SVG,
    "src/styles/_vars.scss": "$c: green;\n",
    ...files,
  };
  for (const [file, content] of Object.entries(all)) {
    fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
    fs.writeFileSync(path.join(root, file), content);
  }
  installStandIns(root);
  return root;
}

async function viteBuild(root: string): Promise<{ css: string; js: string; warnings: string[] }> {
  const warnings: string[] = [];
  const result = (await build({
    root,
    configFile: false,
    logLevel: "silent",
    customLogger: {
      info() {},
      warn: (message: string) => warnings.push(message),
      warnOnce: (message: string) => warnings.push(message),
      error() {},
      clearScreen() {},
      hasErrorLogged: () => false,
      hasWarned: false,
    },
    plugins: [vize({ configMode: false })],
    build: {
      write: false,
      minify: false,
      cssCodeSplit: false,
      lib: { entry: path.join(root, "entry.js"), formats: ["es"], fileName: "index" },
      rolldownOptions: { external: ["vue"] },
    },
  })) as Rollup.RollupOutput | Rollup.RollupOutput[];
  const output = (Array.isArray(result) ? result : [result]).flatMap((item) => item.output);
  const text = (item: (typeof output)[number]) =>
    item.type === "asset" ? String(item.source) : item.code;
  return {
    css: output
      .filter((item) => item.fileName.endsWith(".css"))
      .map(text)
      .join("\n"),
    js: output
      .filter((item) => item.type === "chunk")
      .map(text)
      .join("\n"),
    warnings,
  };
}

void test("#11522 vite build resolves a style src file's relative references from its directory", async (t) => {
  const root = project(t, {
    "src/styles/a.css": '@import "./b.css";\n.a { background: url(./img.svg); }\n',
    "src/styles/scoped.css": ".c { mask: url(./img.svg); }\n",
    "src/styles/s.scss": '@use "./vars" as v;\n.s { color: v.$c; background: url(./img.svg); }\n',
    "src/styles/m.css": ".m { background: url(./img.svg); }\n",
    "src/components/Foo.vue": `<template><p :class="['a', 'b', 'c', 's', $style.m]">x</p></template>
<style src="../styles/a.css"></style>
<style scoped src="../styles/scoped.css"></style>
<style scoped lang="scss" src="../styles/s.scss"></style>
<style module src="../styles/m.css"></style>
`,
  });
  const { css, js, warnings } = await viteBuild(root);
  t.diagnostic(JSON.stringify({ css, js, warnings }));
  assert.deepEqual(warnings, []);
  assert.match(css, /\.b\s*\{\s*color: blue;\s*\}/);
  assert.match(css, new RegExp(`\\.a\\s*\\{\\s*background: ${DATA_URI.source};\\s*\\}`));
  const scope = /\.c\[(data-v-[0-9a-f]{8})\]/.exec(css)?.[1];
  assert.ok(scope, css);
  assert.match(css, new RegExp(`\\.c\\[${scope}\\]\\s*\\{\\s*mask: ${DATA_URI.source};\\s*\\}`));
  assert.match(
    css,
    new RegExp(
      `\\.s\\[${scope}\\]\\s*\\{\\s*color: green;\\s*background: ${DATA_URI.source};\\s*\\}`,
    ),
  );
  // Vite's CSS Modules map, which replaces the compiler's provisional `__cssModules`.
  const local = /\{ m: "([\w-]+)" \}/.exec(js)?.[1];
  assert.ok(local, js);
  assert.match(css, new RegExp(`\\.${local}\\s*\\{\\s*background: ${DATA_URI.source};\\s*\\}`));
  assert.doesNotMatch(css, /url\(\.\/img\.svg\)/);
});

void test("#11522 an inline block still resolves from the SFC, and a missing src import fails", async (t) => {
  const root = project(t, {
    "src/components/local.css": ".l { color: teal; }\n",
    "src/components/Foo.vue": `<template><p class="l">x</p></template>
<style>@import "./local.css";</style>
`,
  });
  assert.match((await viteBuild(root)).css, /\.l\s*\{\s*color: teal;\s*\}/);

  // Negative control: the import is missing next to the src file too.
  fs.writeFileSync(path.join(root, "src/styles/a.css"), '@import "./missing.css";\n');
  fs.writeFileSync(
    path.join(root, "src/components/Foo.vue"),
    '<template><p/></template>\n<style src="../styles/a.css"></style>\n',
  );
  await assert.rejects(viteBuild(root), /missing\.css/);
});
