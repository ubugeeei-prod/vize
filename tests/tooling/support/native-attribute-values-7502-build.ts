// Hosted-only source build and two fresh test processes. Importing this module does not execute them.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  chmodSync,
  copyFileSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { exactKeys7502, hash7502 } from "./native-attribute-values-7502-inputs.ts";
import {
  root7502,
  source7502,
  sourcePath7502,
  testName7502,
  workspacePackage7502,
} from "./native-attribute-values-7502-source.ts";

export { root7502 };
const name = testName7502;
export const cargoArgs7502 = [
  "test",
  "--no-run",
  "--locked",
  "--profile",
  "ci",
  "-p",
  "vize_atelier_sfc",
  "--test",
  name,
  "--message-format=json-render-diagnostics",
];
export function select7502Artifact(stdout: Buffer) {
  const messages = stdout
    .toString("utf8")
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line));
  assert.deepEqual(
    messages.filter((row) => row.reason === "build-finished").map((row) => row.success),
    [true],
  );
  const selected = messages.filter(
    (row) =>
      row.reason === "compiler-artifact" &&
      row.target.name === name &&
      row.target.kind.includes("test") &&
      row.executable,
  );
  assert.equal(selected.length, 1);
  const [artifact] = selected;
  assert.deepEqual(artifact.target.kind, ["test"]);
  assert.equal(artifact.profile.test, true);
  assert.deepEqual(artifact.features.toSorted(), ["default", "native"]);
  assert.equal(
    realpathSync(artifact.target.src_path),
    realpathSync(path.join(root7502, sourcePath7502)),
  );
  workspacePackage7502(artifact.package_id);
  return artifact;
}
export function rawProcess7502(result: any) {
  return {
    exitStatus: result.status,
    signal: result.signal,
    processError: result.error?.message ?? null,
    stdoutBase64: (result.stdout ?? Buffer.alloc(0)).toString("base64"),
    stderrBase64: (result.stderr ?? Buffer.alloc(0)).toString("base64"),
    stdoutSha256: hash7502(result.stdout ?? Buffer.alloc(0)),
    stderrSha256: hash7502(result.stderr ?? Buffer.alloc(0)),
  };
}
export function hosted7502(directory: string) {
  const evidence = path.resolve(directory);
  mkdirSync(evidence, { recursive: true });
  const source = source7502();
  const build = spawnSync("cargo", cargoArgs7502, {
    cwd: root7502,
    timeout: 240_000,
    maxBuffer: 64 * 1024 * 1024,
  });
  writeFileSync(path.join(evidence, "cargo.jsonl"), build.stdout ?? Buffer.alloc(0));
  writeFileSync(path.join(evidence, "cargo.stderr.txt"), build.stderr ?? Buffer.alloc(0));
  writeFileSync(
    path.join(evidence, "build-process.json"),
    JSON.stringify(rawProcess7502(build), null, 2),
  );
  assert.equal(build.error, undefined);
  assert.equal(build.signal, null);
  assert.equal(build.status, 0, "source build failed; raw logs retained");
  const artifact = select7502Artifact(build.stdout);
  const binary = path.join(evidence, name);
  copyFileSync(artifact.executable, binary);
  chmodSync(binary, 0o755);
  const sha256 = hash7502(readFileSync(binary));
  const attempts = [];
  for (const suffix of ["first", "repeat"]) {
    const capture = path.join(evidence, `${suffix}.capture.json`);
    rmSync(capture, { force: true });
    rmSync(`${capture}.envelopes.json`, { force: true });
    const result = spawnSync(binary, ["--nocapture"], {
      cwd: root7502,
      timeout: 120_000,
      maxBuffer: 16 * 1024 * 1024,
      env: { ...process.env, VIZE_NATIVE_ATTRIBUTE_VALUES_7502_CAPTURE: capture },
    });
    const raw = rawProcess7502(result);
    writeFileSync(path.join(evidence, `${suffix}.process.json`), JSON.stringify(raw, null, 2));
    const sha = (file: string) => {
      try {
        return hash7502(readFileSync(file));
      } catch {
        return null;
      }
    };
    attempts.push({
      ...raw,
      capturePath: capture,
      captureSha256: sha(capture),
      envelopesPath: `${capture}.envelopes.json`,
      envelopesSha256: sha(`${capture}.envelopes.json`),
    });
  }
  const receipt = {
    schema: "vize.native-attribute-values-7502.build",
    version: 1,
    source,
    command: ["cargo", ...cargoArgs7502],
    build: rawProcess7502(build),
    artifact,
    binaryPath: binary,
    binarySha256: sha256,
    toolchain: Object.fromEntries(
      ["rustc", "cargo"].map((tool) => {
        const version = spawnSync(tool, ["--version"], { cwd: root7502 });
        assert.equal(version.status, 0);
        assert.equal(version.signal, null);
        assert.equal(version.error, undefined);
        return [tool, version.stdout.toString("utf8").trim()];
      }),
    ),
    attempts,
  };
  writeFileSync(path.join(evidence, "build-receipt.json"), JSON.stringify(receipt, null, 2));
  assert.deepEqual(source7502(), source, "source drift during build/capture");
  assert.equal(hash7502(readFileSync(binary)), sha256, "captured binary drift");
  assert.equal(attempts.length, 2);
  for (const attempt of attempts) {
    assert.equal(attempt.processError, null);
    assert.equal(attempt.signal, null);
    assert(attempt.captureSha256, "complete capture is mandatory even when unfrozen checks fail");
    assert(attempt.envelopesSha256, "complete independent envelope capture is mandatory");
  }
  assert(
    readFileSync(attempts[0].capturePath).equals(readFileSync(attempts[1].capturePath)),
    "complete fresh-process capture repeat",
  );
  assert(
    readFileSync(attempts[0].envelopesPath).equals(readFileSync(attempts[1].envelopesPath)),
    "complete fresh-process envelope repeat",
  );
  // Both real statuses remain in the receipt. Unfrozen fixtures may fail AFTER
  // capture; the action still runs the independent runtime judge, then fails.
  return attempts.every((attempt) => attempt.exitStatus === 0) ? 0 : 1;
}
export function validate7502Build(receipt: any, capturePath: string) {
  exactKeys7502(receipt, [
    "schema",
    "version",
    "source",
    "command",
    "build",
    "artifact",
    "binaryPath",
    "binarySha256",
    "toolchain",
    "attempts",
  ]);
  assert.equal(receipt.schema, "vize.native-attribute-values-7502.build");
  assert.equal(receipt.version, 1);
  assert.deepEqual(receipt.source, source7502());
  assert.deepEqual(receipt.command, ["cargo", ...cargoArgs7502]);
  assert.equal(receipt.build.exitStatus, 0);
  assert.equal(receipt.build.signal, null);
  assert.equal(receipt.build.processError, null);
  const stdout = Buffer.from(receipt.build.stdoutBase64, "base64");
  const stderr = Buffer.from(receipt.build.stderrBase64, "base64");
  validateProcess7502(receipt.build);
  const evidence = path.dirname(receipt.binaryPath);
  assert(stdout.equals(readFileSync(path.join(evidence, "cargo.jsonl"))));
  assert(stderr.equals(readFileSync(path.join(evidence, "cargo.stderr.txt"))));
  assert.deepEqual(
    receipt.build,
    JSON.parse(readFileSync(path.join(evidence, "build-process.json"), "utf8")),
  );
  assert.equal(receipt.build.stdoutSha256, hash7502(stdout));
  assert.equal(receipt.build.stderrSha256, hash7502(stderr));
  assert.deepEqual(select7502Artifact(stdout), receipt.artifact);
  assert.equal(receipt.binarySha256, hash7502(readFileSync(receipt.binaryPath)));
  assert.equal(receipt.binaryPath, path.join(path.dirname(capturePath), name));
  exactKeys7502(receipt.toolchain, ["rustc", "cargo"]);
  for (const tool of ["rustc", "cargo"]) {
    const result = spawnSync(tool, ["--version"], { cwd: root7502, encoding: "utf8" });
    assert.equal(result.status, 0);
    assert.equal(result.signal, null);
    assert.equal(result.error, undefined);
    assert.equal(receipt.toolchain[tool], result.stdout.trim());
  }
  assert.equal(receipt.attempts.length, 2);
  const first = readFileSync(receipt.attempts[0].capturePath);
  const repeated = readFileSync(receipt.attempts[1].capturePath);
  assert(first.equals(repeated));
  assert(
    first.equals(readFileSync(capturePath)),
    "runtime consumes the exact source-built capture",
  );
  const envelopes = readFileSync(receipt.attempts[0].envelopesPath);
  assert(envelopes.equals(readFileSync(receipt.attempts[1].envelopesPath)));
  for (const [index, attempt] of receipt.attempts.entries()) {
    exactKeys7502(attempt, [
      ...processKeys7502,
      "capturePath",
      "captureSha256",
      "envelopesPath",
      "envelopesSha256",
    ]);
    const suffix = index === 0 ? "first" : "repeat";
    assert.equal(attempt.capturePath, path.join(evidence, `${suffix}.capture.json`));
    assert.equal(attempt.envelopesPath, `${attempt.capturePath}.envelopes.json`);
    assert.equal(attempt.envelopesSha256, hash7502(envelopes));
    const raw = Object.fromEntries(processKeys7502.map((key) => [key, attempt[key]]));
    assert.deepEqual(
      raw,
      JSON.parse(readFileSync(path.join(evidence, `${suffix}.process.json`), "utf8")),
    );
    assert.equal(attempt.captureSha256, hash7502(first));
    assert.equal(attempt.processError, null);
    assert.equal(attempt.signal, null);
    assert(
      [0, 101].includes(attempt.exitStatus),
      "only actual test success or unfrozen assertion failure",
    );
    validateProcess7502(raw);
  }
}
const processKeys7502 = [
  "exitStatus",
  "signal",
  "processError",
  "stdoutBase64",
  "stderrBase64",
  "stdoutSha256",
  "stderrSha256",
];
export function validateProcess7502(raw: any) {
  exactKeys7502(raw, processKeys7502);
  assert(raw.exitStatus === null || Number.isInteger(raw.exitStatus));
  for (const key of ["signal", "processError"])
    assert(raw[key] === null || typeof raw[key] === "string");
  for (const stream of ["stdout", "stderr"]) {
    assert.equal(typeof raw[`${stream}Base64`], "string");
    const bytes = Buffer.from(raw[`${stream}Base64`], "base64");
    assert.equal(bytes.toString("base64"), raw[`${stream}Base64`]);
    assert.equal(hash7502(bytes), raw[`${stream}Sha256`]);
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.argv.length, 3);
  process.exitCode = hosted7502(process.argv[2]);
}
