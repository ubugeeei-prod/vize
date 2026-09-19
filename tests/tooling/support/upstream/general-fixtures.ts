import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fixtureRoot } from "./vue-language-tools.ts";

/** The upstream suite pins Vue independently from this repository's app fixtures. */
export function prepareGeneralFixtures(directory: string): void {
  const require = createRequire(import.meta.url);
  const root = path.join(directory, "test-workspace");
  fs.cpSync(path.join(fixtureRoot, "upstream/test-workspace"), root, { recursive: true });
  const modules = path.join(root, "node_modules");
  fs.mkdirSync(modules);
  const vue = path.dirname(require.resolve("vue-language-tools-fixture-vue/package.json"));
  const helpers = path.dirname(require.resolve("vue-component-type-helpers/package.json"));
  assertVersion(vue, "3.6.0-rc.6");
  assertVersion(helpers, "3.3.11");
  for (const [name, target] of [
    ["vue", vue],
    ["@vue", path.join(path.dirname(vue), "@vue")],
    ["vue-component-type-helpers", helpers],
  ]) {
    fs.symlinkSync(
      target,
      path.join(modules, name),
      process.platform === "win32" ? "junction" : "dir",
    );
  }
}

function assertVersion(directory: string, expected: string): void {
  const actual = JSON.parse(fs.readFileSync(path.join(directory, "package.json"), "utf8")).version;
  if (actual !== expected)
    throw new Error(`Upstream fixture dependency drift: ${actual} != ${expected}`);
}
