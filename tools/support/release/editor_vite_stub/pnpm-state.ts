import assert from "node:assert/strict";
import { existsSync, lstatSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  asArray,
  asNumber,
  asRecord,
  asString,
  parseJson,
  readJsonRecord,
} from "../../json-value.ts";
import { Recorder, sha256, success } from "../../config/windows_consumer/common.ts";
import { digest, inside, inventory, type Entry } from "./filesystem.ts";

const stateName = ".pnpm-workspace-state-v1.json";
export function stateSnapshot(root: string) {
  const modules = join(root, "node_modules"),
    path = join(modules, stateName);
  const metadata = lstatSync(path);
  assert.ok(
    metadata.isFile() && !metadata.isSymbolicLink(),
    "PNPM state must be an owned regular file",
  );
  const raw = readFileSync(path);
  const inputs = Object.fromEntries(
    ["package.json", "pnpm-lock.yaml", "vite.config.ts", "src/app.vue"]
      .filter((name) => existsSync(join(root, name)))
      .map((name) => [name, digest(join(root, name))]),
  );
  return {
    entries: inventory(modules),
    inputs,
    state: { bytes: raw.length, sha256: sha256(raw), rawBase64: raw.toString("base64") },
  };
}
export function validateState(snapshot: ReturnType<typeof stateSnapshot>): void {
  const raw = Buffer.from(snapshot.state.rawBase64, "base64");
  assert.equal(raw.length, snapshot.state.bytes);
  assert.equal(sha256(raw), snapshot.state.sha256);
  const row = snapshot.entries.find((entry) => entry.path === stateName);
  assert.ok(row && row.kind === "file");
  assert.equal(row.bytes, snapshot.state.bytes);
  assert.equal(row.sha256, snapshot.state.sha256);
  const state = asRecord(parseJson(raw.toString("utf8")));
  assert.ok(Number.isSafeInteger(asNumber(state.lastValidatedTimestamp)));
  asRecord(state.projects);
  asRecord(state.settings);
  asArray(state.pnpmfiles);
  assert.equal(typeof state.filteredInstall, "boolean");
}
export function capturePnpmState(root: string, record: Recorder, label: string) {
  // Preserve the full inventory even if reading or parsing the metadata fails.
  record.json(label + ".inventory.json", inventory(join(root, "node_modules")));
  try {
    const snapshot = stateSnapshot(root);
    record.json(label + ".snapshot.json", snapshot);
    return snapshot;
  } catch (error) {
    record.json(label + ".error.json", { error: String(error) });
    throw error;
  }
}
export function joinPnpmRounds(
  previous: ReturnType<typeof stateSnapshot>,
  next: ReturnType<typeof stateSnapshot>,
): void {
  assert.deepEqual(next, previous, "complete first-after and second-before snapshots must join");
}
export function onlyPnpmStateChanged(before: Entry[], after: Entry[]): void {
  assert.equal(before.length, after.length, "PNPM must not add/remove dependency entries");
  const current = new Map(after.map((row) => [row.path, row]));
  assert.equal(current.size, after.length);
  for (const row of before) {
    const actual = current.get(row.path);
    assert.ok(actual, "PNPM removed " + row.path);
    if (row.path === stateName) {
      assert.equal(row.kind, "file");
      assert.equal(actual.kind, "file");
      assert.equal(actual.mode, row.mode);
    } else assert.deepEqual(actual, row, "PNPM changed dependency entry " + row.path);
  }
}
/** An inert Node child observes the installation before loading any Vize code. */
export function auditPnpmState(root: string, pnpm: string[], record: Recorder): void {
  const rounds: unknown[] = [];
  let previous: ReturnType<typeof stateSnapshot> | undefined;
  for (const label of ["cold", "fixedpoint"]) {
    const before = capturePnpmState(root, record, label + "-before");
    if (previous) joinPnpmRounds(previous, before);
    validateState(before);
    const output = join(record.root, "pnpm-" + label + "-child-entry.json");
    const packet = record.run(
      "pnpm-" + label + "-inert-node",
      [
        ...pnpm,
        "exec",
        process.execPath,
        fileURLToPath(import.meta.url),
        "--snapshot",
        root,
        output,
      ],
      root,
    );
    const after = capturePnpmState(root, record, label + "-after");
    const child = existsSync(output) ? readJsonRecord(output) : null;
    const stateChanged = before.state.sha256 !== after.state.sha256;
    rounds.push({ label, before, child, after, packet, stateChanged });
    record.json("pnpm-actor.json", { status: "captured", rounds });
    success(packet);
    assert.ok(child);
    validateState(after);
    assert.equal(child.node, process.version);
    assert.equal(child.root, root);
    assert.deepEqual(
      child.snapshot,
      after,
      "the inert child/exit must not change the installation",
    );
    onlyPnpmStateChanged(before.entries, after.entries);
    assert.deepEqual(
      after.inputs,
      before.inputs,
      "PNPM-only observer changed authored inputs or lock",
    );
    if (label === "fixedpoint")
      assert.deepEqual(
        after,
        before,
        "second identical PNPM-only invocation must reach complete-vector stability",
      );
    previous = after;
  }
  record.json("pnpm-actor.json", {
    source:
      "pnpm12.1 commit6d90c71efdffbc909b499490b64c66badc720327 exec.rs105-109; deps-status.rs160-195",
    rounds,
    assertion:
      "when a cold state change occurs it is visible at inert Node entry within this control; both joined fixedpoint vectors are exact; no Vize code is imported by the control child",
  });
}

if (process.argv[2] === "--snapshot") {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  const [rootArg, output] = process.argv.slice(3);
  assert.ok(rootArg && output);
  const root = realpathSync(rootArg);
  assert.ok(!inside(root, output), "observer output must be outside the installation");
  writeFileSync(
    output,
    JSON.stringify(
      {
        node: process.version,
        pid: process.pid,
        argv: process.argv,
        root,
        snapshot: stateSnapshot(root),
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
}
