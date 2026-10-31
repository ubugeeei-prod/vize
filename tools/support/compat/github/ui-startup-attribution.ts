import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { lstatSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

export const sourceSha = "5a2298c91c35bfbb75c414def021887f973bfe48";
const baseSha = "b59f739547a189f85acf862ea650c260e5a62680";
type Row = Record<string, unknown>;
export interface Records {
  parents: Row[];
  workers: Row[];
  constructors: Row[];
}
const row = (value: unknown): Row => {
  assert.ok(
    typeof value === "object" && value !== null && !Array.isArray(value),
    "metadata object",
  );
  return value as Row;
};
const rows = (value: unknown): Row[] => {
  assert.ok(Array.isArray(value), "metadata array");
  return value.map(row);
};
const sha = (raw: Buffer | string): string => createHash("sha256").update(raw).digest("hex");
const git = (...args: string[]): string => execFileSync("git", args, { encoding: "utf8" }).trim();

/** Pure packet checks; the caller separately authenticates source and physical input bytes. */
export function validateRecords(records: Records, expected: readonly string[]) {
  assert.equal(records.parents.length, 1, "one original parent record");
  const parent = records.parents[0];
  assert.equal(parent.schema, "vize.ui.startup.process-samples.v1");
  assert.equal(row(parent.provenance).source_sha, sourceSha);
  assert.ok(parent.outcome === "returned" || parent.outcome === "threw");
  assert.ok(records.workers.length > 0, "actual worker records required");
  const inputs = new Map<string, Row>();
  const indexes = new Set<number>(),
    pids = new Set<number>();
  const total = records.workers[0].worker_total ?? rows(records.workers[0].inputs)[0]?.worker_total;
  assert.ok(typeof total === "number" && Number.isInteger(total) && total > 0 && total <= 8);
  for (const worker of records.workers) {
    assert.equal(worker.schema, "vize.ui.startup.worker.v1");
    assert.equal(row(worker.provenance).source_sha, sourceSha);
    const pid = worker.worker_pid;
    assert.ok(typeof pid === "number" && Number.isSafeInteger(pid) && pid > 0 && !pids.has(pid));
    pids.add(pid);
    const list = rows(worker.inputs);
    assert.ok(list.length > 0);
    const index = list[0].worker_index;
    assert.ok(
      typeof index === "number" &&
        Number.isInteger(index) &&
        index >= 0 &&
        index < total &&
        !indexes.has(index),
    );
    indexes.add(index);
    for (const [ordinal, input] of list.entries()) {
      assert.equal(input.worker_pid, pid);
      assert.equal(input.worker_index, index);
      assert.equal(input.worker_total, total);
      assert.equal(input.input_ordinal, ordinal);
      assert.equal(input.caller_attempt_id, `caller-${pid}-${ordinal}`);
      assert.equal(input.wire_request_id, null);
      assert.ok(typeof input.input_path === "string" && typeof input.input_sha256 === "string");
      assert.match(input.input_sha256, /^[0-9a-f]{64}$/);
      assert.ok(!inputs.has(input.caller_attempt_id as string), "unique caller attempt");
      inputs.set(input.caller_attempt_id as string, input);
    }
    assert.deepEqual(
      list.map((input) => input.input_path),
      expected.filter((_, ordinal) => ordinal % total === index).slice(0, list.length),
    );
    assert.equal(worker.content_hashes_deferred_to_parent, true);
    assert.equal(Object.hasOwn(worker, "already_loaded_node_images"), false);
  }
  const callerCensusComplete =
    parent.outcome === "returned" &&
    records.workers.length === total &&
    inputs.size === expected.length;
  assert.equal(parent.expected_worker_census, "UNKNOWN_not_validated");
  if (parent.outcome === "threw") {
    assert.deepEqual(parent.executable_hashes, []);
    assert.deepEqual(parent.build_manifests, []);
    assert.equal(parent.hashes_omitted_reason, "siblings_may_still_be_active");
  }
  assert.ok(records.constructors.length > 0, "actual constructor records required");
  const attempts = records.constructors.map((packet) => {
    assert.equal(packet.schema, "vize.ui.startup.constructor.v1");
    const caller = row(packet.caller),
      input = inputs.get(caller.caller_attempt_id as string);
    assert.ok(input, "constructor must join an observed caller");
    for (const key of [
      "worker_pid",
      "worker_index",
      "worker_total",
      "input_ordinal",
      "input_path",
      "input_sha256",
    ])
      assert.equal(caller[key], input[key]);
    assert.equal(packet.sdk_wire_request_id, null);
    const config = row(packet.config);
    assert.equal(config.profiler_was_configured, false, "original disabled profiler retained");
    assert.equal(config.profile_collection, "wrap_existing_sink_only");
    assert.ok(typeof packet.elapsed_ns === "string" && /^[0-9]+$/.test(packet.elapsed_ns));
    const events = rows(packet.events);
    assert.ok(events.length <= 256);
    return {
      caller: caller.caller_attempt_id,
      input: caller.input_path,
      elapsed_ns: packet.elapsed_ns,
      returned: packet.returned,
      configured: config,
      events: events.map((item) => row(item.event)),
      dropped: packet.dropped,
    };
  });
  const elapsed = attempts
    .map((item) => BigInt(item.elapsed_ns))
    .sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
  return {
    callerCensusComplete,
    workerTotal: total,
    workers: records.workers.length,
    inputs: inputs.size,
    observedConstructors: attempts.length,
    constructorCensus: "UNKNOWN_not_validated",
    eventCensus: "UNKNOWN_not_validated; empty samples do not imply zero work",
    observedElapsedMinNs: elapsed[0].toString(),
    observedElapsedMaxNs: elapsed.at(-1)!.toString(),
    attempts,
    physicalSameImageProven: false,
    actualRpcWaitProven: false,
    wireRequestIds: null,
    backendExitStatuses: null,
  };
}

function load(directory: string): Records {
  const result: Records = { parents: [], workers: [], constructors: [] };
  const names = readdirSync(directory);
  assert.ok(names.length > 0 && names.length <= 3000, "bounded nonempty metadata");
  let bytes = 0;
  for (const name of names) {
    const target = join(directory, name),
      stat = lstatSync(target);
    assert.ok(stat.isFile() && !stat.isSymbolicLink() && stat.size <= 2 * 1024 * 1024);
    bytes += stat.size;
    assert.ok(bytes <= 32 * 1024 * 1024, "bounded metadata packet");
    const value = row(JSON.parse(readFileSync(target, "utf8")) as unknown);
    if (/^ui-parent-[0-9]+\.json$/.test(name)) result.parents.push(value);
    else if (/^ui-worker-[0-9]+-[0-9]+\.json$/.test(name)) result.workers.push(value);
    else {
      assert.match(name, /^sdk-[0-9]+-[0-9]+\.json$/);
      result.constructors.push(value);
    }
  }
  return result;
}

function prepare(evidence: string): void {
  assert.equal(git("rev-parse", "HEAD"), sourceSha);
  assert.equal(git("rev-parse", "HEAD^"), baseSha);
  assert.match(process.env.CONTROLLER_SHA ?? "", /^[0-9a-f]{40}$/);
  mkdirSync(evidence, { recursive: true });
  const preload = `import { join } from "node:path";\nif (process.env.GITHUB_ACTIONS === "true" && process.platform === "linux" && process.env.RUNNER_TEMP) {\n  process.env.VIZE_UI_STARTUP_METADATA_DIR = join(process.env.RUNNER_TEMP, "ui-startup-metadata");\n  process.env.SOURCE_SHA = "${sourceSha}";\n}\n`;
  writeFileSync(join(evidence, "enable-observer.ts"), preload, { flag: "wx" });
  writeFileSync(
    join(evidence, "source.json"),
    JSON.stringify(
      {
        source: sourceSha,
        base: baseSha,
        tree: git("rev-parse", "HEAD^{tree}"),
        controller: process.env.CONTROLLER_SHA,
        workflowEventHead: process.env.GITHUB_SHA,
        run: process.env.GITHUB_RUN_ID,
        attempt: process.env.GITHUB_RUN_ATTEMPT,
        node: process.version,
        preloadSha256: sha(preload),
        scope: "source-pinned attribution; not shipping/public acceptance",
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
}

function validate(directory: string, evidence: string): void {
  const originalStatus = readFileSync(join(evidence, "original.status"), "utf8").trim();
  let summary: unknown,
    valid = false;
  try {
    assert.equal(git("rev-parse", "HEAD"), sourceSha);
    git("diff", "--exit-code", sourceSha, "--", "npm/ui/src");
    const checkout = git("rev-parse", "--show-toplevel");
    const expected = git("ls-tree", "-r", "--name-only", sourceSha, "--", "npm/ui/src")
      .split("\n")
      .filter((name) => name.endsWith(".vue") && !name.split("/").includes("examples"))
      .map((name) => resolve(checkout, name))
      .sort();
    assert.equal(expected.length, 543, "unchanged lint corpus excludes authored examples");
    const records = load(directory);
    summary = validateRecords(records, expected);
    const ownedPaths = new Set(expected);
    for (const worker of records.workers)
      for (const input of rows(worker.inputs)) {
        assert.ok(typeof input.input_path === "string" && ownedPaths.has(input.input_path));
        assert.equal(input.input_sha256, sha(readFileSync(input.input_path)));
      }
    valid = (summary as { callerCensusComplete: boolean }).callerCensusComplete;
  } catch {
    summary = {
      completeness: "UNOBSERVED_INVALID_OR_PREFIX",
      errorDetailsRetained: false,
      physicalSameImageProven: false,
      actualRpcWaitProven: false,
    };
  }
  const report = {
    schema: "vize.ui.startup.attribution-validation.v1",
    source: sourceSha,
    originalStatus,
    callerCensusComplete: valid,
    summary,
    timingScope: "caller bracket includes observer preparation; not wire wait",
    originalUiGateReplaced: false,
  };
  writeFileSync(
    join(evidence, "metadata-validation.json"),
    JSON.stringify(report, null, 2) + "\n",
    { flag: "wx" },
  );
  console.log(JSON.stringify({ originalStatus, callerCensusComplete: valid, source: sourceSha }));
  if (!valid)
    throw new Error(
      "Startup metadata completeness failed; original status and raw packets preserved separately",
    );
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  if (process.argv[2] === "prepare") prepare(process.argv[3]);
  else {
    assert.equal(process.argv[2], "validate");
    validate(process.argv[3], process.argv[4]);
  }
}
