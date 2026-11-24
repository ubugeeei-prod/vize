/**
 * Records the native SFC compile calls the plugin makes and the files each one
 * compiles, so a suite can assert how compiles are batched.
 *
 * `compiler.ts` takes `compileSfc` and `compileSfcBatchWithResults` off
 * `@vizejs/native` when it is first evaluated. A suite must therefore import
 * this module before it imports the plugin.
 */

import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";

export interface NativeCompileCall {
  /** `single` is `compileSfc`; `batch` is `compileSfcBatchWithResults`. */
  api: "single" | "batch";
  ssr: boolean;
  files: string[];
}

type CompileOptions = { filename?: string; ssr?: boolean } | undefined;
type Compile = (this: unknown, input: unknown, options: CompileOptions) => unknown;

const native = createRequire(import.meta.url)("@vizejs/native") as Record<string, Compile>;
const calls: NativeCompileCall[] = [];

for (const [name, api] of [
  ["compileSfc", "single"],
  ["compileSfcBatchWithResults", "batch"],
] as const) {
  const compile = native[name];
  native[name] = function recorded(input, options) {
    calls.push({
      api,
      ssr: options?.ssr === true,
      files: Array.isArray(input)
        ? input.map((file: { path: string }) => file.path)
        : [options?.filename ?? ""],
    });
    return compile.call(this, input, options);
  };
}

/** The calls recorded since the last take, with their files relative to `root`. */
export function takeNativeCompileCalls(root: string): NativeCompileCall[] {
  return calls.splice(0).map((call) => ({
    ...call,
    files: call.files.map((file) => path.relative(root, file).replaceAll("\\", "/")),
  }));
}

/** A component whose compiled output differs for each `label`. */
export function labelledComponent(label: string): string {
  return `<script setup lang="ts">
const label = "${label}";
</script>

<template>
  <p class="label">{{ label }}</p>
</template>
`;
}

/** A scratch project holding `files`, keyed by their path relative to its root. */
export function scratchProject(files: Record<string, string>) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-demand-")));
  const file = (name: string) => path.join(root, name);
  const write = (name: string, source: string) => {
    fs.mkdirSync(path.dirname(file(name)), { recursive: true });
    fs.writeFileSync(file(name), source);
  };
  for (const [name, source] of Object.entries(files)) write(name, source);
  return { root, file, write, cleanup: () => fs.rmSync(root, { recursive: true, force: true }) };
}
