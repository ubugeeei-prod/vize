import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  instrumentReadSources,
  readBufferDistribution,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-read-buffer.ts";
import { selectObservedTree } from "../../tools/benchmarks/scripts/cold-typechecker-scale-read-buffer-command.ts";
import { readBufferOrders } from "../../tools/benchmarks/scripts/cold-typechecker-scale-paired.ts";
import { strictPairSources } from "../../tools/benchmarks/scripts/cold-typechecker-scale-strict-pair.ts";

import {
  frozenColdSource,
  frozenColdSourcePins,
  verifyFrozenColdSource,
  verifyFrozenReadOverlay,
} from "./support/cold-typechecker-frozen-source.ts";

const sourceDirectory = "crates/vize_canon/src/batch/virtual_project/";
test("both actual frozen source roles accept exactly four read hooks and explicit owner carriage", () => {
  assert.equal(frozenColdSourcePins.base, strictPairSources.base);
  assert.equal(frozenColdSourcePins.head, strictPairSources.head);
  for (const role of ["base", "head"] as const) {
    const sha = frozenColdSourcePins[role];
    const read = (file: string) => frozenColdSource(role, sourceDirectory + file).toString();
    const project = read("project.rs"),
      materialize = read("materialize.rs");
    const builders = {
      build: read("build.rs"),
      script: read("build/script_build.rs"),
      jsx: read("jsx_build.rs"),
      artifacts: read("project/artifacts.rs"),
    };
    const changed = instrumentReadSources(project, materialize, builders);
    const overlay = {
      "project.rs": changed.project,
      "materialize.rs": changed.materialize,
      "build.rs": changed.builders.build,
      "build/script_build.rs": changed.builders.script,
      "jsx_build.rs": changed.builders.jsx,
      "project/artifacts.rs": changed.builders.artifacts,
      "project/owned_read_buffer_observation.rs": fs.readFileSync(
        new URL(
          "../../tools/benchmarks/scripts/cold-typechecker-scale-read-buffer.rs.txt",
          import.meta.url,
        ),
      ),
    };
    verifyFrozenReadOverlay(role, overlay);
    assert.throws(() => verifyFrozenReadOverlay(role, { ...overlay, extra: "unknown" }));
    assert.throws(() =>
      verifyFrozenReadOverlay(role, { ...overlay, "project.rs": changed.project + "\n" }),
    );
    assert.equal(changed.project.split("owned_read_buffer_observation::read(").length - 1, 4);
    assert.equal(changed.materialize.split("final_project(self)").length - 1, 1);
    assert(
      changed.builders.artifacts.includes(
        "::registered(registered.read_observation, &canonical_virtual)",
      ),
    );
    for (const name of ["build", "script", "jsx"] as const) {
      assert(
        changed.builders[name].includes(
          "__read_observation.success(&__observed_registered.original_content)",
        ),
      );
      assert(
        changed.builders[name].includes(
          sha === strictPairSources.head ? "matches!(&source, Cow::Owned(_))" : "false",
        ),
      );
    }
    assert.throws(() =>
      instrumentReadSources(
        project.replace(
          'profile!("canon.file.read", std::fs::read_to_string(path))?',
          "unreviewed_read(path)?",
        ),
        materialize,
        builders,
      ),
    );
    assert.throws(() => instrumentReadSources(changed.project, materialize, builders));
    assert.throws(() =>
      instrumentReadSources(
        project,
        materialize.replace("pub fn materialize(&self)", "pub fn changed(&self)"),
        builders,
      ),
    );
  }
});

test("whole historical fixtures refuse missing, truncated, changed and extra source bytes", () => {
  const file = sourceDirectory + "project.rs";
  const body = frozenColdSource("head", file);
  assert.throws(() => frozenColdSource("head", "unregistered.rs"));
  const changed = Buffer.from(body);
  changed[0] ^= 1;
  for (const mutant of [body.subarray(0, -1), changed, Buffer.concat([body, Buffer.from("\n")])])
    assert.throws(() => verifyFrozenColdSource("head", file, mutant));
});

const sha = "a".repeat(64);
const read = (id = 0) => ({
  kind: "read",
  id,
  path: "/project/a.ts",
  canonical: "/project/a.ts",
  route: "register_paths",
  length: 64,
  capacity: 128,
  pointer: "0x1000",
  sha256: sha,
});
const built = (id = 0, owned = true) => ({
  kind: "built",
  id,
  inputOwned: owned,
  length: 64,
  capacity: 128,
  pointer: owned ? "0x1000" : "0x2000",
  sha256: sha,
});
const registered = (id: number | null = 0) => ({
  kind: "registered",
  id,
  virtualPath: "/cache/a.ts",
});
const final = (r: ReturnType<typeof read> | null = read(), pointer = "0x1000") => ({
  kind: "final-project",
  projectRoot: "/project",
  virtualRoot: "/cache",
  rows: [
    {
      original: "/project/a.ts",
      virtualPath: "/cache/a.ts",
      read: r,
      retainedLength: 64,
      retainedCapacity: 128,
      retainedPointer: pointer,
      retainedSha256: sha,
    },
  ],
});
const trace = (rows: unknown[]) => rows.map((row) => JSON.stringify(row) + "\n").join("");
test("copy credit joins the actual successful final owner and preserves failed/superseded reads", () => {
  const result = readBufferDistribution(
    trace([read(), built(), registered(), read(1), { kind: "build-aborted", id: 1 }, final()]),
  );
  assert.equal(result.reads.length, 2);
  assert.deepEqual(result.abortedReads, [1]);
  assert.equal(result.rows[0].sameReadAllocation, true);
  assert.equal(result.joinedCopyBytes, 0);
  const copied = readBufferDistribution(
    trace([read(), built(0, false), registered(), final(read(), "0x2000")]),
  );
  assert.equal(copied.joinedCopyBytes, 64);
  assert.equal(copied.rows[0].sameReadAllocation, false);
  const inline = readBufferDistribution(
    trace([read(), { ...built(), pointer: "0x2000" }, registered(), final(read(), "0x3000")]),
  );
  assert.equal(inline.joinedCopyBytes, 64);
  const unknown = readBufferDistribution(trace([read(), built(), registered(null), final(null)]));
  assert.equal(unknown.borrowedOrUnjoinedFinalRows, 1);
  assert.equal(unknown.rows[0].copyBytes, null);
  assert.throws(() =>
    readBufferDistribution(
      trace([read(), built(), registered(), read(1), built(1), registered(1), final()]),
    ),
  );
});
test("malformed or fabricated trace relationships cannot create allocation reuse", () => {
  const valid = [read(), built(), registered(), final()];
  for (const mutant of [
    [{ ...read(), id: 1 }, built(), registered(), final()],
    [read(), { ...built(), inputOwned: false }, registered(), final()],
    [read(), { ...built(), sha256: "b".repeat(64) }, registered(), final()],
    [read(), registered(), final()],
    [read(), built(), registered(), { ...final(), rows: [...final().rows, ...final().rows] }],
    [
      read(),
      built(),
      registered(),
      { ...final(), rows: [{ ...final().rows[0], retainedSha256: "b".repeat(64) }] },
    ],
    [read(), built(), registered(), { ...final(), future: true }],
    [...valid, { kind: "unknown" }],
  ])
    assert.throws(() => readBufferDistribution(trace(mutant)));
  assert.throws(() => readBufferDistribution(trace(valid).slice(0, -1)));
});
test("primary order is exactly eight commands and RSS sums one live window, with PID reuse refusal", () => {
  assert.deepEqual(readBufferOrders, [
    ["base", "head"],
    ["head", "base"],
    ["head", "base"],
    ["base", "head"],
  ]);
  assert.equal(readBufferOrders.flat().length, 8);
  const rows = [
    { pid: 10, parent: 1, birthTicks: "4", rssBytes: 10, executable: "/vize" },
    { pid: 11, parent: 10, birthTicks: "5", rssBytes: 100, executable: "/native" },
    { pid: 12, parent: 1, birthTicks: "6", rssBytes: 999, executable: "/unowned" },
  ];
  assert.deepEqual(selectObservedTree(rows, 10, "4"), rows.slice(0, 2));
  assert.equal(
    selectObservedTree(rows, 10, "4").reduce((n, p) => n + p.rssBytes, 0),
    110,
  );
  assert.deepEqual(selectObservedTree(rows.slice(1), 10, "4"), []);
  assert.throws(() => selectObservedTree(rows, 10, "3"));
});
