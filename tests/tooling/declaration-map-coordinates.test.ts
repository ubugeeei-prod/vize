import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { SourceMap } from "node:module";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function position(source: string, needle: string) {
  const offset = source.indexOf(needle);
  assert.notEqual(offset, -1, needle);
  const lines = source.slice(0, offset).split("\n");
  return [lines.length - 1, lines.at(-1)!.length] as const;
}

test("declaration maps resolve emitted public names to exact authored UTF-16 positions", () => {
  const [command, ...launch] = resolveVizeLaunchCommand();
  launch.pop(); // Reuse the checkout-owned CLI selection with the check subcommand.
  for (const newline of ["\n", "\r\n"]) {
    const project = fs.mkdtempSync(path.join(os.tmpdir(), "vize declaration maps "));
    const sources = {
      "App.vue": `<script lang="ts">
export interface Café { message: string; }
export const emoji = '😀'; export const café = 1;
</script>
<script setup lang="ts">defineProps<Café>()</script>
<template><div>{{ message }}</div></template>
`.replaceAll("\n", newline),
      "Widget.vue": `<script lang="tsx">
export interface WidgetProps { title: string; }
export const widgetValue = 1;
</script><template><div/></template>`.replaceAll("\n", newline),
      "index.ts": `export { café } from './App.vue'; export { widgetValue } from './Widget.vue'; const emoji = '😀'; export const after = 1;${newline}`,
    };
    try {
      fs.mkdirSync(path.join(project, "src"));
      fs.symlinkSync(
        path.join(root, "node_modules"),
        path.join(project, "node_modules"),
        "junction",
      );
      fs.writeFileSync(
        path.join(project, "tsconfig.json"),
        JSON.stringify({
          compilerOptions: {
            strict: true,
            target: "ES2022",
            module: "ESNext",
            moduleResolution: "bundler",
            declaration: true,
            emitDeclarationOnly: true,
            declarationMap: true,
            outDir: "types",
            rootDir: "src",
          },
          include: ["src/**/*"],
        }),
      );
      for (const [file, source] of Object.entries(sources)) {
        fs.writeFileSync(path.join(project, "src", file), source);
      }
      const result = spawnSync(
        command,
        [
          ...launch,
          "check",
          ".",
          "--declaration",
          "--declaration-dir",
          "types",
          "--format",
          "json",
        ],
        {
          cwd: project,
          encoding: "utf8",
          timeout: 60_000,
        },
      );
      assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
      assert.equal(JSON.parse(result.stdout).errorCount, 0);
      for (const [file, name, authored] of [
        ["App.vue", "Café", "Café"],
        ["App.vue", "café", "café ="],
        ["Widget.vue", "WidgetProps", "WidgetProps"],
        ["Widget.vue", "widgetValue", "widgetValue ="],
        ["index.ts", "after", "after ="],
      ] as const) {
        const output = file.endsWith(".ts") ? file.slice(0, -3) : file;
        const declaration = fs.readFileSync(path.join(project, "types", `${output}.d.ts`), "utf8");
        const json = JSON.parse(
          fs.readFileSync(path.join(project, "types", `${output}.d.ts.map`), "utf8"),
        );
        const map = new SourceMap(json);
        const entry = map.findEntry(...position(declaration, name));
        assert.deepEqual(
          [entry.originalLine, entry.originalColumn],
          position(sources[file], authored),
          `${file}:${name}`,
        );
        assert.equal(
          path.resolve(project, "types", entry.originalSource!),
          path.join(project, "src", file),
        );
        if (file.endsWith(".vue")) {
          assert.equal(
            map.findEntry(0, 0).originalSource,
            undefined,
            "helper import has no authored target",
          );
        }
      }
    } finally {
      fs.rmSync(project, { recursive: true, force: true });
    }
  }
});
