import { requiredEnv } from "../shared/files.ts";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync, realpathSync, writeFileSync } from "node:fs";
import path from "node:path";
const capture = path.join(requiredEnv("RUNNER_TEMP"), "constructor-emitter");
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
const sha256 = (file: string) => createHash("sha256").update(readFileSync(file)).digest("hex");
const rows = ["base", "head"].map((side) => {
  const target = realpathSync(path.join(requiredEnv("RUNNER_TEMP"), `constructor-target-${side}`));
  const candidates = unitExecutables(
    path.join(target, "ci"),
    /^constructor_emission_projection-[a-f0-9]+$/,
  );
  assert.equal(candidates.length, 1);
  const binary = realpathSync(candidates[0]);
  assert.ok(binary.startsWith(target + path.sep));
  return {
    side,
    source: side === "base" ? process.env.BASE_SHA : process.env.HEAD_SHA,
    target,
    binary,
    sha256: sha256(binary),
  };
});
assert.notEqual(rows[0].target, rows[1].target);
assert.notEqual(rows[0].binary, rows[1].binary);
writeFileSync(path.join(capture, "compiled-observers.json"), JSON.stringify(rows, null, 2) + "\n");
