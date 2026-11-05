import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { build } from "esbuild";
import vizeEsbuild from "./esbuild.ts";
import { vizeUnplugin } from "./unplugin.ts";

// The #11482 report: a package specifier in `<style src>`.
const SOURCE = `<template>
  <button class="foo">Foo</button>
</template>

<style src="fake-normalize/n.css"></style>
`;

function project(t: { after: (fn: () => void) => void }) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-src-requests-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const css = path.join(root, "node_modules", "fake-normalize", "n.css");
  fs.mkdirSync(path.dirname(css), { recursive: true });
  fs.writeFileSync(path.join(path.dirname(css), "package.json"), '{"name":"fake-normalize"}');
  fs.writeFileSync(css, "html { line-height: 1.15; }");
  fs.mkdirSync(path.join(root, "src"));
  const file = path.join(root, "src", "Foo.vue");
  fs.writeFileSync(file, SOURCE);
  fs.writeFileSync(path.join(root, "src", "entry.js"), 'export { default } from "./Foo.vue";\n');
  return { root, css, file };
}

function createHost(context: object) {
  const meta = { framework: "rollup" } as Parameters<typeof vizeUnplugin.raw>[1];
  const hooks = vizeUnplugin.raw({ isProduction: true }, meta) as unknown as {
    transform: (this: unknown, code: string, id: string) => Promise<{ code: string } | null>;
    load: (this: unknown, id: string) => Promise<{ code: string } | null>;
  };
  const watched: string[] = [];
  const ctx = { warn() {}, addWatchFile: (id: string) => watched.push(id), ...context };
  return { hooks, ctx, watched };
}

async function transform(context: object, file: string): Promise<string> {
  const { hooks: plugin, ctx, watched } = createHost(context);
  const result = await plugin.transform.call(ctx, SOURCE, file);
  assert.ok(result);
  // The block is a style module; its content is what the host's CSS pipeline gets.
  const styleId = /^import "([^"]+\?vue&type=style[^"]*)";/m.exec(result.code)?.[1];
  assert.ok(styleId, result.code);
  const style = await plugin.load.call(ctx, styleId);
  return `${style?.code}\n/* watched: ${watched.join(",")} */`;
}

void test("#11482 style src package specifiers resolve with the host's resolver", async (t) => {
  const { css, file } = project(t);
  const requests: string[] = [];
  const rollupLike = {
    async resolve(request: string, importer: string) {
      requests.push(`${request} from ${importer}`);
      return request === "fake-normalize/n.css" ? { id: `${css}?inline` } : null;
    },
  };
  const webpackLike = {
    getNativeBuildContext: () => ({
      framework: "webpack",
      loaderContext: {
        getResolve: () => async (context: string, request: string) => {
          assert.equal(context, path.dirname(file));
          if (request !== "fake-normalize/n.css") throw new Error("Module not found");
          return css;
        },
      },
    }),
  };
  for (const context of [rollupLike, webpackLike]) {
    const code = await transform(context, file);
    assert.match(code, /line-height: 1\.15/);
    assert.match(code, new RegExp(`watched: ${css.replace(/[.*+?^${}()|[\]\\/]/g, "\\$&")}`));
  }
  assert.deepEqual(requests, [`fake-normalize/n.css from ${file}`]);

  // #11494: a module the bundler keeps external stays an import, not a style module.
  const host = createHost({
    resolve: async (request: string) => ({ id: request, external: true }),
  });
  const external = (await host.hooks.transform.call(host.ctx, SOURCE, file))?.code ?? "";
  assert.match(external, /import "fake-normalize\/n\.css";/);
  assert.doesNotMatch(external, /\?vue&type=style|line-height/);

  // Negative controls: an unresolved or resolver-less request fails loudly.
  const unresolved = [{ resolve: async () => null }, {}];
  for (const context of unresolved) {
    await assert.rejects(
      transform(context, file),
      /<style src="fake-normalize\/n\.css"> not found: the bundler resolved no file/,
    );
  }
});

void test("#11482 esbuild resolves a style src package specifier", async (t) => {
  const { root } = project(t);
  const result = await build({
    entryPoints: [path.join(root, "src", "entry.js")],
    bundle: true,
    write: false,
    format: "esm",
    outdir: path.join(root, "dist"),
    external: ["vue"],
    plugins: [vizeEsbuild({ isProduction: true })],
  });
  const css = result.outputFiles.find((file) => file.path.endsWith(".css"))?.text ?? "";
  assert.match(css, /line-height: 1\.15/);

  // #11494: an external package stays an import.
  const external = await build({
    entryPoints: [path.join(root, "src", "entry.js")],
    bundle: true,
    write: false,
    format: "esm",
    external: ["vue", "fake-normalize"],
    plugins: [vizeEsbuild({ isProduction: true })],
  });
  const externalJs = external.outputFiles.map((file) => file.text).join("\n");
  assert.match(externalJs, /import "fake-normalize\/n\.css";/);
  assert.doesNotMatch(externalJs, /line-height/);

  fs.writeFileSync(
    path.join(root, "src", "Foo.vue"),
    SOURCE.replace("fake-normalize/n.css", "missing-normalize/n.css"),
  );
  await assert.rejects(
    build({
      entryPoints: [path.join(root, "src", "entry.js")],
      bundle: true,
      write: false,
      logLevel: "silent",
      external: ["vue"],
      plugins: [vizeEsbuild({ isProduction: true })],
    }),
    /<style src="missing-normalize\/n\.css"> not found: the bundler resolved no file/,
  );
});
