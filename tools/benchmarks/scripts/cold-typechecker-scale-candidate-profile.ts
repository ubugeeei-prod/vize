/** Final eight target commands: reuse the reviewed four-command block separately for each role. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { coldScaleCommand } from "./cold-typechecker-scale-command.ts";
import { captureThenReplay } from "./cold-typechecker-scale-native-command.ts";
import { assertLinuxX64Image, type NativeImage } from "./cold-typechecker-scale-native-custody.ts";
import { reportCpuProfile } from "./cold-typechecker-scale-cpu-report.ts";
import { reportMemoryProfile } from "./cold-typechecker-scale-memory-report.ts";
import { archiveCandidateNativeBlock } from "./cold-typechecker-scale-candidate-native.ts";
import { verifyCandidateInputSeals } from "./cold-typechecker-scale-candidate-inputs.ts";
import {
  resetCandidateCache,
  type CandidateBuild,
} from "./cold-typechecker-scale-candidate-normal.ts";
import type { CandidateSources } from "./cold-typechecker-scale-candidate-source.ts";
import { compareInheritedOptionsRaw } from "./cold-typechecker-scale-candidate-originals.ts";

const hash = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const read = (root: string, file: string) =>
  JSON.parse(fs.readFileSync(path.join(root, file), "utf8"));
const nativeSha = "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70";

export function verifyCandidatePhasePrimary(
  primary: { status: number | null; stdout: Buffer; stderr: Buffer },
  phase: {
    status: number | null;
    signal: string | null;
    error: string | null;
    stdout: Buffer;
    stderr: Buffer;
  },
) {
  assert.equal(phase.signal, null);
  assert.equal(phase.error, null);
  compareInheritedOptionsRaw(primary, phase);
  return { wholeStatusStdoutStderrEqual: true, sameRoleNormalPrimary: true };
}

function providerImages(provider: string) {
  const lock = read(provider, "package-lock.json") as {
    packages: Record<string, { version: string; integrity?: string }>;
  };
  const rows: NativeImage[] = [];
  for (const [packagePath, entry, role, required] of [
    ["node_modules/@typescript/typescript-linux-x64", "lib/tsc", "invoked-native", true],
    [
      "node_modules/vize/node_modules/@typescript/typescript-linux-x64",
      "lib/tsc",
      "public-native",
      false,
    ],
    ["node_modules/@vizejs/native-linux-x64-gnu", null, "public-vize-gnu", false],
    ["node_modules/@vizejs/native-linux-x64-musl", null, "public-vize-musl", false],
    ["node_modules/@oxc-transform/binding-linux-x64-gnu", null, "public-oxc-gnu", false],
    ["node_modules/@oxc-transform/binding-linux-x64-musl", null, "public-oxc-musl", false],
  ] as const) {
    const manifest = path.join(provider, packagePath, "package.json");
    if (!fs.existsSync(manifest)) {
      assert.equal(required, false);
      continue;
    }
    const pkg = JSON.parse(fs.readFileSync(manifest, "utf8"));
    assert.equal(pkg.version, lock.packages[packagePath]?.version);
    assert(lock.packages[packagePath].integrity);
    const relative = entry ?? pkg.main;
    assert.equal(typeof relative, "string");
    if (entry === null)
      assert(
        relative.endsWith(".node") &&
          [path.basename(relative), "./" + path.basename(relative)].includes(relative),
      );
    const root = fs.realpathSync(path.join(provider, packagePath)),
      file = fs.realpathSync(path.join(root, relative));
    assert(file.startsWith(root + path.sep));
    const sha256 = hash(file);
    assertLinuxX64Image(file, sha256);
    rows.push({ path: file, sha256, role });
  }
  assert.equal(rows.find((row) => row.role === "invoked-native")?.sha256, nativeSha);
  return rows;
}

export function candidateProfileCommands(options: {
  provider: string;
  out: string;
  common: string;
  corpus: { app: string; entries: { path: string }[] };
  frozen: CandidateSources;
  normal: Record<"base" | "head", CandidateBuild & { receipt: string }>;
  custody: Record<"base" | "head", CandidateBuild & { receipt: string }>;
  helpers?: { before: Buffer; after: Buffer };
  primaryRaw: Record<"base" | "head", { status: number | null; stdout: Buffer; stderr: Buffer }>;
  enter: (id: string) => void;
}) {
  const { provider, out, common, corpus, frozen, normal, custody, enter } = options;
  const images = providerImages(provider),
    profiles = [],
    nativeRoot = path.join(out, "native");
  fs.mkdirSync(nativeRoot);
  const capture = path.join(out, "native-current");
  for (const role of ["base", "head"] as const) {
    resetCandidateCache(common, false);
    const selected = coldScaleCommand({
      provider,
      out: common,
      app: corpus.app,
      tool: "cli",
      binary: custody[role].binary,
    });
    assert.equal(hash(selected.backend), nativeSha);
    assert.equal(custody[role].sourceSha, normal[role].sourceSha);
    assert.equal(custody[role].treeSha, normal[role].treeSha);
    assert.deepEqual(custody[role].compilerEnvironment, normal[role].compilerEnvironment);
    assert.equal(custody[role].rustc, normal[role].rustc);
    assert.equal(custody[role].cargo.version, normal[role].cargo.version);
    assert.deepEqual(custody[role].cargo.args, [
      ...normal[role].cargo.args,
      "--features",
      "cache-custody",
    ]);
    let captured = false;
    try {
      captureThenReplay({
        profile: true,
        sourceSha: frozen[role].sha,
        sourceTree: frozen[role].tree,
        buildReceipt: custody[role].receipt,
        phaseExecutable: normal[role].binary,
        phaseBuildReceipt: normal[role].receipt,
        providerLockSha256: "9c2974672b5f5358e643276e6ec98c6084681ef609e8f25922ba4aa7adb0f0a6",
        vizeExecutable: selected.command.executable,
        vizeArgs: selected.command.args,
        backend: selected.backend,
        backendSha256: nativeSha,
        project: path.join(common, "project"),
        provider,
        cwd: corpus.app,
        out: capture,
        expectedFiles: corpus.entries.map((row) => path.join(common, "project", row.path)),
        metadataOnlyImages: images,
        runtimeRoots: [path.join(common, "xdg-cache"), path.join(common, "tmp")],
        candidateBuildPin: {
          sha: frozen[role].sha,
          tree: frozen[role].tree,
          cargo: frozen.cargo.map((row) => ({ file: row.file, sha256: row.pin.sha256 })),
        },
      });
      captured = true;
    } finally {
      // Each status is persisted raw-first by its own command. Missing later stages stay unclaimed.
      let missing = false;
      for (const [suffix, status] of [
        ["phase", "phase-vize"],
        ["custody", "vize"],
        ["bare", "native-replay"],
        ["profile", "native-profile"],
      ]) {
        if (fs.existsSync(path.join(capture, status + ".status.json"))) {
          assert(!missing, "non-prefix native command receipt population");
          enter(`native-${role}-${suffix}`);
        } else missing = true;
      }
    }
    assert(captured);
    const phasePrimary = verifyCandidatePhasePrimary(options.primaryRaw[role], {
      ...read(capture, "phase-vize.status.json"),
      stdout: fs.readFileSync(path.join(capture, "phase-vize.stdout.bin")),
      stderr: fs.readFileSync(path.join(capture, "phase-vize.stderr.bin")),
    });
    // These nine/CPU reader calls are post-exit Go readers, not target commands or timing samples.
    reportCpuProfile(capture);
    reportMemoryProfile(capture);
    const archive = path.join(nativeRoot, role);
    const archived = archiveCandidateNativeBlock(capture, archive);
    profiles.push({ role, archive, archived, phasePrimary });
  }
  const before = path.join(nativeRoot, "base"),
    after = path.join(nativeRoot, "head");
  for (const stream of ["stdout", "stderr"]) {
    assert(
      fs
        .readFileSync(path.join(before, `native-replay.${stream}.bin`))
        .equals(fs.readFileSync(path.join(after, `native-replay.${stream}.bin`))),
      "whole native vector changed between roles",
    );
    assert(
      fs
        .readFileSync(path.join(before, `phase-vize.${stream}.bin`))
        .equals(fs.readFileSync(path.join(after, `phase-vize.${stream}.bin`))),
      "whole normal phase output changed",
    );
  }
  assert.equal(
    read(before, "native-replay.status.json").status,
    read(after, "native-replay.status.json").status,
  );
  const baseRequest = read(before, "native-profile.request.json"),
    headRequest = read(after, "native-profile.request.json");
  assert.deepEqual(
    headRequest,
    baseRequest,
    "same physical captured program/argv/env; only native image PID suffix is process-specific",
  );
  const inputProof = verifyCandidateInputSeals(
    read(before, "materialized-before.json"),
    read(after, "materialized-before.json"),
    path.join(common, "xdg-cache"),
    options.helpers ?? null,
    options.helpers !== undefined,
    path.join(common, "tmp"),
  );
  return {
    profiles,
    inputProof,
    scope:
      "one untimed CPU/four-sampled-memory profile per role; no phase inference from the eight primary timings or native-loaded-file census",
  };
}
