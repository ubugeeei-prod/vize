import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import {
  verifyCandidateSdkRoles,
  candidateNativeLibraryPins,
  verifyCandidateLibraryRows,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-candidate-source.ts";
import { currentProfileSource } from "../../tools/benchmarks/scripts/cold-typechecker-scale-native-source.ts";
import { resetCandidateCache } from "../../tools/benchmarks/scripts/cold-typechecker-scale-candidate-normal.ts";
import { verifyCandidatePhasePrimary } from "../../tools/benchmarks/scripts/cold-typechecker-scale-candidate-profile.ts";

const text = (file: string) =>
  fs.readFileSync(new URL("../../tools/benchmarks/scripts/" + file, import.meta.url), "utf8");
test("paired source requires original+debug+census before primaries and all native readers before complete admission", () => {
  const normal = text("cold-typechecker-scale-candidate-normal.ts"),
    paired = text("cold-typechecker-scale-candidate-paired.ts"),
    profile = text("cold-typechecker-scale-candidate-profile.ts");
  const check = (n: string, p: string, q: string) => {
    const order = [
      "verifyInheritedOptionsOutput(",
      'const id = "normal-preflight-"',
      'const id = "allocation-"',
      "eligibleInitializedBindingWork !== true",
      "await captureReadBufferCommand(",
      'const id = "invalid-"',
    ];
    for (let index = 1; index < order.length; index++)
      assert(
        n.indexOf(order[index - 1]) >= 0 && n.indexOf(order[index]) > n.indexOf(order[index - 1]),
      );
    assert(n.includes("compareInheritedOptionsRaw(allocationPackets[role], raw)"));
    const ordered = (body: string, before: string, after: string) => {
      assert(body.indexOf(before) >= 0 && body.indexOf(after) > body.indexOf(before));
    };
    ordered(p, "verifyCandidateSources(", "await candidateNormalCommands(");
    ordered(p, "candidateProfileCommands(", "verifyCandidateCommandPopulation(ids, true)");
    assert(p.includes("comparisonQualified: false"));
    ordered(q, "reportCpuProfile(capture)", "archiveCandidateNativeBlock(capture");
    ordered(q, "reportMemoryProfile(capture)", "archiveCandidateNativeBlock(capture");
    ordered(q, "const phasePrimary = verifyCandidatePhasePrimary(", "reportCpuProfile(capture)");
    assert(q.includes('"phase-vize"') && q.includes('"native-profile"'));
  };
  check(normal, paired, profile);
  assert.throws(() =>
    check(
      normal.replace("verifyInheritedOptionsOutput(", "missingOriginalGuard("),
      paired,
      profile,
    ),
  );
  assert.throws(() =>
    check(
      normal.replace(
        "compareInheritedOptionsRaw(allocationPackets[role], raw)",
        "missingOutputParity()",
      ),
      paired,
      profile,
    ),
  );
  assert.throws(() =>
    check(normal, paired.replace("candidateProfileCommands(", "missingNativeBlock("), profile),
  );
  assert.throws(() =>
    check(normal, paired, profile.replace("reportMemoryProfile(capture)", "missingMemoryReader()")),
  );
});

test("normal phase joins its SAME-role primary whole packet after persistence, with no stream/status filtering", () => {
  const primary = {
    status: 0,
    stdout: Buffer.from("whole unknown JSON bytes\n"),
    stderr: Buffer.from("complete progress\n"),
  };
  const phase = { ...primary, signal: null, error: null };
  assert.deepEqual(verifyCandidatePhasePrimary(primary, phase), {
    wholeStatusStdoutStderrEqual: true,
    sameRoleNormalPrimary: true,
  });
  for (const changed of [
    { ...phase, status: 2 },
    { ...phase, stdout: Buffer.concat([phase.stdout, Buffer.from("extra")]) },
    { ...phase, stderr: Buffer.alloc(0) },
    { ...phase, signal: "SIGTERM" },
    { ...phase, error: "spawn failed" },
  ])
    assert.throws(() => verifyCandidatePhasePrimary(primary, changed));
});

test("all seven real SDK4 role checksums stay distinct from mutable source graph pins", () => {
  const lock = Buffer.from(
    currentProfileSource.sdkRoles
      .map(
        (role) =>
          `[[package]]\nname = "${role.name}"\nversion = "${role.version}"\nsource = "${role.source}"\nchecksum = "${role.checksum}"\n`,
      )
      .join("\n"),
  );
  assert.equal(verifyCandidateSdkRoles(lock).length, 7);
  for (const changed of [
    Buffer.from(lock.toString().replaceAll("2.0.0-beta.4", "2.0.0-beta.3")),
    Buffer.from(lock.toString().replace(currentProfileSource.sdkRoles[0].checksum, "0".repeat(64))),
    Buffer.from(
      lock
        .toString()
        .replace(
          'source = "registry+https://github.com/rust-lang/crates.io-index"',
          'source = "git+https://example.invalid"',
        ),
    ),
    Buffer.concat([
      lock,
      Buffer.from('\n[[package]]\nname = "corsa_extra"\nversion = "2.0.0-beta.4"\n'),
    ]),
  ])
    assert.throws(() => verifyCandidateSdkRoles(changed));
});

test("all116 adjacent library metadata are whole and cannot become a loaded-file vector", () => {
  const pins = candidateNativeLibraryPins();
  assert.equal(pins.length, 116);
  assert.doesNotThrow(() => verifyCandidateLibraryRows(pins));
  for (const mutate of [
    (rows: typeof pins) => {
      rows.pop();
    },
    (rows: typeof pins) => {
      rows[0].sha256 = "0".repeat(64);
    },
    (rows: typeof pins) => {
      rows[0].bytes += 1;
    },
    (rows: typeof pins) => {
      rows.reverse();
    },
  ]) {
    const rows = structuredClone(pins);
    mutate(rows);
    assert.throws(() => verifyCandidateLibraryRows(rows));
  }
});

test("cache reset keeps authored/evidence trees and refuses symlink roots; native block can start both absent", (t) => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "candidate-reset-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  for (const name of ["project", "xdg-cache", "tmp"]) fs.mkdirSync(path.join(root, name));
  fs.writeFileSync(path.join(root, "project/App.vue"), "owned input");
  resetCandidateCache(root, false);
  assert(!fs.existsSync(path.join(root, "xdg-cache")) && !fs.existsSync(path.join(root, "tmp")));
  assert(fs.existsSync(path.join(root, "project/App.vue")));
  fs.symlinkSync(path.join(root, "project"), path.join(root, "xdg-cache"), "dir");
  assert.throws(() => resetCandidateCache(root));
  assert(fs.existsSync(path.join(root, "project/App.vue")));
});
