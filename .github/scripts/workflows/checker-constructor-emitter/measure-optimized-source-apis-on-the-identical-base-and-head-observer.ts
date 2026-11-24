import { requiredEnv } from "../shared/files.ts";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import path from "node:path";
// Cargo 1.99 writes <profile>/deps/<name>-<hash>; Cargo 1.100 writes
// <profile>/build/<package>/<hash>/out/<name>-<hash>.
const unitExecutables = (profile: string, pattern: RegExp) => {
  const directories = [path.join(profile, "deps")];
  const units = path.join(profile, "build");
  for (const pkg of existsSync(units) ? readdirSync(units) : [])
    for (const hash of readdirSync(path.join(units, pkg)))
      directories.push(path.join(units, pkg, hash, "out"));
  return directories
    .filter((directory) => existsSync(directory))
    .flatMap((directory) =>
      readdirSync(directory)
        .filter((name) => pattern.test(name))
        .map((name) => path.join(directory, name)),
    );
};
const directory = `${process.env.RUNNER_TEMP}/constructor-emitter`;
interface Row {
  case: string;
  mapper: unknown;
  source: string;
  iterationsPerSample: number;
  packet: { text?: string; virtual_ts: string };
  allocationCallsPerSample: number;
  requestedBytesPerSample: number;
  samplesNanoseconds: number[];
}
interface Profile {
  rows: Row[];
  workingDirectory: string;
  scope: string;
  allocationDefinition: string;
}
const read = (name: string): Profile => JSON.parse(readFileSync(`${directory}/${name}`, "utf8"));
const base = read("profile-base.json"),
  head = read("profile-head.json");
assert.equal(base.rows.length, 6);
assert.equal(head.rows.length, 6);
assert.equal(base.workingDirectory, head.workingDirectory);
const median = (values: number[]) => [...values].sort((a, b) => a - b)[1];
const rows = head.rows.map((current, index) => {
  const previous = base.rows[index];
  assert.deepEqual(
    [current.case, current.mapper, current.source, current.iterationsPerSample],
    [previous.case, previous.mapper, previous.source, previous.iterationsPerSample],
  );
  if (current.case !== "literal") assert.deepEqual(current.packet, previous.packet);
  else {
    assert.notDeepEqual(
      current.packet,
      previous.packet,
      "quoted generic fixture must reach the repaired production projection",
    );
    const expectedProps =
      "export type Props<T extends 'a,b' = 'a,b', U extends `x,${string}` = 'x,y', F extends (left: T, right: U) => readonly [T, U] = (left: T, right: U) => readonly [T, U]> = Awaited<ReturnType<typeof __setup<T, U, F>>>[\"__vize_setup_props\"];";
    const declarations = (packet: Row["packet"]) =>
      (packet.text ?? packet.virtual_ts)
        .split("\n")
        .filter((line) => line.startsWith("export type Props<"));
    assert.deepEqual(
      declarations(current.packet),
      [expectedProps],
      "head must emit the complete independently authored literal contract",
    );
    assert.notDeepEqual(
      declarations(previous.packet),
      [expectedProps],
      "baseline must retain the original quoted-comma corruption",
    );
  }
  return {
    case: current.case,
    mapper: current.mapper,
    iterationsPerSample: current.iterationsPerSample,
    baseAllocationCalls: previous.allocationCallsPerSample,
    headAllocationCalls: current.allocationCallsPerSample,
    baseRequestedBytes: previous.requestedBytesPerSample,
    headRequestedBytes: current.requestedBytesPerSample,
    baseMedianNanoseconds: median(previous.samplesNanoseconds),
    headMedianNanoseconds: median(current.samplesNanoseconds),
  };
});
const sha256 = (path: string) => createHash("sha256").update(readFileSync(path)).digest("hex");
const binaries = ["base", "head"].map((side) => {
  const target = path.join(requiredEnv("RUNNER_TEMP"), `constructor-target-${side}`);
  const candidates = unitExecutables(
    path.join(target, "ci-opt"),
    /^generic_parameter_profile-[a-f0-9]+$/,
  );
  assert.equal(candidates.length, 1);
  const binary = candidates[0];
  return { side, binary, sha256: sha256(binary) };
});
assert.notEqual(binaries[0].binary, binaries[1].binary);
assert.notEqual(
  binaries[0].sha256,
  binaries[1].sha256,
  "the repaired literal control must come from different source executables",
);
const receipt = {
  base: process.env.BASE_SHA,
  head: process.env.HEAD_SHA,
  binaries,
  profile: "ci-opt",
  observerSha256: sha256("tools/benchmarks/probes/generic_parameter_profile.rs"),
  basePacketSha256: sha256(`${directory}/profile-base.json`),
  headPacketSha256: sha256(`${directory}/profile-head.json`),
  scope: head.scope,
  workingDirectory: head.workingDirectory,
  allocationDefinition: head.allocationDefinition,
  rows,
};
writeFileSync(`${directory}/profile-receipt.json`, JSON.stringify(receipt, null, 2) + "\n");
console.log(JSON.stringify(receipt));
