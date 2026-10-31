import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import {
  lstatSync,
  readFileSync,
  readdirSync,
  realpathSync,
  statSync,
  writeFileSync,
  writeSync,
} from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createSampler } from "../../../../tools/support/compat/davinci/lib/lsp-resource-sampler.ts";

const require = createRequire(import.meta.url);
const failureMarker = "Vize UI startup metadata unavailable (redacted)";
type Row = Record<string, unknown>;
type State = {
  root: string;
  configured: string;
  inputs: Row[];
  provider: string | undefined;
  flush: number;
};
let active: State | undefined;
let warned = false;
const provenance = () => ({
  github_sha: selectedSetting(process.env.GITHUB_SHA, /^[0-9a-f]{40}$/i),
  source_sha: selectedSetting(process.env.SOURCE_SHA, /^[0-9a-f]{40}$/i),
  run_id: selectedSetting(process.env.GITHUB_RUN_ID, /^[0-9]{1,20}$/),
  attempt: selectedSetting(process.env.GITHUB_RUN_ATTEMPT, /^[0-9]{1,6}$/),
  node: process.version,
});
function selectedSetting(value: string | undefined, pattern: RegExp): string | null {
  return value !== undefined && pattern.test(value) ? value : null;
}

function state(): State | undefined {
  const configured = process.env.VIZE_UI_STARTUP_METADATA_DIR;
  if (!configured) {
    active = undefined;
    return;
  }
  if (active?.configured === configured) return active;
  active = undefined;
  try {
    assert.equal(
      process.env.GITHUB_ACTIONS,
      "true",
      "startup metadata is an Actions-only observer",
    );
    assert.equal(process.platform, "linux", "the initial passive process slice requires /proc");
    assert.ok(process.env.RUNNER_TEMP && path.isAbsolute(configured));
    const root = realpathSync(configured),
      temporary = realpathSync(process.env.RUNNER_TEMP);
    const relative = path.relative(temporary, root);
    assert.ok(
      relative &&
        relative !== ".." &&
        !relative.startsWith(".." + path.sep) &&
        !path.isAbsolute(relative),
    );
    assert.ok(lstatSync(configured).isDirectory() && !lstatSync(configured).isSymbolicLink());
    active = { root, configured, inputs: [], provider: undefined, flush: 0 };
    return active;
  } catch {
    unavailable();
    return;
  }
}

function unavailable(): void {
  if (warned) return;
  warned = true;
  try {
    writeSync(2, failureMarker + "\n");
  } catch {
    /* Never replace the original outcome. */
  }
}

/** Never retain arbitrary error messages, stacks, argv or environment objects. */
export function normalizedError(error: unknown): Row {
  const name =
    error instanceof TypeError
      ? "TypeError"
      : error instanceof RangeError
        ? "RangeError"
        : error instanceof Error
          ? "Error"
          : "other";
  const descriptor =
    typeof error === "object" && error !== null
      ? Object.getOwnPropertyDescriptor(error, "code")
      : undefined;
  const code: unknown = descriptor && "value" in descriptor ? descriptor.value : undefined;
  return {
    kind: name,
    code:
      typeof code === "string" && ["ENOENT", "EACCES", "EPERM", "EEXIST"].includes(code)
        ? code
        : "other",
    redacted: true,
  };
}

function save(current: State, name: string, value: Row): void {
  try {
    writeFileSync(path.join(current.root, name), JSON.stringify(value, null, 2) + "\n", {
      flag: "wx",
    });
  } catch {
    // Preserve the original lint result/error; an absent proof is never admitted as complete.
    unavailable();
  }
}

export function startupProvider(selected: string | undefined): void {
  const current = state();
  if (current) current.provider = selected;
}

export function startupInput<T>(
  filename: string,
  source: string,
  ordinal: number,
  shard: { readonly index: number; readonly total: number },
  run: () => T,
): T {
  const current = state();
  if (!current) return run();
  const caller = {
    schema: "vize.ui.startup.caller.v1",
    caller_attempt_id: `caller-${process.pid}-${ordinal}`,
    worker_pid: process.pid,
    worker_index: shard.index,
    worker_total: shard.total,
    input_ordinal: ordinal,
    input_path: filename,
    input_sha256: createHash("sha256").update(source).digest("hex"),
  };
  const previous = process.env.VIZE_UI_STARTUP_CONTEXT;
  process.env.VIZE_UI_STARTUP_CONTEXT = JSON.stringify(caller);
  const utc = Date.now(),
    start = process.hrtime.bigint();
  let outcome = "returned";
  try {
    return run();
  } catch (error) {
    outcome = "threw";
    throw error;
  } finally {
    const elapsed = process.hrtime.bigint() - start;
    if (previous === undefined) delete process.env.VIZE_UI_STARTUP_CONTEXT;
    else process.env.VIZE_UI_STARTUP_CONTEXT = previous;
    current.inputs.push({
      ...caller,
      wire_request_id: null,
      identity_kind: "caller_only_not_sdk_wire_id",
      clock: "node.hrtime.bigint",
      utc_ms: utc,
      start_ns: start.toString(),
      elapsed_ns: elapsed.toString(),
      outcome,
    });
  }
}

/** Called only in the parent after ALL original workers returned. Never from worker flush. */
function postOutcomeHash(selected: string): Row {
  try {
    const target = realpathSync(selected),
      before = statSync(target, { bigint: true });
    assert.ok(before.isFile());
    const raw = readFileSync(target),
      after = statSync(target, { bigint: true });
    const identity = (value: typeof before) =>
      [value.dev, value.ino, value.size, value.mtimeNs, value.ctimeNs].map(String);
    assert.deepEqual(identity(before), identity(after));
    return {
      path: target,
      bytes: raw.length,
      sha256: createHash("sha256").update(raw).digest("hex"),
      file_identity: identity(after),
      hash_timing: "after_all_original_workers_returned",
      custody: "sampled_path_and_later_bytes_not_pre_execution_image_attestation",
    };
  } catch (error) {
    return { path: selected, available: false, error: normalizedError(error) };
  }
}

export function flushStartupWorker(): void {
  const current = state();
  if (!current) return;
  const images = Object.keys(require.cache).filter(
    (name) => name.endsWith(".node") && require.cache[name]?.loaded,
  );
  save(current, `ui-worker-${process.pid}-${current.flush++}.json`, {
    schema: "vize.ui.startup.worker.v1",
    provenance: provenance(),
    worker_pid: process.pid,
    inputs: current.inputs.splice(0),
    selected_provider_path: current.provider ?? null,
    already_loaded_node_image_paths: images,
    content_hashes_deferred_to_parent: true,
    native_identity_queries_added: 0,
  });
}

function workerImagePaths(current: State): { paths: string[]; complete: boolean } {
  const paths: string[] = [];
  let complete = true;
  for (const name of readdirSync(current.root).filter((name) =>
    /^(?:ui-worker|sdk)-[0-9]+-[0-9]+\.json$/.test(name),
  )) {
    try {
      const file = path.join(current.root, name),
        metadata = lstatSync(file);
      assert.ok(metadata.isFile() && !metadata.isSymbolicLink() && metadata.size <= 1024 * 1024);
      const value: unknown = JSON.parse(readFileSync(file, "utf8")) as unknown;
      assert.ok(typeof value === "object" && value !== null);
      const row = value as Row;
      const selected: unknown[] = [row.selected_provider_path];
      if (Array.isArray(row.already_loaded_node_image_paths))
        selected.push(...row.already_loaded_node_image_paths);
      if (typeof row.config === "object" && row.config !== null)
        selected.push((row.config as Row).executable);
      for (const item of selected)
        if (typeof item === "string" && item.length <= 4096 && !item.includes("\0"))
          paths.push(item);
    } catch {
      complete = false;
    }
  }
  return { paths: [...new Set(paths)], complete };
}

/** This runs in the UI parent, outside synchronous native lint workers. No signal/proxy/gate. */
export function startupParent(): ((outcome: string) => void) | undefined {
  const current = state();
  if (!current) return;
  let sampler: ReturnType<typeof createSampler>, fresh: boolean;
  try {
    fresh = readdirSync(current.root).length === 0;
    sampler = createSampler(process.pid);
  } catch {
    unavailable();
    return;
  }
  const start = process.hrtime.bigint();
  return (outcome) => {
    try {
      const sampled = sampler.stop();
      const observations = sampled.observed_processes.map((entry) => ({
        pid: entry.pid,
        birth_ticks: entry.birth_ticks,
        parent_pids: entry.parent_pids,
        first_sample: entry.first_sample,
        last_sample: entry.last_sample,
        samples: entry.samples,
        executable_paths: entry.executables,
      }));
      const returned = outcome === "returned";
      const mayHash = returned && fresh;
      const workers = mayHash ? workerImagePaths(current) : { paths: [], complete: false };
      const images = mayHash
        ? [
            ...new Set([
              ...observations.flatMap((entry) => entry.executable_paths),
              ...workers.paths,
            ]),
          ]
        : [];
      save(current, `ui-parent-${process.pid}.json`, {
        schema: "vize.ui.startup.process-samples.v1",
        parent_pid: process.pid,
        provenance: provenance(),
        outcome,
        elapsed_ns: (process.hrtime.bigint() - start).toString(),
        sample_interval_ms: 50,
        observations,
        executable_hashes: images.map(postOutcomeHash),
        build_manifests: (mayHash ? ["Cargo.lock", "pnpm-lock.yaml"] : []).map((name) =>
          postOutcomeHash(fileURLToPath(new URL("../../../../" + name, import.meta.url))),
        ),
        metadata_directory_fresh: fresh,
        discovered_worker_files_parsed: workers.complete,
        expected_worker_census: "UNKNOWN_not_validated",
        sample_window: returned
          ? "until_original_workers_returned"
          : "prefix_until_original_fail_fast",
        hashes_omitted_reason: mayHash
          ? null
          : returned
            ? "metadata_directory_not_fresh"
            : "siblings_may_still_be_active",
        physical_same_image_custody: "UNPROVEN_post_outcome_hashes_only",
        backend_exit_status: null,
        sdk_wire_request_ids: null,
        limits: [
          "sampling can miss short-lived children",
          "last_seen is not an exit event",
          "post-outcome hashes do not attest pre-execution bytes",
        ],
      });
    } catch {
      unavailable();
    }
  };
}
