import assert from "node:assert/strict";
import { mkdirSync, realpathSync, statfsSync } from "node:fs";
import { join, resolve } from "node:path";
import { homedir } from "node:os";
import { Recorder } from "../../config/windows_consumer/common.ts";
import { inside } from "../editor_vite_stub/filesystem.ts";
import { authenticate, originalContract } from "./authority.ts";
import { install, preservePayload } from "./install.ts";
import { runSemi } from "./semi.ts";
import { runSort } from "./sort.ts";
import { runArt } from "./art.ts";

export const forbiddenOverrides = [
  "NODE_OPTIONS",
  "NODE_PATH",
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_NATIVE_PATH",
  "VIZE_PREFER_WORKSPACE_BINDING",
  "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
  "VIZE_CONFIG_CACHE",
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
] as const;
export function assertEnvironment(env: NodeJS.ProcessEnv): void {
  assert.equal(env.GITHUB_ACTIONS, "true");
  assert.equal(env.GITHUB_REF, "refs/heads/main");
  for (const key of forbiddenOverrides) assert(!env[key], "initial override must be empty: " + key);
  assert.equal(env.VIZE_LSP_LOG_FILE, undefined, "default logger route must stay selected");
}

async function run(args: string[]): Promise<void> {
  assertEnvironment(process.env);
  assert.equal(args.length, 4);
  const [version, producer, blob, outputArg] = args;
  assert(version && producer && blob && outputArg);
  assert.equal(process.version, "v24.14.0");
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  const output = resolve(outputArg);
  assert(!inside(realpathSync(process.cwd()), output) && output !== realpathSync(process.cwd()));
  mkdirSync(output, { recursive: false });
  const disk = statfsSync(output);
  assert(
    disk.bavail * disk.bsize >= 3 * 1024 ** 3,
    "retain one GiB floor plus two GiB fresh installation forecast",
  );
  const record = new Recorder(realpathSync(output));
  const original = originalContract();
  let phase = "authority";
  record.json("controller.json", {
    controllerSource: process.env.GITHUB_SHA,
    runId: process.env.GITHUB_RUN_ID,
    attempt: process.env.GITHUB_RUN_ATTEMPT,
    version,
    producer,
    authorityBlob: blob,
    originalSource: original.source,
    node: process.version,
    platform: process.platform,
    arch: process.arch,
    logFileOverride: null,
    osHome: homedir(),
    cacheEnvironment: Object.fromEntries(
      ["HOME", "XDG_CACHE_HOME", "LOCALAPPDATA", "APPDATA", "USERPROFILE"].map((name) => [
        name,
        process.env[name] ?? null,
      ]),
    ),
    initialOverrides: Object.fromEntries(
      forbiddenOverrides.map((name) => [name, process.env[name] ?? null]),
    ),
  });
  try {
    const authority = await authenticate(version, producer, blob, original, record);
    phase = "fresh-public-install";
    record.json("progress.json", { phase, state: "started" });
    const context = install(join(record.root, "consumer"), authority, original, record);
    for (const [name, execute] of [
      ["semi", runSemi],
      ["sort", runSort],
      ["art", runArt],
    ] as const) {
      phase = name;
      record.json("progress.json", { phase, state: "started" });
      await execute(context);
      preservePayload(context, name + "-complete");
      record.json(name + ".complete.json", { phase, state: "passed" });
    }
    record.json("qualification.json", {
      schema: "vize.formatter-profiles-public.v1",
      state: "passed",
      version,
      candidate: authority.candidate,
      producerSource: authority.ownership.producerSource,
      producerRun: producer,
      authorityBlob: blob,
      controllerSource: process.env.GITHUB_SHA,
      consumerRun: process.env.GITHUB_RUN_ID,
      attempt: process.env.GITHUB_RUN_ATTEMPT,
      node: process.version,
      platform: process.platform,
      arch: process.arch,
      originalSource: original.source,
      semi: "twelve delivered regression cells, 48 original commands and three refusals",
      sort: "whole original CLI and document/range/on-type formatting controls",
      art: "whole original UTF16 selection, four profiles and finite source controls",
      filesystem:
        "published payload exact throughout; complete module CLI effects retained; strict whole project+installation for all LSP intervals",
      limitations: [
        "Linux x64/glibc and Node24 only",
        "Frozen independent Prettier3.8.3 outputs; reference not reexecuted here",
        "No warm watched-config recovery, diagnostic/typechecker parity, generic formatter coverage, timing claim, or whole6098 closure",
      ],
    });
  } catch (error) {
    record.json("qualification.json", {
      schema: "vize.formatter-profiles-public.v1",
      state: "failed",
      phase,
      version,
      producer,
      authorityBlob: blob,
      originalSource: original.source,
      controllerSource: process.env.GITHUB_SHA,
      consumerRun: process.env.GITHUB_RUN_ID,
      error:
        error instanceof Error
          ? { name: error.name, message: error.message, stack: error.stack }
          : { value: String(error) },
      scope:
        "partial original evidence retained; no acceptance or cause inferred from failure status alone",
    });
    throw error;
  }
}

if (import.meta.main) {
  try {
    await run(process.argv.slice(2));
  } catch (error) {
    process.stderr.write(
      (error instanceof Error ? (error.stack ?? error.message) : String(error)) + "\n",
    );
    process.exitCode = 1;
  }
}
