import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { build, type Rollup } from "vite";
import { parseSfc } from "@vizejs/native";
import {
  inlineSfcSrcBlocks,
  listSfcSrcRequests,
  type SfcSrcResolutions,
} from "../../shared/sfc-src.ts";
import { compileFile } from "./compiler.ts";
import { vize } from "./index.ts";
import { prepareSfcSrcRequests } from "./plugin/src-requests.ts";
import { getCompileOptionsForRequest } from "./plugin/state.ts";
import type { CompiledModule } from "./types.ts";

// The #11482 report: a package specifier in `<style src>`.
const SOURCE = `<template>
  <button class="foo">Foo</button>
</template>

<style src="fake-normalize/n.css"></style>
`;

function project(t: { after: (fn: () => void) => void }) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-src-package-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const write = (file: string, content: string) => {
    fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
    fs.writeFileSync(path.join(root, file), content);
  };
  write("node_modules/fake-normalize/package.json", '{"name":"fake-normalize","main":"n.css"}');
  write("node_modules/fake-normalize/n.css", "html { line-height: 1.15; }");
  write("src/styles/aliased.css", ".foo { color: navy; }");
  write("src/near.css", ".near { color: olive; }");
  write("src/Foo.vue", SOURCE);
  write("entry.js", 'export { default } from "./src/Foo.vue";\n');
  return { root, write, file: path.join(root, "src", "Foo.vue") };
}

/** `scan: false` skips the startup precompile, so `load` compiles on demand. */
async function viteBuild(
  root: string,
  scan = true,
  external: (string | RegExp)[] = [],
): Promise<string> {
  const result = (await build({
    root,
    configFile: false,
    logLevel: "silent",
    resolve: { alias: [{ find: "@", replacement: path.join(root, "src") }] },
    plugins: [vize({ configMode: false, ...(scan ? {} : { scanPatterns: [] }) })],
    build: {
      write: false,
      minify: false,
      lib: { entry: path.join(root, "entry.js"), formats: ["es"], fileName: "index" },
      rolldownOptions: { external: ["vue", ...external] },
    },
  })) as Rollup.RollupOutput | Rollup.RollupOutput[];
  return (Array.isArray(result) ? result : [result])
    .flatMap((item) => item.output)
    .map((item) => (item.type === "asset" ? String(item.source) : item.code))
    .join("\n");
}

void test("#11482 vite build resolves style src packages and aliases with Vite's resolver", async (t) => {
  const { root, write } = project(t);
  for (const scan of [true, false]) {
    write("src/Foo.vue", SOURCE);
    assert.match(await viteBuild(root, scan), /line-height: 1\.15/);
    write("src/Foo.vue", SOURCE.replace("fake-normalize/n.css", "@/styles/aliased.css"));
    assert.match(await viteBuild(root, scan), /color: navy/);
  }

  // Negative controls: Vite resolves neither, so the build fails loudly. A bare
  // file name next to the SFC is a module request, as with @vitejs/plugin-vue.
  for (const src of ["missing-normalize/n.css", "near.css"]) {
    write("src/Foo.vue", SOURCE.replace("fake-normalize/n.css", src));
    await assert.rejects(
      viteBuild(root, false),
      new RegExp(
        `<style src="${src.replace(/[./]/g, "\\$&")}"> not found: the bundler resolved no file`,
      ),
    );
  }
});

void test("#11493 vite build finds src blocks with the parser", async (t) => {
  const { root, write } = project(t);
  write(
    "src/Foo.vue",
    `<script setup>
const s = '<style src="./missing.css"></style>'
</script>
<template><button class="foo">{{ s }}</button></template>
<!-- <style src="./missing.css"></style> -->
<style src="./near.css" />
<style>.after { color: teal; }</style>
`,
  );
  for (const scan of [true, false]) {
    const output = await viteBuild(root, scan);
    assert.match(output, /color: olive/);
    assert.match(output, /color: teal/);
    assert.match(output, /<style src=\\?"\.\/missing\.css\\?"><\/style>/);
  }
});

void test("#11494 vite build keeps an external style src as an import", async (t) => {
  const { root, write } = project(t);
  for (const scan of [true, false]) {
    write("src/Foo.vue", SOURCE);
    const output = await viteBuild(root, scan, [/^fake-normalize\//]);
    assert.match(output, /import "fake-normalize\/n\.css";/);
    assert.doesNotMatch(output, /line-height/);
    // A script imports the component from it.
    write("src/Foo.vue", `<script src="fake-normalize/opts.js"></script>\n${SOURCE}`);
    assert.match(
      await viteBuild(root, scan, [/^fake-normalize\//]),
      /import \w+ from "fake-normalize\/opts\.js";/,
    );
    // Negative control: scoped styles need the file, which an external has not.
    write("src/Foo.vue", SOURCE.replace("<style src", "<style scoped src"));
    await assert.rejects(
      viteBuild(root, scan, [/^fake-normalize\//]),
      /<style scoped src="fake-normalize\/n\.css"> resolved to an external module; a scoped style needs its contents/,
    );
  }
});

void test("#11482 the synchronous compile reads module requests from prepared resolutions", async (t) => {
  const { root, file } = project(t);
  const css = path.join(root, "node_modules/fake-normalize/n.css");
  assert.deepEqual(
    listSfcSrcRequests(
      parseSfc,
      file,
      `${SOURCE}<script src="./a.ts"></script><style src="./b.css" />`,
    ),
    ["fake-normalize/n.css"],
  );

  const state = {
    isProduction: true,
    mergedOptions: {},
    fileCompilerOptions: undefined,
    viteBuildSourcemap: false,
    root,
    srcResolutions: new Map<string, SfcSrcResolutions>(),
  };
  const requests: string[] = [];
  await prepareSfcSrcRequests(
    state,
    async (request, importer) => {
      requests.push(`${request} from ${importer}`);
      return { id: `${css}?direct` };
    },
    file,
    SOURCE,
  );
  assert.deepEqual(requests, [`fake-normalize/n.css from ${file}`]);
  const compiled = compileFile(
    file,
    new Map<string, CompiledModule>(),
    getCompileOptionsForRequest(state, false, file),
    SOURCE,
  );
  assert.match(compiled.css ?? "", /line-height: 1\.15/);
  assert.deepEqual(compiled.dependencies, [css]);

  // Relative paths need no resolution; unresolved requests still fail.
  assert.match(
    inlineSfcSrcBlocks(parseSfc, file, '<style src="./near.css"></style>').source,
    /color: olive/,
  );
  assert.throws(
    () => inlineSfcSrcBlocks(parseSfc, file, SOURCE),
    /<style src="fake-normalize\/n\.css"> not found: the bundler resolved no file/,
  );
});

void test("#11528 an on-demand load reads its SFC once", async (t) => {
  const { root, file } = project(t);
  let reads = 0;
  const { readFileSync } = fs;
  fs.readFileSync = ((target: fs.PathOrFileDescriptor, ...rest: never[]) => {
    if (target === file) reads++;
    return readFileSync(target, ...rest);
  }) as typeof fs.readFileSync;
  try {
    // The `src` request resolved from that one read still reaches the compile.
    assert.match(await viteBuild(root, false), /line-height: 1\.15/);
  } finally {
    fs.readFileSync = readFileSync;
  }
  assert.equal(reads, 1, "the src request scan and the compile share one read");
});
