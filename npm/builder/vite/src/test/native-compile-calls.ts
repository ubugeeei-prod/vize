/**
 * Records the native SFC compile calls the plugin makes and the files each one
 * compiles, so a suite can assert how compiles are batched.
 *
 * `compiler.ts` and `compile-targets.ts` take the compile functions off
 * `@vizejs/native` when they are first evaluated. A suite must therefore import
 * this module before it imports the plugin.
 */

import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";

export interface NativeCompileCall {
  /**
   * `single` is `compileSfc`; `batch` is `compileSfcBatchWithResults`;
   * `targets` is `compileSfcBatchForTargets`, which also compiles for `alsoSsr`.
   */
  api: "single" | "batch" | "targets";
  ssr: boolean;
  /** The target a `targets` call compiles besides `ssr`. */
  alsoSsr?: boolean;
  files: string[];
}

type CompileOptions = { filename?: string; ssr?: boolean } | undefined;
type Targets = { ssr?: boolean }[] | undefined;
type Compile = (
  this: unknown,
  input: unknown,
  options: CompileOptions,
  targets?: Targets,
) => unknown;

const native = createRequire(import.meta.url)("@vizejs/native") as Record<string, Compile>;
const calls: NativeCompileCall[] = [];

for (const [name, api] of [
  ["compileSfc", "single"],
  ["compileSfcBatchWithResults", "batch"],
  ["compileSfcBatchForTargets", "targets"],
] as const) {
  const compile = native[name];
  if (typeof compile !== "function") continue;
  native[name] = function recorded(input, options, targets) {
    const files = Array.isArray(input)
      ? input.map((file: { path: string }) => file.path)
      : [options?.filename ?? ""];
    calls.push(
      api === "targets"
        ? { api, ssr: targets?.[0]?.ssr === true, alsoSsr: targets?.[1]?.ssr === true, files }
        : { api, ssr: options?.ssr === true, files },
    );
    return compile.call(this, input, options, targets);
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

/** How often `run` reads, stats and probes each `.vue` file, by its path relative to `root`. */
export async function countVueFileAccess(root: string, run: () => unknown) {
  const calls = { read: new Map<string, number>(), stat: new Map<string, number>() };
  const probes = new Map<string, number>();
  const count = (counted: Map<string, number>, file: unknown) => {
    if (typeof file !== "string" || !file.endsWith(".vue")) return;
    const name = path.relative(root, file).replaceAll("\\", "/");
    counted.set(name, (counted.get(name) ?? 0) + 1);
  };
  const { readFileSync, statSync, existsSync } = fs;
  fs.readFileSync = ((file: fs.PathOrFileDescriptor, ...rest: never[]) => {
    count(calls.read, file);
    return readFileSync(file, ...rest);
  }) as typeof readFileSync;
  fs.statSync = ((file: fs.PathLike, ...rest: never[]) => {
    count(calls.stat, file);
    return statSync(file, ...rest);
  }) as typeof statSync;
  fs.existsSync = (file) => {
    count(probes, file);
    return existsSync(file);
  };
  try {
    await run();
  } finally {
    Object.assign(fs, { readFileSync, statSync, existsSync });
  }
  return {
    read: Object.fromEntries(calls.read),
    stat: Object.fromEntries(calls.stat),
    exists: Object.fromEntries(probes),
  };
}
