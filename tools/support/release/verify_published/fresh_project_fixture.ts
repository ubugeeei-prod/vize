import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { isAbsolute, join, relative, sep } from "node:path";

export const APP = `<script setup lang="ts">
const items: number[] = [1, 2]
</script>

<template>
  <ul><li v-for="item in items">{{ item }}</li></ul>
</template>
`;
export const BROKEN_APP = APP.replace("[1, 2]", '[1, "broken"]');
export const PLANNED_DEPENDENCIES = ["@vizejs/vite-plugin", "vize"] as const;

export const sha256 = (text: string): string =>
  createHash("sha256").update(text, "utf8").digest("hex");

export function writeJson(path: string, value: unknown): void {
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
}

export function writeProject(project: string): void {
  mkdirSync(join(project, "src"), { recursive: true });
  writeJson(join(project, "package.json"), {
    name: "vize-published-fresh-project",
    version: "0.0.0",
    private: true,
    type: "module",
    scripts: { build: "vite build" },
    dependencies: { vue: "3.5.34" },
    devDependencies: { typescript: "6.0.3", vite: "8.0.0" },
  });
  writeJson(join(project, "tsconfig.json"), {
    compilerOptions: {
      target: "ES2022",
      module: "ESNext",
      moduleResolution: "Bundler",
      jsx: "preserve",
      lib: ["ES2022", "DOM", "DOM.Iterable"],
      types: [],
      strict: true,
      noEmit: true,
      skipLibCheck: true,
    },
    include: ["src/**/*.ts", "src/**/*.vue"],
  });
  writeFileSync(
    join(project, "vite.config.ts"),
    [
      'import { defineConfig } from "vite";',
      "",
      "export default defineConfig({",
      "  plugins: [],",
      "});",
      "",
    ].join("\n"),
  );
  writeFileSync(join(project, "src", "App.vue"), APP);
}

function sourceFiles(root: string, directory = root): string[] {
  const files: string[] = [];
  for (const name of readdirSync(directory).sort()) {
    if (name === "node_modules") continue;
    const path = join(directory, name);
    const stat = lstatSync(path);
    if (stat.isDirectory()) files.push(...sourceFiles(root, path));
    else files.push(relative(root, path));
  }
  return files;
}

export function sourceSnapshot(root: string): Record<string, string> {
  return Object.fromEntries(
    sourceFiles(root).map((name) => [name, sha256(readFileSync(join(root, name), "utf8"))]),
  );
}

function isInside(root: string, path: string): boolean {
  const candidate = relative(root, path);
  return (
    candidate === "" ||
    (candidate !== ".." && !candidate.startsWith(`..${sep}`) && !isAbsolute(candidate))
  );
}

export function assertCheckoutIsolation(checkout: string, project: string): void {
  assert.ok(!isInside(checkout, project), "fresh project is in the checkout");
  for (const name of sourceFiles(project)) {
    const path = join(project, name);
    if (lstatSync(path).isSymbolicLink()) {
      assert.ok(isInside(project, realpathSync(path)), `${name} links outside the fresh project`);
    }
    assert.ok(!readFileSync(path, "utf8").includes(checkout), `${name} names the source checkout`);
  }
}
