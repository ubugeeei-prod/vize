import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { inspect } from "node:util";
import { createHash } from "node:crypto";
import { resolveConfig, type InlineConfig, type Plugin } from "vite";
import { toVirtualId } from "../virtual.ts";

const require = createRequire(import.meta.url);
interface Fixture {
  issue: number;
  scope: string;
  reportFileSha256: string;
  reportBodySha256: string;
  fencesFileSha256: string;
  files: { path: string; source: string }[];
  reportedCommands: string;
}

export function reportedProject(issue: 11375 | 11376) {
  const raw = fs.readFileSync(new URL(`./fixtures/startup-scan-${issue}.json`, import.meta.url));
  assert.equal(
    createHash("sha256").update(raw).digest("hex"),
    issue === 11375
      ? "b028204b4519dd5eafa31d200ced090834243ec918271e39048ad0639d33cc44"
      : "83ca0975a3d2bd1cbb703e074bd5f5e9b2325a428c2eada576a2c3800d8e263a",
  );
  const fixture: Fixture = JSON.parse(raw.toString("utf8"));
  assert.equal(fixture.issue, issue);
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), `vize-${issue}-`)));
  for (const file of fixture.files) {
    const target = path.join(root, file.path);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, file.source);
  }
  return {
    root,
    fixture,
    file: (name: string) => path.join(root, name),
    cleanup: () => fs.rmSync(root, { recursive: true, force: true }),
  };
}

/** The scratch graph uses the already prepared workspace Vue, without installing anything. */
export function linkWorkspaceVue(root: string) {
  const target = path.dirname(require.resolve("vue/package.json"));
  fs.mkdirSync(path.join(root, "node_modules"), { recursive: true });
  fs.symlinkSync(
    target,
    path.join(root, "node_modules/vue"),
    process.platform === "win32" ? "junction" : "dir",
  );
  return JSON.parse(fs.readFileSync(path.join(target, "package.json"), "utf8")) as {
    version: string;
  };
}

function hook<T extends Function>(value: T | { handler: T } | undefined): T {
  const fn = typeof value === "function" ? value : value?.handler;
  assert.equal(typeof fn, "function");
  return fn as T;
}

/** Real Vite configuration plus the plugin's actual buildStart and SFC load/compile consumers. */
export async function startupHost(
  root: string,
  config: InlineConfig,
  command: "build" | "serve" = "build",
  observeCandidates = false,
) {
  const candidates: { file: string; source: string }[] = [];
  const observer: Plugin = {
    name: "test:observe-startup-candidates",
    api: {
      vizePrecompileInputOwner: {
        version: 1,
        forConfig: () => ({
          ownsInput(file: string, source: string) {
            candidates.push({ file, source });
            return false;
          },
        }),
      },
    },
  };
  const resolved = await resolveConfig(
    {
      ...config,
      root,
      configFile: false,
      plugins: [...(config.plugins ?? []), ...(observeCandidates ? [observer] : [])],
    },
    command,
  );
  const plugin = resolved.plugins.find((value) => value.name === "vite-plugin-vize");
  const sfcPlugin = resolved.plugins.find((value) => value.name === "vite-plugin-vize:sfc");
  assert.ok(plugin && sfcPlugin);
  const context = { environment: { getTopLevelConfig: () => resolved } };
  return {
    resolved,
    candidates,
    start: () => hook(plugin.buildStart).call(context as never, {} as never),
    load: async (file: string, ssr = false) => {
      const id = toVirtualId(file, ssr);
      const pluginContext = { addWatchFile() {}, resolve: async () => null } as never;
      const loaded = await hook(plugin.load).call(pluginContext, id, { ssr });
      const source = typeof loaded === "string" ? loaded : (loaded as { code: string }).code;
      return hook(sfcPlugin.transform).call(pluginContext, source, id, { ssr } as never);
    },
    close: () => hook(plugin.closeBundle).call(context as never),
  };
}

/** Capture actual JS console arguments; no compiler result or error is replaced. */
export async function observeConsole<T>(action: () => T | Promise<T>) {
  const calls: { log: unknown[][]; warn: unknown[][]; error: unknown[][] } = {
    log: [],
    warn: [],
    error: [],
  };
  const original = { log: console.log, warn: console.warn, error: console.error };
  for (const key of ["log", "warn", "error"] as const)
    console[key] = (...args: unknown[]) => {
      calls[key].push(args);
    };
  try {
    return { value: await action(), error: null, calls };
  } catch (error) {
    return { value: undefined, error, calls };
  } finally {
    Object.assign(console, original);
  }
}

export function retainObservation(label: string, value: unknown) {
  console.log(
    `startup-scan ${label}: ${inspect(value, { depth: null, maxStringLength: null, maxArrayLength: null })}`,
  );
}
