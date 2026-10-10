// Additive exact-1.81 qualification, reusing the existing staged source receipt.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { installMixedHost, qualifyHistoricalMixedDirectory } from "./mixed-directory-history.mjs";

const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const errorPacket = (error) =>
  error == null
    ? null
    : {
        name: error.name,
        ...Object.fromEntries(Object.getOwnPropertyNames(error).map((key) => [key, error[key]])),
      };
export function qualifyMixedDirectory8507({ root, packageDir, artifacts, receipt, binary }) {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(receipt.source.head, process.env.GITHUB_SHA);
  assert.equal(hash(fs.readFileSync(binary)), receipt.frozen.sha256);
  const output = path.join(artifacts, "mixed-directory-8507");
  fs.mkdirSync(output);
  const invocation = {
    complete: false,
    source: receipt.source,
    toolchain: receipt.toolchain,
    binary: { path: fs.realpathSync(binary), sha256: receipt.frozen.sha256 },
    phase: "install",
  };
  const save = () =>
    fs.writeFileSync(
      path.join(output, "invocation.json"),
      JSON.stringify(invocation, null, 2) + "\n",
    );
  save();
  const temporary = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-8507-source-host-")),
  );
  let qualified;
  try {
    const provider = installMixedHost(temporary, path.join(output, "source"));
    const custody = {
      schema: "vize.oxlint.source-native",
      version: 1,
      source: receipt.source,
      toolchain: receipt.toolchain,
      binary: { path: fs.realpathSync(binary), sha256: receipt.frozen.sha256 },
      calls: path.join(output, "source-native-calls.jsonl"),
    };
    const configuration = path.join(output, "source-custody.json");
    fs.writeFileSync(configuration, JSON.stringify(custody, null, 2) + "\n");
    const preload = fileURLToPath(new URL("./project-html-custody.cjs", import.meta.url));
    invocation.phase = "source-after";
    save();
    const argv = ["src/mixed-directory-8507.test.mjs"];
    const result = spawnSync(process.execPath, argv, {
      cwd: packageDir,
      timeout: 180_000,
      maxBuffer: 64 * 1024 * 1024,
      env: {
        ...process.env,
        NODE_OPTIONS:
          `${process.env.NODE_OPTIONS ?? ""} --require=${JSON.stringify(preload)}`.trim(),
        VIZE_OXLINT_NATIVE_CUSTODY: configuration,
        VIZE_OXLINT_TEST_ENTRYPOINT: provider.engine,
        VIZE_OXLINT_SOURCE_BIN: path.join(packageDir, "dist/cli.mjs"),
        VIZE_OXLINT_MIXED_CAPTURE: path.join(output, "source-after.json"),
      },
    });
    const record = {
      command: [process.execPath, ...argv],
      cwd: packageDir,
      status: result.status,
      signal: result.signal,
      error: errorPacket(result.error),
      stdoutBytes: Array.from(result.stdout ?? []),
      stderrBytes: Array.from(result.stderr ?? []),
    };
    fs.writeFileSync(
      path.join(output, "source-process.json"),
      JSON.stringify(record, null, 2) + "\n",
    );
    process.stdout.write(result.stdout ?? "");
    process.stderr.write(result.stderr ?? "");
    assert.equal(record.signal, null);
    assert.equal(record.error, null);
    assert.equal(record.status, 0);
    const capture = JSON.parse(fs.readFileSync(path.join(output, "source-after.json")));
    assert.equal(capture.complete, true);
    assert.deepEqual(capture.source, receipt.source);
    assert.deepEqual(capture.passed, { cli: 45, native: 8 });
    invocation.phase = "historical-before";
    save();
    qualified = {
      sourceAfter: capture.passed,
      provider,
      historicalBefore: qualifyHistoricalMixedDirectory({
        root,
        output: path.join(output, "before"),
      }),
    };
  } catch (error) {
    invocation.failure = errorPacket(error);
    save();
    throw error;
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
  assert.equal(hash(fs.readFileSync(binary)), receipt.frozen.sha256);
  fs.writeFileSync(
    path.join(output, "qualification.json"),
    JSON.stringify(
      {
        source: receipt.source,
        toolchain: receipt.toolchain,
        binarySha256: receipt.frozen.sha256,
        ...qualified,
        limits: [
          "source-built Linux Actions, not the reporter's Darwin installation",
          "public installed acceptance requires a genuinely included published release",
        ],
      },
      null,
      2,
    ) + "\n",
  );
  invocation.complete = true;
  invocation.phase = "complete";
  save();
  return qualified;
}
