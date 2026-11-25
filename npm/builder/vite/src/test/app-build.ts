/**
 * Builds a scratch project the way `vite build --app` does, and records each
 * environment's output files, native compile calls, warnings and error.
 *
 * `buildAppWithoutTargets` builds it again in a child process whose native
 * binding lacks `compileSfcBatchForTargets`, which is how the plugin compiled
 * before a batch could compile for several targets: the reference a shared
 * build must reproduce.
 */

import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Plugin } from "vite";

import type { NativeCompileCall } from "./native-compile-calls.ts";

export interface AppBuild {
  outputs: Record<string, Record<string, string>>;
  calls: Record<string, NativeCompileCall[]>;
  warnings: Record<string, string[]>;
  /** The environment whose build failed, and the message it failed with. */
  error?: { environment: string; message: string };
}

/** The environments of the app, by name, with their entries. */
export type AppEnvironments = Record<string, { entry: string; consumer?: "server" }>;

/** A host plugin the build adds after the plugin, by name (a child process cannot take a function). */
export type HostPlugin = "rewrite-ssr-item0";

function hostPlugin(name: HostPlugin): Plugin {
  // A host transform ordered before the compiler rewrites one SFC's SSR text.
  return {
    name: `test:${name}`,
    enforce: "pre",
    transform(code, id) {
      // The SFC main module of a build is a `?vue&vize` request of the file.
      if (this.environment.name !== "ssr" || !id.split("?")[0].endsWith("/Item0.vue")) return null;
      return code.replace('"src/components/Item0.vue"', '"rewritten for ssr"');
    },
  };
}

export async function buildApp(
  root: string,
  environments: AppEnvironments,
  plugins: HostPlugin[] = [],
): Promise<AppBuild> {
  // First: it wraps the native compile functions before the plugin takes them.
  const { takeNativeCompileCalls } = await import("./native-compile-calls.ts");
  const { createBuilder } = await import("vite");
  const { vize } = await import("../plugin/index.ts");
  const record: AppBuild = { outputs: {}, calls: {}, warnings: {} };
  let current = "";
  const observe: Plugin = {
    name: "test:app-build",
    buildStart() {
      current = this.environment.name;
      record.warnings[current] = [];
    },
    generateBundle(_, bundle) {
      record.outputs[this.environment.name] = Object.fromEntries(
        Object.values(bundle).map((file) => [
          file.fileName,
          file.type === "chunk" ? file.code : String(file.source),
        ]),
      );
    },
  };
  const builder = await createBuilder({
    root,
    configFile: false,
    logLevel: "silent",
    plugins: [...vize({ configMode: false }), observe, ...plugins.map(hostPlugin)],
    build: { write: false, minify: false },
    builder: {},
    environments: Object.fromEntries(
      Object.entries(environments).map(([name, { entry, consumer }]) => [
        name,
        { ...(consumer ? { consumer } : {}), build: { rollupOptions: { input: entry } } },
      ]),
    ),
  });
  const warn = console.warn;
  console.warn = (...args: unknown[]) => {
    record.warnings[current]?.push(args.map(String).join(" "));
  };
  try {
    for (const name of Object.keys(environments)) {
      try {
        await builder.build(builder.environments[name]);
      } catch (error) {
        // The host prints the stack into the message; which request of a
        // batch raised the error depends on the order the host requests in.
        const message = (error as Error).message
          .split("\n")
          .filter((line) => !/^\s+at /.test(line))
          .join("\n");
        record.error = { environment: name, message };
        break;
      } finally {
        // Environments build one after another, so the calls since the last take are this one's.
        record.calls[name] = takeNativeCompileCalls(root).map((call) => ({
          ...call,
          files: [...call.files].sort(),
        }));
      }
    }
  } finally {
    console.warn = warn;
  }
  return record;
}

const script = fileURLToPath(import.meta.url);

/** `buildApp` in a child process whose native binding cannot compile for several targets. */
export function buildAppWithoutTargets(
  root: string,
  environments: AppEnvironments,
  plugins: HostPlugin[] = [],
): AppBuild {
  const child = spawnSync(
    process.execPath,
    [script, "--without-targets", JSON.stringify({ root, environments, plugins })],
    { encoding: "utf8", maxBuffer: 256 * 1024 * 1024 },
  );
  if (child.status !== 0) throw new Error(`reference build failed:\n${child.stderr}`);
  return JSON.parse(child.stdout) as AppBuild;
}

if (process.argv[1] && path.resolve(process.argv[1]) === script) {
  const [flag, input] = process.argv.slice(2);
  if (flag !== "--without-targets" || input === undefined) {
    throw new Error("usage: app-build.ts --without-targets <json>");
  }
  // Before the plugin takes the compile functions off the binding.
  const native = createRequire(import.meta.url)("@vizejs/native") as Record<string, unknown>;
  delete native.compileSfcBatchForTargets;
  const { root, environments, plugins } = JSON.parse(input) as {
    root: string;
    environments: AppEnvironments;
    plugins: HostPlugin[];
  };
  const record = await buildApp(root, environments, plugins);
  process.stdout.write(JSON.stringify(record));
}
