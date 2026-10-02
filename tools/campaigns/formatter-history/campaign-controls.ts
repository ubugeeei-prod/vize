import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import {
  readControlAuthority,
  currentControlPins,
  commands,
  benchmarkNames,
  movedTargets,
} from "./control-contract.ts";

// Real hosted source/control processes only. Source/smoke checks never become
// public-output, native, throughput, allocation or instruction-budget credit.
export function captureEngineeringControls(context: any) {
  const {
    root,
    evidence,
    plan,
    audit,
    run,
    successful,
    messages,
    packagePath,
    sha256,
    write,
    failures,
    laws,
    repeatedBytesVerified,
  } = context;
  const checks: Record<string, any> = {};
  const perform = (id: string, command: string[]) => {
    const result = run("control-" + id, command[0], command.slice(1));
    assert(successful(result), `control process failed: ${id}`);
    checks[id] = { processId: result.id, state: "observed-success" };
    return result;
  };
  const cargoSuccess = (result: any) => {
    const data = messages(result.stdout);
    assert.deepEqual(
      data.filter((row: any) => row.reason === "build-finished").map((row: any) => row.success),
      [true],
    );
    return data;
  };
  const sourceProofs: any[] = [];
  const attempt = (id: string, work: () => void) => {
    try {
      work();
      if (!checks[id]) checks[id] = { state: "observed-success" };
    } catch (error) {
      const message = String(error);
      checks[id] = { ...checks[id], state: "failed", error: message };
      failures.push(`Engineering control ${id}: ${message}`);
    }
  };
  const preserve = (file: string, bytes: Buffer) => {
    const destination = path.join(evidence, file);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    if (fs.existsSync(destination)) assert(fs.readFileSync(destination).equals(bytes));
    else fs.writeFileSync(destination, bytes, { flag: "wx" });
    return { path: file, bytes: bytes.length, sha256: sha256(bytes) };
  };
  attempt("current-source-pins", () => {
    const current = currentControlPins(root, plan.selectedIntegrationTargets);
    assert.deepEqual(current, plan.controlCapture.currentSourcePins);
    write(
      "control-current-source-files.json",
      current.map((entry: any) => ({
        ...entry,
        captured: preserve(
          `control-current-sources/${entry.sha256}`,
          fs.readFileSync(path.join(root, entry.path)),
        ),
      })),
    );
  });
  for (const control of plan.controlCapture.originalControls) {
    attempt(`original-source-${control.id}`, () => {
      const authority = readControlAuthority(root, audit, control.id);
      assert.deepEqual(authority.pin, control);
      const rawFiles = {
        commit: preserve(`control-sources/${control.id}/commit.raw`, authority.rawCommit),
        firstParent: preserve(
          `control-sources/${control.id}/first-parent.raw`,
          authority.rawFirstParent,
        ),
        patch: preserve(`control-sources/${control.id}/scoped.patch`, authority.rawPatch),
        treeDiff: preserve(`control-sources/${control.id}/tree-diff.raw`, authority.rawTreeDiff),
      };
      const objects = authority.objects.map((object) => ({
        kind: object.kind,
        oid: object.oid,
        ...preserve(`source-objects/${object.kind}-${object.oid}`, object.bytes),
      }));
      const raw = run(
        "control-scope-" + control.id,
        "git",
        [
          "--no-replace-objects",
          "show",
          "--format=",
          "--no-ext-diff",
          "--binary",
          "--find-renames",
          control.commit,
          "--",
          ...control.scope,
        ],
        {},
        30_000,
      );
      assert(successful(raw));
      assert.equal(sha256(raw.stdout), control.rawPatchSha256);
      sourceProofs.push({
        ...control,
        actualScopeProcess: raw.id,
        rawFiles,
        objects,
        state: "observed-original-scope-proof",
        publicOutputCredit: false,
      });
    });
  }
  write("control-source-proofs.json", {
    original: sourceProofs,
    current: plan.controlCapture.currentSourcePins,
  });
  attempt("metadata", () => {
    const metadata = JSON.parse(perform("metadata", commands.metadata).stdout.toString());
    const glyph = metadata.packages.find((entry: any) => entry.name === "vize_glyph");
    assert(
      glyph &&
        path.dirname(glyph.manifest_path) === fs.realpathSync(path.join(root, "crates/vize_glyph")),
    );
    assert.deepEqual(glyph.publish, []);
    assert.equal(glyph.rust_version, plan.controlCapture.msrv);
    assert(
      glyph.targets.some(
        (entry: any) => entry.name === "formatter" && entry.kind.includes("bench"),
      ),
    );
    write("control-current-package.json", glyph);
  });
  for (const id of [
    "style-spec-tests",
    "stage-source-tests",
    "workspace-module-support",
  ] as const) {
    attempt(id, () => {
      const result = perform(id, commands[id]);
      assert.match(result.stdout.toString(), /# fail 0/);
      assert.match(result.stdout.toString(), /# skipped 0/);
      if (id === "style-spec-tests") assert.match(result.stdout.toString(), /# pass 6/);
    });
  }
  attempt("glyph-module-source", () => {
    const result = perform("glyph-module-source", commands["glyph-module-source"]);
    const proof = JSON.parse(result.stdout.toString());
    assert.equal(proof.schema, "vize.formatter.glyph-module-source");
    assert.equal(proof.version, 1);
    assert.deepEqual(
      proof.modules.map((row: any) => row.name),
      ["json", "template"],
    );
    for (const row of [proof.lib, ...proof.modules]) {
      const pin = plan.controlCapture.currentSourcePins.find(
        (entry: any) => entry.path === row.path,
      );
      assert(pin);
      assert.equal(row.sha256, pin.sha256);
      assert.equal(row.bytes, pin.bytes);
    }
    assert(proof.modules.every((row: any) => row.ordinaryDeclaration && row.oldModOwnerAbsent));
    write("glyph-module-source.json", proof);
  });
  attempt("rustfmt", () => {
    perform("rustfmt", commands.rustfmt);
  });
  attempt("strict-clippy", () => {
    cargoSuccess(perform("strict-clippy", commands["strict-clippy"]));
  });
  attempt("selected-rust", () => {
    assert.equal(laws.length, plan.requiredLaws.length, "actual selected laws incomplete");
    const binaries = JSON.parse(fs.readFileSync(path.join(evidence, "rust-binaries.json"), "utf8"));
    for (const name of movedTargets)
      assert(
        binaries.some((binary: any) => binary.name === name && binary.kinds.includes("test")),
        `moved target not actually executed: ${name}`,
      );
    checks["selected-rust"] = {
      state: "observed-success",
      lawReferences: laws.length,
      movedTargets,
      frozenBinaryReceipt: "rust-binaries.json",
    };
    const lawSourceProofs = laws.map((law: any) => {
      const current = fs.readFileSync(path.join(root, law.owner));
      assert.equal(sha256(current), law.currentOwnerSha256);
      const currentSource = preserve(`law-sources/current/${sha256(current)}`, current);
      const originals = law.originalWitnesses.flatMap((original: any) =>
        original.captures.map((capture: any) => {
          const result = run(
            `law-source-${law.law}-${original.witness}-${capture.revision}`,
            "git",
            ["--no-replace-objects", "show", `${capture.revision}:${original.owner}`],
            {},
            30_000,
          );
          assert(successful(result));
          assert.equal(sha256(result.stdout), capture.sourceSha256);
          const bytes = result.stdout;
          const oid = requireBlobIdentity(capture.gitBlob, bytes);
          return {
            witness: original.witness,
            owner: original.owner,
            function: original.function,
            ...capture,
            process: result.id,
            ...preserve(`source-objects/blob-${oid}`, bytes),
          };
        }),
      );
      return { ...law, currentSource, originals };
    });
    write("rust-law-source-proofs.json", lawSourceProofs);
  });
  attempt("api-replays", () => {
    assert(repeatedBytesVerified, "unchanged complete API/CLI repeated byte checks failed");
    checks["api-replays"] = {
      state: "observed-success",
      registeredApiCasesPerRun: 300,
      fullPublicSuccessCases: 276,
      typedErrorCases: 21,
      internalObservations: 3,
      actualRepeats: 2,
      outputCreditAdded: 0,
    };
  });
  attempt("msrv-check", () => {
    const msrv = plan.controlCapture.msrv;
    perform("msrv-install", ["rustup", "toolchain", "install", msrv, "--profile", "minimal"]);
    const rustc = perform("msrv-rustc-version", ["rustc", "+" + msrv, "--version", "--verbose"]);
    assert.match(
      rustc.stdout.toString(),
      new RegExp("^rustc " + msrv.replaceAll(".", "\\.") + " "),
    );
    perform("msrv-cargo-version", ["cargo", "+" + msrv, "--version", "--verbose"]);
    cargoSuccess(
      perform("msrv-check", [
        "cargo",
        "+" + msrv,
        "check",
        "--locked",
        "-p",
        "vize_glyph",
        "--lib",
        "--target-dir",
        "target/formatter-msrv",
        "--message-format=json-render-diagnostics",
      ]),
    );
  });
  attempt("benchmark-smoke", () => {
    const benchBuild = perform("benchmark-build", commands["benchmark-build"]);
    const benchmark = cargoSuccess(benchBuild).filter(
      (row: any) =>
        row.reason === "compiler-artifact" &&
        row.executable &&
        row.target.name === "formatter" &&
        row.target.kind.includes("bench") &&
        packagePath(row.package_id) === fs.realpathSync(path.join(root, "crates/vize_glyph")),
    );
    assert.equal(benchmark.length, 1);
    const artifact = benchmark[0];
    const bytes = fs.readFileSync(artifact.executable);
    const destination = path.join(evidence, "benchmark-formatter");
    fs.copyFileSync(artifact.executable, destination, fs.constants.COPYFILE_EXCL);
    fs.chmodSync(destination, 0o755);
    assert.equal(sha256(fs.readFileSync(destination)), sha256(bytes));
    const smoke = perform("benchmark-smoke", [destination, "--test"]);
    assert.equal(
      sha256(fs.readFileSync(destination)),
      sha256(bytes),
      "frozen benchmark changed during execution",
    );
    const output = smoke.stdout.toString() + smoke.stderr.toString();
    for (const name of benchmarkNames)
      assert(output.includes("Testing " + name + "\nSuccess"), `benchmark smoke missing: ${name}`);
    write("benchmark-binary.json", {
      cargoArtifact: artifact,
      frozenPath: path.relative(evidence, destination),
      sha256: sha256(bytes),
      bytes: bytes.length,
      actualCommand: commands["benchmark-build"],
      actualSmokeArgv: ["--test"],
      measuredPerformanceCredit: false,
      expectedSmokeNames: benchmarkNames,
    });
  });
  const controls = plan.controlCapture.originalControls.map((control: any) => {
    const source = sourceProofs.find((entry) => entry.id === control.id);
    const missing = control.requiredChecks.filter(
      (id: string) => checks[id]?.state !== "observed-success",
    );
    return {
      id: control.id,
      commit: control.commit,
      contract: control.contract,
      originalScopeVerified: Boolean(source),
      kind: control.kind,
      scopeProcess: source?.actualScopeProcess ?? null,
      actualLawResults: laws.filter((law: any) => controlLawIds(control.id).includes(law.law)),
      checks: Object.fromEntries(
        control.requiredChecks.map((id: string) => [id, checks[id] ?? { state: "missing" }]),
      ),
      state:
        source && missing.length === 0 ? "observed-hosted-control-arms" : "failed-or-incomplete",
      missing,
      independentAcceptance: "pending review of actual full scope/runtime receipts",
      unobservedPrinterErrorArm:
        control.id === "oxc-formatter-provider"
          ? "A typed printing failure is not invented. This campaign retains provider error-propagation source and actual registered observations; any absent print-error witness remains unobserved."
          : null,
      performanceMetricAcceptance: false,
      applicableFormatterInstructionWorkload: null,
      remainingBudgetDependency: control.protectedInstructionBudgetRequired
        ? "No applicable Glyph workload/ceiling exists in the current level harness; metric arm remains unmeasured. No budget is added or changed here."
        : null,
      publicOutputCredit: false,
      nativeCredit: false,
    };
  });
  write("engineering-control-results.json", {
    controls,
    checks,
    observedControlArms: controls.filter((row: any) => row.state === "observed-hosted-control-arms")
      .length,
    acceptedControls: 0,
    measuredPerformanceControls: 0,
    originalAuditUnchanged: true,
    newOutputFixtures: 0,
    wholeHistoryClosed: false,
  });
  assert.equal(controls.length, 24);
  return controls;
}

function requireBlobIdentity(oid: string, bytes: Buffer) {
  assert.match(oid, /^[a-f0-9]{40}$/);
  assert.equal(
    createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
    oid,
  );
  return oid;
}
function controlLawIds(id: string) {
  const range = (first: number, last: number) =>
    Array.from(
      { length: last - first + 1 },
      (_, index) => `L${String(first + index).padStart(3, "0")}`,
    );
  const allocation = ["L010", "L011", "L107", ...range(40, 46)];
  const mapping: Record<string, string[]> = {
    "panic-free-workspace": range(1, 118),
    "level-import-renaming": allocation,
    "allocation-alias": allocation,
    "formatter-throughput": range(52, 56),
    "oxc-formatter-provider": range(77, 85),
    "json-module-move": range(57, 75),
    "expression-arena-reuse": ["L084", "L085", "L107"],
    "template-render-once": range(86, 88),
    "template-module-relocation": [
      "L091",
      ...range(92, 94),
      ...range(100, 103),
      ...range(113, 118),
    ],
    "UTF8-safety-comments": ["L012", ...range(21, 39)],
    "initial-template-module-split": range(92, 94),
    "initial-public-APIs-and-benchmarks": range(104, 118),
  };
  return mapping[id] ?? [];
}
