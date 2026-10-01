import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { test } from "node:test";
import {
  observerBuildArgs,
  productObserverSourceIdentity,
  selectObserverArtifact,
  type ObserverSpec,
} from "../differential/observer-build.ts";

const spec: ObserverSpec = {
  product: "linter",
  packageName: "vize_patina",
  exampleName: "lint_history_observer",
  sourcePath: "crates/vize_patina/examples/lint_history_observer.rs",
  probes: [["--contract"]],
};

function fixture(run: (root: string) => void) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-observer-build-contract-"));
  const git = (...args: string[]) => execFileSync("git", args, { cwd: root, stdio: "pipe" });
  try {
    git("init", "--quiet");
    for (const [name, text] of Object.entries({
      "Cargo.lock": "# synthetic lock\n",
      "Cargo.toml": "[workspace]\n",
      "crates/vize_patina/src/lib.rs": "// synthetic product\n",
      [spec.sourcePath]: "// synthetic observer\n",
      "davinci/vize_l1/src/lib.rs": "// synthetic upstream\n",
      "tests/davinci_test_support/src/lib.rs": "// synthetic test provider\n",
    })) {
      const file = path.join(root, name);
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, text);
    }
    git("add", ".");
    git(
      "-c",
      "user.name=Fixture",
      "-c",
      "user.email=fixture@example.invalid",
      "commit",
      "--quiet",
      "-m",
      "test: pin synthetic source",
    );
    run(root);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

void test("observer source identity rejects dirty upstream/test-provider inputs and untracked Rust", () => {
  fixture((root) => {
    const original = productObserverSourceIdentity(root, spec);
    assert.match(original.sourceRevision, /^[a-f0-9]{40}$/);
    for (const source of [
      "davinci/vize_l1/src/lib.rs",
      "tests/davinci_test_support/src/lib.rs",
      spec.sourcePath,
      "Cargo.lock",
    ]) {
      const file = path.join(root, source);
      const before = fs.readFileSync(file);
      fs.appendFileSync(file, "// dirty\n");
      assert.throws(
        () => productObserverSourceIdentity(root, spec),
        /committed Rust\/Cargo inputs/,
      );
      fs.writeFileSync(file, before);
    }
    const untracked = path.join(root, "davinci/vize_l1/src/injected.rs");
    fs.writeFileSync(untracked, "// not source-bound\n");
    assert.throws(() => productObserverSourceIdentity(root, spec), /untracked Rust\/Cargo/);
    fs.unlinkSync(untracked);
    assert.deepEqual(productObserverSourceIdentity(root, spec), original);
  });
});

void test("Cargo evidence requires exact successful package/example/source and rejects old binary substitution", () => {
  fixture((root) => {
    const artifact = {
      reason: "compiler-artifact",
      package_id: `path+${pathToFileURL(path.join(root, "crates/vize_patina")).href}#vize_patina@0.1.0`,
      target: {
        name: spec.exampleName,
        kind: ["example"],
        src_path: path.join(root, spec.sourcePath),
      },
      profile: { test: false },
      features: [],
      executable: path.join(root, "old-executable"),
    };
    const encode = (rows: unknown[]) =>
      Buffer.from(rows.map((row) => JSON.stringify(row)).join("\n"));
    const success = { reason: "build-finished", success: true };
    assert.deepEqual(selectObserverArtifact(encode([artifact, success]), spec, root), artifact);
    assert.throws(() => selectObserverArtifact(encode([artifact]), spec, root), /build-finished/);
    assert.throws(
      () => selectObserverArtifact(encode([artifact, { ...success, success: false }]), spec, root),
      /build-finished/,
    );
    assert.throws(
      () => selectObserverArtifact(encode([artifact, artifact, success]), spec, root),
      /exactly one/,
    );
    for (const wrong of [
      { ...artifact, package_id: artifact.package_id.replace("#vize_patina@", "#wrong_package@") },
      { ...artifact, features: ["legacy-differential"] },
      { ...artifact, profile: { test: true } },
      {
        ...artifact,
        target: { ...artifact.target, src_path: path.join(root, "crates/vize_patina/src/lib.rs") },
      },
    ])
      assert.throws(() => selectObserverArtifact(encode([wrong, success]), spec, root));
  });
});

void test("actual Cargo version-only workspace package IDs remain accepted", () => {
  fixture((root) => {
    const glyph = {
      ...spec,
      product: "formatter",
      packageName: "vize_glyph",
      exampleName: "formatter_observe",
      sourcePath: "crates/vize_glyph/examples/formatter_observe.rs",
    };
    const source = path.join(root, glyph.sourcePath);
    fs.mkdirSync(path.dirname(source), { recursive: true });
    fs.writeFileSync(source, "// synthetic source for captured Cargo shape\n");
    // Real formatter-observer Cargo record captured on 2026-10-01; only its
    // checkout prefix/executable are normalized to this temporary repository.
    const captured = {
      reason: "compiler-artifact",
      package_id: `path+${pathToFileURL(path.join(root, "crates/vize_glyph")).href}#0.429.2`,
      target: {
        kind: ["example"],
        crate_types: ["bin"],
        name: "formatter_observe",
        src_path: source,
        edition: "2024",
        doc: false,
        doctest: false,
        test: false,
      },
      profile: {
        opt_level: "0",
        debuginfo: 2,
        debug_assertions: true,
        overflow_checks: true,
        test: false,
      },
      features: [],
      executable: path.join(root, "formatter_observe"),
    };
    const records = Buffer.from(
      `${JSON.stringify(captured)}\n${JSON.stringify({ reason: "build-finished", success: true })}\n`,
    );
    assert.deepEqual(selectObserverArtifact(records, glyph, root), captured);
    const wrongPath = {
      ...captured,
      package_id: `path+${pathToFileURL(path.join(root, "crates/vize_patina")).href}#0.429.2`,
    };
    assert.throws(
      () =>
        selectObserverArtifact(
          Buffer.from(
            `${JSON.stringify(wrongPath)}\n${JSON.stringify({ reason: "build-finished", success: true })}\n`,
          ),
          glyph,
          root,
        ),
      /package path differs/,
    );
  });
});

void test("observer build recipes retain locked profiles and nested examples without arbitrary targets", () => {
  assert.deepEqual(observerBuildArgs(spec, "ci", false), [
    "build",
    "--locked",
    "--profile",
    "ci",
    "-p",
    "vize_patina",
    "--example",
    "lint_history_observer",
    "--message-format=json-render-diagnostics",
  ]);
  assert(observerBuildArgs(spec, "dev", true).includes("--offline"));
  assert.throws(() => observerBuildArgs(spec, "release", false), /unknown observer build profile/);
  assert.throws(() => observerBuildArgs({ ...spec, sourcePath: "../outside.rs" }, "ci", false));
  const nested = {
    ...spec,
    sourcePath: "crates/vize_patina/examples/lint_history_observer/main.rs",
  };
  assert(observerBuildArgs(nested, "ci", false).includes("lint_history_observer"));
});
