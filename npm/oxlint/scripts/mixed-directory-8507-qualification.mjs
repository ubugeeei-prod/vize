// Additive exact-1.81 qualification, reusing the existing staged source receipt.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import { performance } from "node:perf_hooks";
import { captureProjectCommand } from "./project-command-capture.mjs";
import { PUBLIC_OVERRIDE_KEYS } from "../src/test-support/original-sfc-host-matrix.ts";
import { plainHostEnvironment, snapshot } from "../src/test-support/html-cli-oracles.mjs";
import { installMixedHost, qualifyHistoricalMixedDirectory } from "./mixed-directory-history.mjs";
import {
  joinMixedCaptures,
  runMixedWorkers,
} from "../src/test-support/mixed-directory-8507-workers.mjs";

const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const errorPacket = (error) =>
  error == null
    ? null
    : {
        name: error.name,
        ...Object.fromEntries(Object.getOwnPropertyNames(error).map((key) => [key, error[key]])),
      };
export async function qualifyMixedDirectory8507({ root, packageDir, artifacts, receipt, binary }) {
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
    const provider = await installMixedHost(temporary, path.join(output, "source"));
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
    const records = await runMixedWorkers({
      packageDir,
      output,
      custody,
      preload,
      environment: {
        ...process.env,
        VIZE_OXLINT_TEST_ENTRYPOINT: provider.engine,
      },
    });
    const plan = JSON.parse(
      fs.readFileSync(
        path.join(
          root,
          "tests/_fixtures/differential/lint/oxlint-mixed-directory-8507/controls.json",
        ),
      ),
    );
    const captures = records.map((record) => JSON.parse(fs.readFileSync(record.capture)));
    const journals = records.map((record) => fs.readFileSync(record.calls));
    fs.writeFileSync(custody.calls, Buffer.concat(journals));
    for (const [index, journal] of journals.entries()) {
      assert.deepEqual(
        journal
          .toString("utf8")
          .trim()
          .split("\n")
          .map((line) => JSON.parse(line)),
        [
          ...captures[index].initialEvents,
          ...captures[index].observations.flatMap((record) => record.events),
        ],
      );
    }
    const capture = joinMixedCaptures(plan, custody, captures);
    fs.writeFileSync(
      path.join(output, "source-after.json"),
      JSON.stringify(capture, null, 2) + "\n",
    );
    assert.equal(capture.complete, true);
    assert.deepEqual(capture.source, receipt.source);
    assert.deepEqual(capture.passed, { cli: 45, native: 8 });
    invocation.phase = "historical-before";
    save();
    qualified = {
      sourceAfter: capture.passed,
      provider,
      historicalBefore: await qualifyHistoricalMixedDirectory({
        root,
        output: path.join(output, "before"),
      }),
    };
    invocation.phase = "original-sfc-public-baseline";
    save();
    // The candidate is already exercised by the package test suite. Install
    // this historical public comparison once per qualification, never per test.
    qualified.originalSfcPublicBaseline = await qualifyOriginalSfcPublicBaseline({
      packageDir,
      output: path.join(output, "original-sfc-7004-7005"),
    });
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

async function qualifyOriginalSfcPublicBaseline({ packageDir, output }) {
  const started = performance.now();
  let completed = false;
  fs.mkdirSync(output);
  const installation = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-original-sfc-before-")),
  );
  const inventory = () =>
    snapshot(installation).map(([file, kind, bytes]) =>
      kind === "file" ? [file, kind, hash(Buffer.from(bytes, "base64"))] : [file, kind, bytes],
    );
  try {
    // Explicit host selection leaves every existing installMixedHost call at1.81.
    const provider = await installMixedHost(
      installation,
      path.join(output, "install"),
      "0.458.0",
      "1.86.0",
    );
    const before = inventory();
    fs.writeFileSync(
      path.join(output, "installed-files.json"),
      JSON.stringify(before, null, 2) + "\n",
    );
    for (const name of ["oxlint", "oxlint-plugin-vize"])
      fs.copyFileSync(
        path.join(installation, "node_modules", name, "package.json"),
        path.join(output, `${name}-package.json`),
        fs.constants.COPYFILE_EXCL,
      );
    const lock = JSON.parse(fs.readFileSync(path.join(installation, "package-lock.json")));
    const nativePackages = Object.entries(lock.packages).filter(([name]) =>
      name.startsWith("node_modules/@vizejs/native"),
    );
    assert.ok(nativePackages.length > 0, "public native package identity must be present");
    for (const [, entry] of nativePackages) {
      assert.equal(entry.version, "0.458.0");
      assert.match(entry.integrity, /^sha512-/u);
      assert.ok(entry.resolved.startsWith("https://registry.npmjs.org/"));
    }
    const args = [
      path.join(packageDir, "src/test-support/original-sfc-host-matrix.ts"),
      "--plugin-package",
      path.join(installation, "node_modules/oxlint-plugin-vize"),
      "--oxlint-package",
      path.join(installation, "node_modules/oxlint"),
      "--output",
      path.join(output, "matrix"),
      "--expectation",
      "public-0.458",
    ];
    const env = plainHostEnvironment(process.env);
    for (const key of PUBLIC_OVERRIDE_KEYS) delete env[key];
    const result = await captureProjectCommand(process.execPath, args, {
      cwd: installation,
      env,
      timeout: 180_000,
      maxBuffer: 64 * 1024 * 1024,
    });
    const command = {
      command: process.execPath,
      args,
      cwd: installation,
      status: result.status,
      signal: result.signal,
      error: errorPacket(result.error),
      stdoutBytes: Array.from(result.stdout ?? []),
      stderrBytes: Array.from(result.stderr ?? []),
    };
    fs.writeFileSync(path.join(output, "process.json"), JSON.stringify(command, null, 2) + "\n");
    assert.equal(command.error, null);
    assert.equal(command.signal, null);
    assert.equal(command.status, 0);
    const matrix = JSON.parse(fs.readFileSync(path.join(output, "matrix/receipt.json")));
    assert.equal(matrix.contractAssertionsPassed, true);
    assert.equal(matrix.pluginVersion, "0.458.0");
    assert.equal(matrix.hostVersion, "1.86.0");
    const packet = JSON.parse(
      fs.readFileSync(path.join(packageDir, "src/test-support/original-sfc-host-7004-7005.json")),
    );
    const expectedCells = packet.cases.flatMap((original) =>
      ["direct", "wrapper"].flatMap((host) => [
        ...["original-default", "json"].map(
          (reporter) => `${original.filename}-${host}-${reporter}`,
        ),
        ...[...original.contrasts.map((entry) => entry.name), "rule-disabled"].map(
          (name) => `${original.filename}-${host}-${name}`,
        ),
      ]),
    );
    assert.deepEqual(
      matrix.cells.map((cell) => cell.name).sort(),
      expectedCells.sort(),
      "every original and independent contrast must have an actual process receipt",
    );
    for (const cell of matrix.cells)
      for (const stream of [cell.stdout, cell.stderr]) {
        const raw = fs.readFileSync(stream.path);
        assert.equal(raw.length, stream.bytes);
        assert.equal(hash(raw), stream.sha256);
      }
    const image = path.relative(installation, fs.realpathSync(matrix.native.path));
    assert.ok(
      image !== "" && !path.isAbsolute(image) && image.split(path.sep)[0] !== "..",
      "reference image must belong to the actual public installation",
    );
    assert.equal(hash(fs.readFileSync(matrix.native.path)), matrix.native.sha256);
    assert.deepEqual(inventory(), before, "the whole installed provider must remain unchanged");
    completed = true;
    return {
      provider,
      matrix: path.join(output, "matrix/receipt.json"),
      scope:
        "historical public comparison only; direct-host capability remains separately observed",
    };
  } finally {
    // All raw receipts/locks/manifests/inputs/whole-file inventory are retained
    // under the existing always-upload artifact, including matrix failure.
    fs.rmSync(installation, { recursive: true, force: true });
    fs.writeFileSync(
      path.join(output, "baseline-invocation.json"),
      JSON.stringify(
        {
          completed,
          elapsedMs: performance.now() - started,
          scope:
            "one actual public install, full matrix, whole provider checks and owned cleanup; historical comparison only",
        },
        null,
        2,
      ) + "\n",
    );
  }
}
