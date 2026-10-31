import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { ObservationWriter, parseObservationConfig } from "../preload.ts";
import { OBSERVATION_LIMITS } from "../types.ts";
import { cohort, config, fill, phases, snapshot } from "./preload-fixture.ts";

function directory(): string {
  return fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-observation-control-")));
}
function summary(root: string, name: string): Record<string, unknown> {
  return JSON.parse(fs.readFileSync(path.join(root, `${name}.summary.json`), "utf8")) as Record<string, unknown>;
}
function writer(root: string, name: string): ObservationWriter {
  return new ObservationWriter(root, name, { kind: "header", diagnosticOnly: true, qualificationCredit: 0 });
}
function set(value: object, keys: readonly (string | number)[], replacement: unknown): void {
  let target: object = value;
  for (const key of keys.slice(0, -1)) {
    const next: unknown = Reflect.get(target, String(key));
    assert.ok(next !== null && typeof next === "object"); target = next;
  }
  Reflect.set(target, String(keys.at(-1)), replacement);
}

test("config validates exact schema, source digests, absolute paths, metadata and distinct image roles", () => {
  const original = config("/synthetic/output");
  assert.deepEqual(parseObservationConfig(original), original);
  assert.deepEqual(parseObservationConfig({ ...original, images: [...original.images].reverse() }).images,
    [...original.images].reverse());
  const changes: Array<[readonly (string | number)[], unknown]> = [
    [["schema"], "foreign"], [["version"], 2], [["sourceRevision"], "a".repeat(39)],
    [["workflowRevision"], "B".repeat(40)], [["sessionSourceSha256"], "d".repeat(63)],
    [["metricsSourceSha256"], "e".repeat(63)], [["metricsSourceSha256"], undefined],
    [["repoRoot"], "relative"], [["outputRoot"], "/absolute\0injection"],
    [["images"], [original.images[0]]], [["images", 1, "role"], "server"],
    [["images", 1, "realpath"], original.images[0]!.realpath],
    [["images", 0, "realpath"], "relative"], [["images", 0, "sha256"], "not-a-sha"],
    [["images", 0, "bytes"], 0], [["images", 0, "bytes"], Number.NaN],
    [["images", 0, "dev"], "-1"], [["nodeImage", "mtimeNs"], "invalid"],
    [["nodeImage", "realpath"], "node"], [["nodeImage", "sha256"], "0".repeat(65)],
  ];
  for (const [keys, replacement] of changes) {
    const value = structuredClone(original); set(value, keys, replacement);
    assert.throws(() => parseObservationConfig(value), keys.join("."));
  }
  for (const value of [null, [], {}]) assert.throws(() => parseObservationConfig(value));
  assert.deepEqual(original, config("/synthetic/output"), "validation must not mutate caller metadata");
});

test("writer refuses aliases and unsafe basenames and requires an owned canonical output directory", () => {
  const root = directory();
  try {
    const alias = path.join(root, "alias"); fs.symlinkSync(root, alias);
    assert.throws(() => writer(alias, "alias-control"));
    assert.throws(() => writer(path.join(root, "missing"), "absent-control"));
    for (const name of ["../escape", "Uppercase", "line\ninjection", ""]) assert.throws(() => writer(root, name));
    const okay = writer(root, "canonical-control");
    okay.finish(config(root), 900, snapshot(), [], cohort());
    assert.equal(summary(root, "canonical-control")["complete"], false);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("exclusive JSONL and summary creation preserve stale evidence and collision bytes", () => {
  const root = directory();
  try {
    fs.writeFileSync(path.join(root, "old-jsonl.jsonl"), "preserved-jsonl\n", { flag: "wx" });
    assert.throws(() => writer(root, "old-jsonl"));
    assert.equal(fs.readFileSync(path.join(root, "old-jsonl.jsonl"), "utf8"), "preserved-jsonl\n");
    fs.writeFileSync(path.join(root, "old-summary.summary.json"), "preserved-summary\n", { flag: "wx" });
    assert.throws(() => writer(root, "old-summary"));
    assert.equal(fs.existsSync(path.join(root, "old-summary.jsonl")), false);
    const race = writer(root, "racing-summary");
    fs.writeFileSync(path.join(root, "racing-summary.summary.json"), "racing-summary\n", { flag: "wx" });
    assert.throws(() => race.finish(config(root), 900, snapshot(), [], cohort()));
    assert.equal(fs.readFileSync(path.join(root, "racing-summary.summary.json"), "utf8"), "racing-summary\n");
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("all twenty-eight Vben edges and actual sample records yield only diagnostic completeness with exact output bytes", () => {
  const root = directory();
  try {
    const output = writer(root, "complete-control"); fill(output);
    output.finish(config(root), 900, snapshot(), [], cohort());
    const value = summary(root, "complete-control");
    assert.equal(value["complete"], true);
    assert.equal(value["qualificationCredit"], 0);
    assert.equal(value["intervalCallbacks"], 7, "new callback timing evidence must survive finalization");
    assert.deepEqual(value["missingPhases"], []);
    assert.deepEqual(value["missingContainedWindows"], []);
    const jsonl = fs.statSync(path.join(root, "complete-control.jsonl")).size;
    const bytes = jsonl + fs.statSync(path.join(root, "complete-control.summary.json")).size;
    assert.equal(value["jsonlBytes"], jsonl); assert.equal(value["outputBytes"], bytes);
    const before = fs.readFileSync(path.join(root, "complete-control.summary.json"));
    output.finish(config(root), 900, snapshot(), [], cohort());
    assert.deepEqual(fs.readFileSync(path.join(root, "complete-control.summary.json")), before);
    assert.throws(() => output.record({ kind: "notice", atNs: "0", reason: "closed", fatal: false }));
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("every missing edge is incomplete while fast lanes do not require contained samples", () => {
  const root = directory();
  try {
    for (const [index, phase] of phases.entries()) {
      const name = `missing-edge-${index}`; const output = writer(root, name); fill(output, phase);
      output.finish(config(root), 900, snapshot(), [], cohort());
      const value = summary(root, name);
      assert.equal(value["complete"], false); assert.ok((value["missingPhases"] as string[]).includes(phase));
    }
    const output = writer(root, "overlap-only"); fill(output, undefined, false);
    output.finish(config(root), 900, snapshot(), [], cohort());
    const value = summary(root, "overlap-only");
    assert.equal(value["complete"], true);
    assert.deepEqual(value["containedSamplesByPhase"], {});
    assert.deepEqual(value["missingContainedWindows"], []);
    assert.equal(value["observedSampleRecords"], 3);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("invalid, in-flight, absent-root, zero-sample and preload-error observations never become complete", () => {
  const root = directory();
  try {
    for (const [index, changes] of [{ invalid: true }, { inFlight: true }, { root: null }, { samples: 0 }].entries()) {
      const name = `incomplete-${index}`; const output = writer(root, name); fill(output);
      output.finish(config(root), 900, snapshot(changes), [], cohort());
      assert.equal(summary(root, name)["complete"], false);
    }
    const output = writer(root, "preload-error"); fill(output);
    output.finish(config(root), 900, snapshot(), ["synthetic refusal"], cohort());
    assert.equal(summary(root, "preload-error")["complete"], false);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("UTF8 JSONL cap reserves terminal metadata and total real bytes never exceed eight MiB", () => {
  const root = directory();
  try {
    const output = writer(root, "unicode-cap"); fill(output);
    let refused = false;
    for (let index = 0; index < 10_000; index += 1) {
      try { output.record({ kind: "notice", atNs: String(index), reason: "雪🌸".repeat(256), fatal: false }); }
      catch { refused = true; break; }
    }
    assert.ok(refused, "control must actually hit the UTF8 byte cap");
    output.finish(config(root), 900, snapshot(), [], cohort());
    const value = summary(root, "unicode-cap");
    const jsonl = fs.statSync(path.join(root, "unicode-cap.jsonl")).size;
    const terminal = fs.statSync(path.join(root, "unicode-cap.summary.json")).size;
    assert.ok(jsonl <= OBSERVATION_LIMITS.outputBytes - OBSERVATION_LIMITS.finalReserveBytes);
    assert.ok(terminal <= OBSERVATION_LIMITS.finalReserveBytes);
    assert.ok(jsonl + terminal <= OBSERVATION_LIMITS.outputBytes);
    assert.equal(value["outputBytes"], jsonl + terminal);
    assert.equal(value["truncated"], true); assert.equal(value["complete"], false);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("UTF8-heavy refusal metadata still produces a compact bounded incomplete terminal summary", () => {
  const root = directory();
  try {
    const output = writer(root, "compact-summary"); fill(output);
    output.finish(config(root), 900, snapshot({
      invalid: true, refusals: Array<string>(8).fill("🌸".repeat(128)),
      stops: [{ reason: "🌸".repeat(4000), atNs: "90" }],
    }), Array<string>(4).fill("🌸".repeat(128)), cohort());
    const value = summary(root, "compact-summary");
    const jsonl = fs.statSync(path.join(root, "compact-summary.jsonl")).size;
    const terminal = fs.statSync(path.join(root, "compact-summary.summary.json")).size;
    assert.ok(terminal <= OBSERVATION_LIMITS.finalReserveBytes);
    assert.ok(jsonl + terminal <= OBSERVATION_LIMITS.outputBytes);
    assert.equal(value["outputBytes"], jsonl + terminal);
    assert.equal(value["complete"], false);
    const valid = writer(root, "compact-valid-window"); fill(valid);
    valid.finish(config(root), 900, snapshot({ stops: [{ reason: "🌸".repeat(4000), atNs: "90" }] }), [], cohort());
    const compacted = summary(root, "compact-valid-window");
    assert.equal(compacted["complete"], false, "metadata loss alone cannot retain a completeness claim");
    const compactBytes = fs.statSync(path.join(root, "compact-valid-window.summary.json")).size;
    assert.ok(compactBytes <= OBSERVATION_LIMITS.finalReserveBytes);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("selected cohort identity/refusals and sample-count mismatches cannot claim completeness", () => {
  const root = directory();
  try {
    const target = cohort().target; assert.ok(target);
    const invalid = [cohort({ state: "incomplete" }), cohort({ target: null }), cohort({ refusalCount: 1 }),
      cohort({ refusals: ["synthetic cohort refusal"] }), cohort({ target: { ...target, parentPid: 99 } }),
      cohort({ target: { ...target, pid: 900 } }), cohort({ target: { ...target, executableSha256: "0".repeat(64) } }),
      cohort({ target: { ...target, birthTicks: "foreign" } })];
    for (const [index, selected] of invalid.entries()) {
      const name = `bad-cohort-${index}`, output = writer(root, name); fill(output);
      output.finish(config(root), 900, snapshot(), [], selected);
      assert.equal(summary(root, name)["complete"], false);
    }
    const output = writer(root, "sample-count-mismatch"); fill(output);
    output.finish(config(root), 900, snapshot({ samples: 4 }), [], cohort());
    assert.equal(summary(root, "sample-count-mismatch")["complete"], false);
    const duplicate = writer(root, "duplicate-edge"); fill(duplicate);
    duplicate.record({ kind: "phase", event: { phase: phases[0]!, atNs: "99" } });
    duplicate.finish(config(root), 900, snapshot(), [], cohort());
    assert.equal(summary(root, "duplicate-edge")["complete"], false);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});
