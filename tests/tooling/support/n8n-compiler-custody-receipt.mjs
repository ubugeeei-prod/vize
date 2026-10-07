import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import {
  productSource,
  capturePaths,
  driverPaths,
  official,
} from "./n8n-compiler-capture-paths.mjs";
import { validateInstanceMode } from "./n8n-compiler-diagnostic-validation.mjs";
export {
  productSource,
  example,
  cases,
  official,
  driverPaths,
} from "./n8n-compiler-capture-paths.mjs";
export const recipe = [
  "run",
  "--locked",
  "--profile",
  "ci",
  "-p",
  "vize_atelier_sfc",
  "--features",
  "legacy-dom-differential",
  "--example",
  "n8n_compiler_custody",
];
export const sha256 = (bytes) => crypto.createHash("sha256").update(bytes).digest("hex");
export const read = (root, relative) => fs.readFileSync(path.join(root, relative));

export function run(command, args, cwd, options = {}) {
  const result = spawnSync(command, args, { cwd, encoding: "utf8", ...options });
  assert.equal(
    result.status,
    0,
    `${command} ${args.join(" ")}: ${result.stderr ?? result.error ?? "failed"}`,
  );
  return result.stdout?.trim() ?? "";
}

export function verifyBaseline(root, baseline, repairAnchor) {
  assert.match(baseline, /^[a-f0-9]{40}$/u);
  assert.match(repairAnchor, /^[a-f0-9]{40}$/u);
  run("git", ["merge-base", "--is-ancestor", baseline, repairAnchor], root);
  const production = run(
    "git",
    [
      "diff",
      productSource,
      baseline,
      "--",
      "crates/*/src",
      "davinci/*/src",
      "Cargo.toml",
      "Cargo.lock",
    ],
    root,
  );
  assert.equal(
    production,
    "",
    "capture-only ancestor changed production bytes or dependency graph",
  );
  const changed = run("git", ["diff", "--name-only", productSource, baseline], root).split("\n");
  assert.deepEqual(changed, capturePaths, "exact capture-only path inventory");
  const manifestPath = "crates/vize_atelier_sfc/Cargo.toml";
  const baseManifest = run("git", ["show", `${productSource}:${manifestPath}`], root);
  const capturedManifest = run("git", ["show", `${baseline}:${manifestPath}`], root);
  const declaration =
    /\[\[example\]\]\nname = "n8n_compiler_custody"\nrequired-features = \["legacy-dom-differential"\]\n\n/u;
  assert.equal(
    capturedManifest.replace(declaration, ""),
    baseManifest,
    "capture target changed existing Cargo configuration",
  );
  const ancestor = spawnSync("git", ["merge-base", "--is-ancestor", baseline, "HEAD"], {
    cwd: root,
  });
  assert.ok(ancestor.status === 0 || ancestor.status === 1);
  return {
    productSource,
    baselineSource: baseline,
    repairAnchor,
    repairAnchorTree: run("git", ["rev-parse", `${repairAnchor}^{tree}`], root),
    baselineAncestorOfRepairAnchor: true,
    baselineAncestorOfCurrentSource: ancestor.status === 0,
    productionBytesIdentical: true,
    capturePaths: changed,
  };
}

export function verifyInputs(fixtureRoot, manifest) {
  assert.equal(run("git", ["rev-parse", "HEAD"], fixtureRoot), manifest.revision);
  assert.equal(run("git", ["status", "--porcelain"], fixtureRoot), "", "upstream fixture is dirty");
  assert.equal(manifest.cases.length, 10);
  assert.equal(new Set(manifest.cases.map((item) => item.path)).size, 10);
  const originals = manifest.cases.map((item) => {
    assert.ok(!path.isAbsolute(item.path) && !item.path.split("/").includes(".."));
    const bytes = read(fixtureRoot, item.path);
    assert.equal(sha256(bytes), item.sha256, `original input changed: ${item.path}`);
    return { ...item, bytes: bytes.length };
  });
  const licenses = ["LICENSE.md", "LICENSE_EE.md"].map((name) => ({
    name,
    sha256: sha256(read(fixtureRoot, name)),
  }));
  return { fixtureRevision: manifest.revision, originals, licenses };
}

export function validateCapture(root, fixtureRoot, manifest) {
  const observations = [];
  for (const item of manifest.cases) {
    const filename = `${path.basename(item.path)}.json`;
    const packet = JSON.parse(read(root, filename));
    assert.equal(packet.path, item.path);
    assert.equal(sha256(Buffer.from(packet.originalSource)), item.sha256);
    assert.equal(packet.rows.length, 6, `incomplete source packet: ${item.path}`);
    assert.deepEqual(
      packet.rows.filter((row) => row.kind === "template").map((row) => row.prefixIdentifiers),
      [false, true],
    );
    assert.deepEqual(
      packet.rows.filter((row) => row.kind === "sfc").map((row) => [row.inline, row.sourceMap]),
      [
        [false, false],
        [false, true],
        [true, false],
        [true, true],
      ],
    );
    for (const row of packet.rows) {
      if (row.kind === "template") {
        assert.equal(typeof row.legacy.assembled, "string");
        assert.ok(Array.isArray(row.legacy.errors));
        assert.ok(typeof row.native.assembled === "string" || row.native.assembled === null);
        assert.ok(typeof row.native.error === "string" || row.native.error === null);
        assert.ok(Array.isArray(row.native.loweredDiagnostics));
        assert.ok(Array.isArray(row.native.effectiveDiagnostics));
        assert.ok(["returned", "refused"].includes(row.native.production.status));
      } else {
        assert.ok(row.selected && row.legacy, `missing complete SFC result: ${item.path}`);
      }
    }
    observations.push({
      path: item.path,
      packet: filename,
      sha256: sha256(read(root, filename)),
      moduleHashes: moduleHashes(packet),
    });
  }
  assert.deepEqual(JSON.parse(read(root, "cases.json")), manifest);
  for (const name of ["LICENSE.md", "LICENSE_EE.md"])
    assert.deepEqual(read(root, name), read(fixtureRoot, name));
  return observations;
}

function moduleHashes(packet) {
  return packet.rows.flatMap((row, index) => {
    const producers = row.kind === "template" ? ["legacy", "native"] : ["legacy", "selected"];
    const hashes = producers.flatMap((producer) => {
      const code = row.kind === "template" ? row[producer]?.assembled : row[producer]?.Ok?.code;
      return typeof code === "string"
        ? [{ row: index, producer, sha256: sha256(Buffer.from(code)) }]
        : [];
    });
    const diagnosed = row.native?.diagnosedEmission?.assembled;
    if (typeof diagnosed === "string")
      hashes.push({
        row: index,
        producer: "nativeDiagnosed",
        sha256: sha256(Buffer.from(diagnosed)),
      });
    return hashes;
  });
}

export function validateAuthoredCapture(root) {
  const packetName = "n8n-default-slot-loop.json";
  const bytes = read(root, packetName);
  const packet = JSON.parse(bytes);
  assert.equal(
    sha256(Buffer.from(packet.originalSource)),
    "1aa581ed0ab7f543b36085d9102ebb7769c69427061193c89eddb1ffc1084963",
  );
  assert.deepEqual(
    packet.rows.map((row) => [row.kind, row.prefixIdentifiers]),
    [["template", false]],
  );
  for (const row of packet.rows) {
    assert.deepEqual(row.legacy.errors, []);
    assert.equal(typeof row.legacy.assembled, "string");
    assert.equal(typeof row.native.assembled, "string");
  }
  return [{ packet: packetName, sha256: sha256(bytes), moduleHashes: moduleHashes(packet) }];
}

export function validateCurrentParity(root, manifest, diagnosticContract) {
  const comparisons = [];
  for (const item of manifest.cases) {
    const packet = JSON.parse(read(root, `${path.basename(item.path)}.json`));
    for (const row of packet.rows) {
      if (row.kind === "template") {
        if (item.path.endsWith("/InstanceAiConfirmationPanel.vue")) {
          assert.ok(diagnosticContract, "original diagnostic/code contract is not qualified");
          validateInstanceMode(row, diagnosticContract);
          comparisons.push({
            path: item.path,
            kind: row.kind,
            prefixIdentifiers: row.prefixIdentifiers,
            qualification: row.prefixIdentifiers
              ? "accepted module"
              : "diagnostic output; no successful module or runtime credit",
          });
          continue;
        }
        assert.deepEqual(
          row.legacy.errors,
          [],
          `current original template diagnostics: ${item.path}`,
        );
        assert.deepEqual(row.native.loweredDiagnostics, []);
        assert.deepEqual(row.native.effectiveDiagnostics, []);
        assert.equal(row.native.production.status, "returned");
        assert.equal(row.native.production.assembled, row.native.assembled);
        assert.equal(row.native.error, null);
        assert.equal(row.native.diagnosedEmission, null);
        assert.equal(
          row.native.assembled,
          row.legacy.assembled,
          `complete current template module: ${item.path} prefix=${row.prefixIdentifiers}`,
        );
        comparisons.push({
          path: item.path,
          kind: row.kind,
          prefixIdentifiers: row.prefixIdentifiers,
        });
      } else {
        assert.ok(
          Object.hasOwn(row.selected, "Ok"),
          `current original SFC refused: ${item.path} inline=${row.inline} map=${row.sourceMap}`,
        );
        assert.deepEqual(Object.keys(row.selected.Ok).sort(), [
          "bindings",
          "code",
          "css",
          "errors",
          "macroArtifacts",
          "map",
          "warnings",
        ]);
        assert.deepEqual(row.selected.Ok.errors, [], `current original SFC errors: ${item.path}`);
        assert.deepEqual(
          row.selected,
          row.legacy,
          `complete current SFC Result: ${item.path} inline=${row.inline} map=${row.sourceMap}`,
        );
        comparisons.push({
          path: item.path,
          kind: row.kind,
          inline: row.inline,
          sourceMap: row.sourceMap,
          result: "Ok",
        });
      }
    }
  }
  assert.equal(comparisons.length, 60);
  return comparisons;
}

export function sourceReceipt(root, output, targetRoot, inputs) {
  const binary = path.join(targetRoot, "ci/examples/n8n_compiler_custody");
  run("git", ["diff", "HEAD", "--exit-code"], root);
  return {
    schema: "vize.n8n.compiler-source",
    version: 1,
    sourceRevision: run("git", ["rev-parse", "HEAD"], root),
    sourceTree: run("git", ["rev-parse", "HEAD^{tree}"], root),
    recipe: ["cargo", ...recipe, "--", "<same read-only fixture>", "<phase output>"],
    officialRecipe: [
      process.execPath,
      official,
      "<same read-only fixture>",
      "<phase output>",
      "<checksum-pinned official bundle>",
    ],
    node: process.version,
    cargo: run("cargo", ["--version"], root),
    rustc: run("rustc", ["--version", "--verbose"], root),
    cargoLockSha256: sha256(read(root, "Cargo.lock")),
    drivers: driverPaths.map((name) => ({
      path: name,
      sha256: sha256(read(root, name)),
    })),
    binary: { path: binary, sha256: sha256(fs.readFileSync(binary)) },
    inputs,
    output,
    environment: { RUSTFLAGS: process.env.RUSTFLAGS ?? null, CARGO_TARGET_DIR: targetRoot },
  };
}
