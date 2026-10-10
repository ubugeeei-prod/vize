import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "../..");
const manifest = JSON.parse(readFileSync(resolve(root, "docs/dist/_og/manifest.json"), "utf8")) as {
  sourceSha: string;
};
const source = execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
assert.equal(manifest.sourceSha, source, "browser evidence belongs to the exact source build");
const routes = [
  "/stability",
  "/ja/stability",
  "/fr/stability",
  "/pt-BR/stability",
  "/zh-CN/stability",
  "/ja/guide/analysis-diagnostics",
  "/ja/guide/comment-annotations",
  "/ja/guide/oxlint",
  "/ja/guide/unplugin",
  "/ja/guide/workflows",
  "/ja/integrations/vscode",
];
const result = spawnSync(
  process.execPath,
  [
    "docs/scripts/verify-navigation-render.ts",
    "--routes",
    routes.join(","),
    "--output",
    "docs-render-evidence/ja-editorial",
  ],
  { cwd: root, stdio: "inherit" },
);
assert.equal(result.status, 0, "all reviewed reading paths render and their links resolve");
