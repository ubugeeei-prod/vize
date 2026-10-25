import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { build, createServer, type InlineConfig, type Plugin, type Rollup } from "vite";

import { vize } from "./index.ts";

const require = createRequire(import.meta.url);
const componentPath = "/src/components/Card 雪.vue";
const assetBodies = ["direct", "mask", "nested"].map((name) => ({
  name,
  bytes: Buffer.from(
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 2 2"><path id="${name}" d="M0 0h2v2H0z"/></svg>\n`,
  ),
}));

interface Artifact {
  fileName: string;
  sha256: string;
  body: string;
}

interface Built {
  css: Artifact[];
  js: Artifact[];
  assets: Artifact[];
  styleImports: string[];
  scopeIds: string[];
  html: string;
}

function digest(bytes: string | Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}

function fixture(root: string, scoped: boolean): void {
  for (const directory of ["components", "styles/nested", "assets"]) {
    fs.mkdirSync(path.join(root, "src", directory), { recursive: true });
  }
  fs.writeFileSync(path.join(root, "entry.js"), `export { default } from ".${componentPath}";\n`);
  fs.writeFileSync(
    path.join(root, componentPath.slice(1)),
    `<template><p class="card local nested">Card</p></template>
<style${scoped ? " scoped" : ""}>
@import "../styles/colors.css";
.local { --relative-local: local-origin; padding: 7px; background-image: url("../assets/direct.svg"); mask-image: url("../assets/mask.svg"); }
</style>
`,
  );
  fs.writeFileSync(
    path.join(root, "src/styles/colors.css"),
    '@import "./nested/spacing.css";\n.card { --relative-colors: colors-origin; color: tomato; }\n',
  );
  fs.writeFileSync(
    path.join(root, "src/styles/nested/spacing.css"),
    '.nested { --relative-chain: nested-origin; margin-left: 17px; border-image-source: url("../../assets/nested.svg"); }\n',
  );
  for (const { name, bytes } of assetBodies) {
    fs.writeFileSync(path.join(root, "src/assets", `${name}.svg`), bytes);
  }
  assert.equal(fs.existsSync(path.join(root, "node_modules")), false);
}

function styleImports(code: string): string[] {
  return [...code.matchAll(/\bimport\s*["']([^"'\n]+\.__vize_style_[^"'\n]+)["']/gu)].map(
    (match) => match[1],
  );
}

function cssUrls(css: string): string[] {
  return [...css.matchAll(/url\(\s*(?:"([^"]*)"|'([^']*)'|([^\s)]*))\s*\)/gu)].map(
    (match) => match[1] ?? match[2] ?? match[3],
  );
}

function assertCss(css: string, scopeIds: string[]): void {
  assert.match(css, /--relative-colors:\s*colors-origin\b/u);
  assert.match(css, /--relative-chain:\s*nested-origin\b/u);
  assert.match(css, /--relative-local:\s*local-origin\b/u);
  assert.match(css, /color:\s*(?:tomato|#ff6347)\b/u);
  assert.match(css, /margin-left:\s*17px\b/u);
  assert.match(css, /padding:\s*7px\b/u);
  assert.doesNotMatch(css, /@import\b/u);
  if (scopeIds.length === 1) {
    assert.ok(css.includes(`.local[${scopeIds[0]}]`), css);
  } else {
    assert.match(css, /\.local\s*\{/u);
    assert.doesNotMatch(css, /data-v-/u);
  }
}

async function buildFixture(
  root: string,
  transformer: "postcss" | "lightningcss",
  scoped: boolean,
  ssr: boolean,
): Promise<Built> {
  const imports = new Set<string>();
  const observeImports: Plugin = {
    name: "test:observe-stable-style-imports",
    enforce: "post",
    transform(code, id) {
      if (!id.includes("type=style")) {
        for (const imported of styleImports(code)) imports.add(imported);
      }
      return null;
    },
  };
  const entry = path.join(root, "entry.js");
  const config: InlineConfig = {
    root,
    configFile: false,
    logLevel: "silent",
    plugins: [vize({ configMode: false }), observeImports],
    css: { transformer },
    build: {
      write: false,
      minify: true,
      cssMinify: false,
      cssCodeSplit: false,
      assetsInlineLimit: 0,
      ...(ssr ? { ssr: entry, ssrEmitAssets: true } : {}),
      rollupOptions: {
        input: entry,
        preserveEntrySignatures: "strict",
        external: [/^(?:@vue\/|vue(?:\/|$))/u],
      },
    },
  };
  const result = (await build(config)) as Rollup.RollupOutput | Rollup.RollupOutput[];
  const output = (Array.isArray(result) ? result : [result]).flatMap((item) => item.output);
  const artifacts = output.map((item) => {
    const bytes = item.type === "chunk" ? Buffer.from(item.code) : Buffer.from(item.source);
    return { fileName: item.fileName, sha256: digest(bytes), body: bytes.toString() };
  });
  const css = artifacts.filter((item) => item.fileName.endsWith(".css"));
  const js = artifacts.filter((item) => /\.[cm]?js$/u.test(item.fileName));
  const assets = artifacts.filter((item) => item.fileName.endsWith(".svg"));
  assert.equal(css.length, 1);
  assert.equal(js.length, 1);
  assert.equal(assets.length, assetBodies.length);
  for (const { bytes } of assetBodies) {
    assert.equal(assets.filter((asset) => asset.sha256 === digest(bytes)).length, 1);
  }
  const cssBody = css[0].body;
  assert.deepEqual(
    [...new Set(cssUrls(cssBody))].map((url) => decodeURIComponent(url).replace(/^\//u, "")).sort(),
    assets.map((asset) => asset.fileName).sort(),
    "every CSS URL must point to exactly one emitted, byte-checked asset",
  );
  const scopeIds = [...new Set(js[0].body.match(/data-v-[0-9a-f]{8}/gu) ?? [])];
  assert.equal(scopeIds.length, scoped ? 1 : 0, js[0].body);
  assertCss(cssBody, scopeIds);
  for (const artifact of [...css, ...js]) {
    assert.equal(
      artifact.body.includes(root),
      false,
      "shipped artifacts cannot contain the checkout",
    );
    assert.equal(artifact.body.includes(encodeURIComponent(root)), false);
  }
  assert.equal(imports.size, 1, "observe the actual SFC producer import before CSS resolution");
  for (const imported of imports) {
    const query = new URLSearchParams(imported.slice(imported.indexOf("?") + 1));
    assert.equal(decodeURIComponent(imported.split("?")[0]), `${componentPath}.__vize_style_0.css`);
    assert.equal(query.has("vize-root-relative"), true);
    assert.equal(query.get("vize-file"), componentPath);
    assert.equal(query.get("index"), "0");
    assert.equal(query.get("type"), "style");
    assert.equal(query.has("scoped"), scoped);
    if (scoped) assert.equal(query.get("scoped"), scopeIds[0]);
  }

  const vueRuntime = require.resolve("vue");
  const serverRenderer = require.resolve("vue/server-renderer");
  const executable = js[0].body
    .replace(
      /from\s*["'](?:@vue\/server-renderer|vue\/server-renderer)["']/gu,
      `from ${JSON.stringify(pathToFileURL(serverRenderer).href)}`,
    )
    .replace(/from\s*["']vue["']/gu, `from ${JSON.stringify(pathToFileURL(vueRuntime).href)}`);
  const moduleFile = path.join(root, `${transformer}-${scoped}-${ssr}.mjs`);
  fs.writeFileSync(moduleFile, executable);
  const component = (await import(pathToFileURL(moduleFile).href)).default;
  const { createSSRApp } = await import(pathToFileURL(vueRuntime).href);
  const { renderToString } = await import(pathToFileURL(serverRenderer).href);
  const html = await renderToString(createSSRApp(component));
  assert.equal(html, `<p class="card local nested"${scoped ? ` ${scopeIds[0]}` : ""}>Card</p>`);
  return { css, js, assets, styleImports: [...imports], scopeIds, html };
}

async function devFixture(root: string, transformer: "postcss" | "lightningcss", scoped: boolean) {
  const server = await createServer({
    root,
    configFile: false,
    logLevel: "silent",
    plugins: [vize({ configMode: false })],
    css: { transformer },
    build: { assetsInlineLimit: 0 },
    resolve: { alias: [{ find: /^vue$/u, replacement: require.resolve("vue") }] },
    optimizeDeps: { noDiscovery: true, include: [] },
    server: { host: "127.0.0.1", port: 0 },
  });
  try {
    await server.listen();
    const address = server.httpServer?.address();
    assert.ok(address && typeof address === "object");
    const origin = `http://127.0.0.1:${address.port}`;
    const component = await fetch(`${origin}${encodeURI(componentPath)}`);
    assert.equal(component.status, 200);
    const imports = styleImports(await component.text());
    assert.equal(imports.length, 1);
    const response = await fetch(`${origin}${imports[0]}&direct`);
    assert.equal(response.status, 200);
    assert.match(response.headers.get("content-type") ?? "", /text\/css/u);
    const css = await response.text();
    const scopeIds = [...new Set(css.match(/data-v-[0-9a-f]{8}/gu) ?? [])];
    assert.equal(scopeIds.length, scoped ? 1 : 0);
    assertCss(css, scopeIds);
    const assets = [];
    for (const url of new Set(cssUrls(css))) {
      const asset = await fetch(new URL(url, origin));
      assert.equal(asset.status, 200, url);
      const bytes = Buffer.from(await asset.arrayBuffer());
      assert.equal(assetBodies.filter((body) => body.bytes.equals(bytes)).length, 1, url);
      assets.push({ url, sha256: digest(bytes) });
    }
    assert.equal(assets.length, assetBodies.length);
    assert.equal(new Set(assets.map((asset) => asset.sha256)).size, assetBodies.length);
    return { css, scopeIds, assets };
  } finally {
    await server.close();
  }
}

void test(
  "real Vite preserves relative CSS origins and stable production identities",
  {
    skip:
      process.env.VIZE_STYLE_RELATIVE_MATRIX !== "1"
        ? "dedicated source-native workflow installs the isolated CSS transformer host"
        : false,
    timeout: 120_000,
  },
  async (t) => {
    const viteRequire = createRequire(require.resolve("vite"));
    const lightningEntry = fs.realpathSync(viteRequire.resolve("lightningcss"));
    const lightningPackage = JSON.parse(
      fs.readFileSync(path.resolve(path.dirname(lightningEntry), "../package.json"), "utf8"),
    );
    assert.equal(lightningPackage.version, "1.33.0", "use the isolated pinned CSS host");
    const parent = fs.mkdtempSync(path.join(os.tmpdir(), "vize-relative-style-10659-"));
    t.after(() => fs.rmSync(parent, { recursive: true, force: true }));
    const samples = [];
    for (const transformer of ["postcss", "lightningcss"] as const) {
      for (const scoped of [false, true]) {
        await t.test(
          `${transformer}, ${scoped ? "scoped" : "unscoped"}, client/SSR/dev`,
          async () => {
            const checkouts = [];
            for (const directory of [
              "checkout a 雪/app",
              "checkout b/deeper/monorepo/packages/site 雪",
            ]) {
              const root = path.join(parent, `${transformer}-${scoped}`, directory);
              fixture(root, scoped);
              const client = await buildFixture(root, transformer, scoped, false);
              const ssr = await buildFixture(root, transformer, scoped, true);
              assert.deepEqual(
                ssr.css,
                client.css,
                "SSR and client CSS use the same physical origin",
              );
              assert.deepEqual(ssr.assets, client.assets);
              assert.deepEqual(ssr.scopeIds, client.scopeIds);
              const dev = await devFixture(root, transformer, scoped);
              checkouts.push({ client, ssr });
              samples.push({ transformer, scoped, root, client, ssr, dev });
            }
            assert.deepEqual(
              checkouts[1],
              checkouts[0],
              "checkout/subroot paths cannot alter shipped identities",
            );
          },
        );
      }
    }
    if (process.env.VIZE_STYLE_RELATIVE_IMPORTS_REPORT) {
      fs.writeFileSync(
        process.env.VIZE_STYLE_RELATIVE_IMPORTS_REPORT,
        JSON.stringify(
          {
            node: process.version,
            vite: require("vite/package.json").version,
            vue: require("vue/package.json").version,
            lightningcss: { version: lightningPackage.version, entry: lightningEntry },
            samples,
          },
          null,
          2,
        ),
      );
    }
  },
);
