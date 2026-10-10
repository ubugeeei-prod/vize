import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "../..");
const dist = resolve(root, "docs/dist");
const sourceSha = execFileSync("git", ["rev-parse", "HEAD"], {
  cwd: root,
  encoding: "utf8",
}).trim();
const manifest = JSON.parse(readFileSync(resolve(dist, "_og/manifest.json"), "utf8")) as {
  sourceSha: string;
};
assert.equal(
  manifest.sourceSha,
  sourceSha,
  "Configuration capture requires this exact built source",
);
const original = JSON.parse(
  readFileSync(resolve(root, "tests/tooling/fixtures/docs-config-journey.json"), "utf8"),
) as { documents: { file: string; headings: string[] }[] };
for (const document of original.documents) {
  const route = document.file.replace(/^docs\/content\//u, "").replace(/\.md$/u, "/index.html");
  const html = readFileSync(resolve(dist, route), "utf8");
  for (const id of document.headings)
    assert.ok(html.includes(`id="${id}"`), `${route}: original #${id} survives SSG`);
}
const pages = [
  "/getting-started",
  "/guide/configuration",
  "/guide/configuration-reference",
  "/guide/compiler-configuration-reference",
  "/guide/vite-plugin",
  "/guide/workflows",
];
const routes = ["", "/ja", "/fr", "/pt-BR", "/zh-CN"].flatMap((locale) =>
  pages.map((page) => `${locale}${page}`),
);
const rendered = spawnSync(
  process.execPath,
  [
    resolve(root, "docs/scripts/verify-navigation-render.ts"),
    "--dir",
    dist,
    "--routes",
    routes.join(","),
    "--output",
    resolve(root, "docs-render-evidence/config-journey"),
  ],
  { cwd: root, stdio: "inherit" },
);
assert.ifError(rendered.error);
assert.equal(rendered.status, 0, "All five localized configuration reading paths render correctly");
