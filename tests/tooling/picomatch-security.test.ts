import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import YAML from "yaml";
import { toolingProgram } from "./support/program-path.ts";
import {
  assertPicomatchLock,
  patchedPicomatchRef,
  picomatchPatches,
  picomatchVersions,
} from "../../tools/support/security/picomatch-lock.ts";
import {
  assertPicomatchPackage,
  installedPicomatchCensus,
  verifyInstalledPicomatch,
  verifyPicomatchPatches,
} from "../../tools/support/security/picomatch-proof.ts";
import {
  pathological,
  verifyPicomatchExtglob,
  type Picomatch,
} from "../../tools/support/security/picomatch-laws.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const workspace = () => YAML.parse(fs.readFileSync(path.join(root, "pnpm-workspace.yaml"), "utf8"));
const lock = () =>
  YAML.parseAllDocuments(fs.readFileSync(path.join(root, "pnpm-lock.yaml"), "utf8"))[1].toJS();
let census: Set<string> | undefined;
const instanceOf = (version: string) =>
  [...(census ??= installedPicomatchCensus(root))].find(
    (directory) =>
      JSON.parse(fs.readFileSync(path.join(directory, "package.json"), "utf8")).version === version,
  )!;

function unpatchedCopy(temporary: string, version: string): string {
  const packageRoot = path.join(temporary, version, "picomatch");
  fs.cpSync(instanceOf(version), packageRoot, { recursive: true });
  const patch = fs.readFileSync(path.join(root, picomatchPatches[version as "4.0.7"].path));
  const result = spawnSync("git", ["apply", "--reverse", "--unsafe-paths", "-"], {
    cwd: packageRoot,
    input: patch,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  return packageRoot;
}

// Measure one pathological match in a child, so an unbounded run is killed.
function timedMatch(packageRoot: string, pattern: string, input: string, timeout: number) {
  const matcher: RegExp = createRequire(import.meta.url)(packageRoot).makeRe(pattern);
  return spawnSync(process.execPath, [toolingProgram("process/measure-regexp-match.cjs")], {
    input: JSON.stringify({ source: matcher.source, flags: matcher.flags, input }),
    encoding: "utf8",
    timeout,
  });
}

test("every installed picomatch copy is the patched source and passes the extglob laws", () => {
  const proof = verifyInstalledPicomatch(root);
  assert.deepEqual(proof.versions, ["2.3.2", "4.0.7"]);
  assert.equal(proof.instances, 2);
  assert.ok(proof.edges > 0);
  assert.equal(proof.negativeChecks, proof.instances * 19);
  assert.equal(proof.positiveChecks, proof.instances * 21);
});

test("pathological single-branch repeated extglobs backtrack in the original and not in the repair", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "picomatch-redos-"));
  try {
    for (const version of picomatchVersions) {
      const patched = instanceOf(version),
        original = unpatchedCopy(temporary, version);
      // 40 characters: the original needs about 2^16 times its 24-character
      // 0.25 s run, so a 1.5 s deadline separates the two by orders of magnitude.
      const input = "a".repeat(40) + "b";
      const killed = timedMatch(original, "+(*a)", input, 1500);
      assert.equal(killed.signal, "SIGTERM", `original ${version} finished: ${killed.stdout}`);
      for (const [pattern, pathologicalInput] of [["+(*a)", input], ...pathological]) {
        const result = timedMatch(patched, pattern, pathologicalInput, 10_000);
        assert.equal(result.status, 0, result.stderr);
        const { m, ms } = JSON.parse(result.stdout);
        assert.equal(m, false);
        assert.ok(ms < 50, `${version} ${pattern} took ${ms} ms`);
      }
    }
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

test("whole-package proof rejects original, changed, extra and symlink bytes", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "picomatch-source-laws-"));
  try {
    for (const version of picomatchVersions) {
      const original = unpatchedCopy(temporary, version);
      assert.throws(() => assertPicomatchPackage(original));
      // The shape laws fail fast on the original instead of hanging.
      const require = createRequire(path.join(original, "package.json"));
      assert.throws(() => verifyPicomatchExtglob(require(original) as Picomatch));
      const packageRoot = path.join(temporary, version, "copy");
      fs.cpSync(instanceOf(version), packageRoot, { recursive: true });
      assert.equal(assertPicomatchPackage(packageRoot), version);
      const file = path.join(packageRoot, "lib/parse.js"),
        bytes = fs.readFileSync(file);
      fs.writeFileSync(file, Buffer.concat([bytes, Buffer.from("\n")]));
      assert.throws(() => assertPicomatchPackage(packageRoot));
      fs.writeFileSync(file, bytes);
      fs.writeFileSync(path.join(packageRoot, "extra.js"), "module.exports={};");
      assert.throws(() => assertPicomatchPackage(packageRoot));
      fs.unlinkSync(path.join(packageRoot, "extra.js"));
      fs.symlinkSync(file, path.join(packageRoot, "extra.js"));
      assert.throws(() => assertPicomatchPackage(packageRoot));
    }
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

test("the actual lock rejects unpatched, aliased, overridden and unreviewed picomatch", () => {
  const actualLock = lock();
  assert.ok(assertPicomatchLock(workspace(), actualLock) > 0);
  const patched = patchedPicomatchRef("4.0.7");
  const firstEdge = (l: any) =>
    Object.values(l.snapshots).find((snapshot: any) => snapshot?.dependencies?.picomatch) as any;
  const mutations: ((w: any, l: any) => void)[] = [
    (w) => delete w.patchedDependencies["picomatch@4.0.7"],
    (w) => delete w.patchedDependencies["picomatch@2.3.2"],
    (w) => (w.patchedDependencies["picomatch@4.0.7"] = "patches/other.patch"),
    (_w, l) => (l.patchedDependencies["picomatch@4.0.7"] = "0".repeat(64)),
    (_w, l) => (l.packages["picomatch@4.0.7"].resolution.integrity = "bad"),
    (_w, l) => (l.packages["picomatch@4.0.8"] = l.packages["picomatch@4.0.7"]),
    (_w, l) => (l.snapshots["picomatch@4.0.7"] = {}),
    (_w, l) => (firstEdge(l).dependencies.picomatch = "4.0.7"),
    (_w, l) => (firstEdge(l).dependencies.alias = "picomatch@" + patched),
    (_w, l) => (l.snapshots["alien@1.0.0"] = { dependencies: { fdir: "6.5.0(picomatch@4.0.7)" } }),
    (_w, l) => (l.snapshots["alien@1.0.0(picomatch@4.0.7)"] = {}),
    (_w, l) => (l.importers["npm/builder/vite"].dependencies.picomatch.version = "4.0.7"),
    (w) => (w.audit = { ignoreGhsas: ["GHSA-fc5q-39xc-vjpf"] }),
    (w) => (w.overrides.picomatch = "4.0.7"),
    (_w, l) => (l.overrides = { ...l.overrides, "anymatch>picomatch": "2.3.2" }),
  ];
  for (const field of ["dependencies", "devDependencies", "optionalDependencies"])
    mutations.push((_w, l) => {
      l.importers["."][field] ??= {};
      l.importers["."][field].alias = { specifier: "npm:picomatch@4.0.7", version: patched };
    });
  for (const mutate of mutations) {
    const w = workspace(),
      l = structuredClone(actualLock);
    mutate(w, l);
    assert.throws(() => assertPicomatchLock(w, l), String(mutate));
  }
});

test("patch attestation rejects missing and changed bytes", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "picomatch-patch-laws-"));
  try {
    assert.throws(() => verifyPicomatchPatches(temporary));
    fs.mkdirSync(path.join(temporary, "patches"));
    for (const version of picomatchVersions) {
      const relative = picomatchPatches[version].path;
      fs.copyFileSync(path.join(root, relative), path.join(temporary, relative));
    }
    verifyPicomatchPatches(temporary);
    fs.appendFileSync(path.join(temporary, picomatchPatches["2.3.2"].path), "\n");
    assert.throws(() => verifyPicomatchPatches(temporary));
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
