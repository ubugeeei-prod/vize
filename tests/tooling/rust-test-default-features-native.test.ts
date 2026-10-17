import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import type { CargoMetadata } from "../../tools/support/compat/github/plan-affected-rust.ts";
import { rustTestTargetProjection } from "../../tools/support/compat/github/run-rust-test-cohort.ts";
import {
  rustTestCohortForBinary,
  rustTestMatrix,
} from "../../tools/support/compat/github/rust-test-cohorts.ts";
import { prDefaultFeatures } from "../../tools/support/compat/github/rust-test-default-features.ts";

type Artifact = {
  reason: string;
  package_id: string;
  target: { name: string; kind: string[] };
  features: string[];
  profile: { test: boolean };
  executable: string | null;
};

test(
  "native Cargo retains every default test identity and normal example with the same dev features",
  { timeout: 30_000 },
  (t) => {
    const directory = mkdtempSync(join(tmpdir(), "vize-default-targets-"));
    t.after(() => rmSync(directory, { recursive: true, force: true }));
    const toolchain = readFileSync(
      new URL("../../rust-toolchain.toml", import.meta.url),
      "utf8",
    ).match(/^channel = "([^"]+)"/m)?.[1];
    assert.ok(toolchain);
    const env = {
      ...process.env,
      RUSTUP_TOOLCHAIN: toolchain,
      CARGO_TARGET_DIR: join(directory, "target"),
      CARGO_NET_OFFLINE: "true",
    };
    const cargo = (...args: string[]) =>
      execFileSync("cargo", args, {
        cwd: directory,
        env,
        encoding: "utf8",
        maxBuffer: 8 * 1024 * 1024,
        stdio: ["ignore", "pipe", "pipe"],
      });
    const files = {
      "Cargo.toml":
        '[workspace]\nmembers=["canon", "l1"]\nexclude=["support"]\nresolver="3"\n[profile.ci]\ninherits="test"\ndebug=0\n',
      "canon/Cargo.toml": `[package]
name="vize_canon"
version="0.0.0"
edition="2024"
[features]
default=["native"]
native=["transitive"]
transitive=[]
legacy=[]
[dev-dependencies]
probe_support={path="../support",features=["dev"]}
[[test]]
name="native_case"
required-features=["native"]
[[test]]
name="legacy_case"
required-features=["legacy"]
`,
      "canon/src/lib.rs": "#[test]\nfn unit() { assert_eq!(probe_support::enabled(), 7); }\n",
      "canon/tests/native_case.rs":
        '#[test]\nfn native_case() { assert!(cfg!(feature="native")); }\n',
      "canon/tests/legacy_case.rs": 'compile_error!("an inactive feature must not be forced");\n',
      "canon/examples/normal.rs": "fn main() { assert_eq!(probe_support::enabled(), 7); }\n",
      "l1/Cargo.toml": '[package]\nname="vize_l1"\nversion="0.0.0"\nedition="2024"\n',
      "l1/src/lib.rs": "#[test]\nfn unit() {}\n",
      "l1/tests/compiler_case.rs": "#[test]\nfn compiler_case() {}\n",
      "support/Cargo.toml":
        '[package]\nname="probe_support"\nversion="0.0.0"\nedition="2024"\n[features]\ndev=[]\n',
      "support/src/lib.rs":
        '#[cfg(not(feature="dev"))]\ncompile_error!("dev features must stay enabled");\npub fn enabled() -> u8 { 7 }\n',
    };
    for (const [path, content] of Object.entries(files)) {
      mkdirSync(dirname(join(directory, path)), { recursive: true });
      writeFileSync(join(directory, path), content);
    }
    cargo("generate-lockfile", "--offline");
    const metadata: CargoMetadata = JSON.parse(
      cargo("metadata", "--no-deps", "--format-version", "1", "--locked", "--offline"),
    );
    const full: unknown = JSON.parse(
      cargo("metadata", "--format-version", "1", "--locked", "--offline"),
    );
    const selected = metadata.packages.map((pkg) => pkg.name);
    const options = [
      "--cargo-profile",
      "ci",
      "--timings",
      "--cargo-message-format=json-render-diagnostics",
      "--profile",
      "pr",
      "-E",
      "all()",
      "--no-tests=pass",
    ];
    const proof = prDefaultFeatures(metadata, selected, options, "pull_request", () => full);
    assert.ok(proof, "the real full-workspace/default metadata must be recognized");
    assert.equal(proof.requiredFeatures("vize_canon", ["native"]), "active");
    assert.equal(proof.requiredFeatures("vize_canon", ["legacy"]), "inactive");
    assert.equal(proof.requiredFeatures("vize_canon", ["transitive"]), "unknown");
    const roots = selected.flatMap((pkg) => ["--package", pkg]);
    const artifacts = (command: string, args: string[]) =>
      cargo(
        command,
        ...roots,
        "--profile",
        "ci",
        "--locked",
        "--offline",
        "--message-format=json",
        ...args,
      )
        .split("\n")
        .filter(Boolean)
        .map((line) => JSON.parse(line) as Artifact)
        .filter((packet) => packet.reason === "compiler-artifact");
    const baseline = artifacts("test", ["--no-run"]);
    const names = new Map(metadata.packages.map((pkg) => [pkg.id, pkg.name]));
    const identity = (artifact: Artifact) =>
      `${artifact.package_id}:${artifact.target.kind.join(",")}:${artifact.target.name}`;
    const testIdentities = (items: Artifact[]) =>
      items
        .filter((item) => item.profile.test && item.executable)
        .flatMap((item) =>
          execFileSync(item.executable!, ["--list", "--format", "terse"], { encoding: "utf8" })
            .trim()
            .split("\n")
            .filter(Boolean)
            .map((name) => `${identity(item)}:${name}`),
        )
        .sort();
    const projected: Artifact[] = [];
    for (const { cohort } of rustTestMatrix(metadata, selected).include) {
      const projection = rustTestTargetProjection(metadata, selected, cohort, proof);
      assert.equal(projection.reason, undefined);
      projected.push(
        ...artifacts("test", ["--no-run", ...projection.args]).filter(
          (item) =>
            item.profile.test &&
            names.has(item.package_id) &&
            rustTestCohortForBinary(names.get(item.package_id)!, item.target.name) === cohort,
        ),
      );
    }
    assert.equal(testIdentities(baseline).length, 4);
    assert.deepEqual(testIdentities(projected), testIdentities(baseline));
    const examples = artifacts("build", ["--examples"]);
    const exampleIdentities = (items: Artifact[]) =>
      items
        .filter((item) => item.target.kind.includes("example"))
        .map((item) => ({
          identity: identity(item),
          test: item.profile.test,
          features: item.features,
        }));
    assert.deepEqual(exampleIdentities(examples), exampleIdentities(baseline));
    assert.equal(exampleIdentities(examples).length, 1);
    assert.equal(exampleIdentities(examples)[0].test, false);
    assert.deepEqual(
      examples.filter((item) => item.target.name === "probe_support").map((item) => item.features),
      [["dev"]],
    );
    execFileSync(examples.find((item) => item.target.kind.includes("example"))!.executable!, [], {
      encoding: "utf8",
    });
    const plan = join(directory, "plan.json");
    writeFileSync(
      plan,
      JSON.stringify({
        schemaVersion: 1,
        scope: "workspace",
        packages: selected,
        cargoArgs: roots,
      }),
    );
    writeFileSync(
      join(directory, "canon/examples/normal.rs"),
      'compile_error!("deliberate example failure"); fn main() {}\n',
    );
    const failed = spawnSync(
      process.execPath,
      [
        fileURLToPath(
          new URL("../../tools/support/compat/github/run-affected-rust.ts", import.meta.url),
        ),
        plan,
        "cargo",
        "build",
        "@packages@",
        "--profile",
        "ci",
        "--examples",
        "--message-format=json-render-diagnostics",
      ],
      { cwd: directory, env, encoding: "utf8" },
    );
    assert.equal(failed.status, 101, failed.error?.message ?? failed.stderr);
    assert.match(failed.stderr, /deliberate example failure/);
    assert.deepEqual(JSON.parse(failed.stdout.trim().split("\n").at(-1)!), {
      reason: "build-finished",
      success: false,
    });
  },
);
