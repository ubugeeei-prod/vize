// Additive exact-1.81 qualification, reusing the existing staged source receipt.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import { fileURLToPath, pathToFileURL } from "node:url";
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
    // Keep the existing workspace-host candidate and install this historical
    // public comparison once. Its same host is then reused for the source matrix.
    qualified.originalSfcPublicBaseline = await qualifyOriginalSfcPublicBaseline({
      packageDir,
      receipt,
      binary,
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

async function qualifyOriginalSfcPublicBaseline({ packageDir, output, receipt, binary }) {
  const started = performance.now();
  let completed = false,
    publicElapsedMs = null,
    sourceStarted = null;
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
    publicElapsedMs = performance.now() - started;
    sourceStarted = performance.now();
    // A fresh candidate child uses the identical installed host after the untouched
    // public baseline. Reuse the already authenticated staged addon, without
    // another install, native build, stage copy or public-provider mutation.
    const sourceOutput = path.join(output, "candidate-1.86");
    fs.mkdirSync(sourceOutput);
    const sourceManifest = JSON.parse(fs.readFileSync(path.join(packageDir, "package.json")));
    const sourceFiles = [
      "package.json",
      sourceManifest.main,
      sourceManifest.bin["oxlint-vize"],
      "dist/cli.mjs",
      "scripts/project-native-staging.ts",
      "../native/index.js",
    ].map((file) => {
      const physical = fs.realpathSync(path.resolve(packageDir, file));
      return { path: physical, sha256: hash(fs.readFileSync(physical)) };
    });
    const sourceCommand = originalSfcSourceCommand({
      packageDir,
      installation,
      output: sourceOutput,
      receipt,
      binary,
    });
    const sourceResult = await captureProjectCommand(
      sourceCommand.command,
      sourceCommand.args,
      sourceCommand.options,
    );
    fs.writeFileSync(
      path.join(sourceOutput, "process.json"),
      JSON.stringify(
        {
          command: sourceCommand.command,
          args: sourceCommand.args,
          cwd: sourceCommand.options.cwd,
          environment: Object.fromEntries(
            PUBLIC_OVERRIDE_KEYS.map((key) => [key, sourceCommand.options.env[key] ?? null]),
          ),
          source: receipt.source,
          sourceFiles,
          binary: { path: fs.realpathSync(binary), sha256: receipt.frozen.sha256 },
          status: sourceResult.status,
          signal: sourceResult.signal,
          error: errorPacket(sourceResult.error),
          stdoutBytes: Array.from(sourceResult.stdout ?? []),
          stderrBytes: Array.from(sourceResult.stderr ?? []),
        },
        null,
        2,
      ) + "\n",
    );
    assert.equal(sourceResult.error, undefined);
    assert.equal(sourceResult.signal, null);
    assert.equal(sourceResult.status, 0);
    const sourceMatrix = JSON.parse(fs.readFileSync(path.join(sourceOutput, "receipt.json")));
    assertOriginalSfcSourceMatrix({
      matrix: sourceMatrix,
      baseline: matrix,
      receipt,
      binary,
      plugin: sourceFiles[1],
      version: sourceManifest.version,
    });
    for (const file of sourceFiles) assert.equal(hash(fs.readFileSync(file.path)), file.sha256);
    assert.deepEqual(
      inventory(),
      before,
      "the same installed host/public provider must remain unchanged",
    );
    completed = true;
    return {
      provider,
      matrix: path.join(output, "matrix/receipt.json"),
      sameHostSourceMatrix: path.join(sourceOutput, "receipt.json"),
      source: receipt.source,
      sourceBinarySha256: receipt.frozen.sha256,
      scope:
        "same-host public/source comparison; direct-host capability remains separately observed",
    };
  } finally {
    // All raw receipts/locks/manifests/inputs/whole-file inventory are retained
    // under the existing always-upload artifact, including matrix failure.
    try {
      fs.writeFileSync(
        path.join(output, "installed-files-after.json"),
        JSON.stringify(inventory(), null, 2) + "\n",
      );
    } finally {
      fs.rmSync(installation, { recursive: true, force: true });
    }
    fs.writeFileSync(
      path.join(output, "baseline-invocation.json"),
      JSON.stringify(
        {
          completed,
          elapsedMs: performance.now() - started,
          publicElapsedMs,
          sourceElapsedMs: sourceStarted === null ? null : performance.now() - sourceStarted,
          scope:
            "one public install, public/source full matrices on the same1.86 host, whole provider checks and owned cleanup; no direct-host completion claim",
        },
        null,
        2,
      ) + "\n",
    );
  }
}

/** Selected source authority is explicit; ambient overrides cannot authorize it. */
export function originalSfcSourceCommand({
  packageDir,
  installation,
  output,
  receipt,
  binary,
  environment = process.env,
}) {
  assert.equal(receipt.source.head, environment.GITHUB_SHA);
  assert.equal(hash(fs.readFileSync(binary)), receipt.frozen.sha256);
  const env = plainHostEnvironment(environment);
  for (const key of PUBLIC_OVERRIDE_KEYS) delete env[key];
  const wrapper = fs.realpathSync(path.resolve(packageDir, "../native/index.js"));
  const configuration = path.join(output, "source-preload.json");
  fs.writeFileSync(
    configuration,
    JSON.stringify(
      {
        schema: "vize.oxlint.original-sfc-preload",
        source: receipt.source,
        wrapper: { path: wrapper, sha256: hash(fs.readFileSync(wrapper)) },
        binary: { path: fs.realpathSync(binary), sha256: receipt.frozen.sha256 },
      },
      null,
      2,
    ) + "\n",
  );
  env.VIZE_OXLINT_NATIVE_CUSTODY = configuration;
  env.NODE_OPTIONS = `--import=${pathToFileURL(path.join(packageDir, "scripts/project-native-staging.ts")).href}`;
  env.VIZE_PREFER_WORKSPACE_BINDING = "1";
  return {
    command: process.execPath,
    args: [
      path.join(packageDir, "src/test-support/original-sfc-host-matrix.ts"),
      "--plugin-package",
      packageDir,
      "--oxlint-package",
      path.join(installation, "node_modules/oxlint"),
      "--output",
      output,
      "--expectation",
      "candidate",
    ],
    options: { cwd: installation, env, timeout: 180_000, maxBuffer: 64 * 1024 * 1024 },
  };
}

/** Source and public receipts must describe the same host and every original cell. */
export function assertOriginalSfcSourceMatrix({
  matrix,
  baseline,
  receipt,
  binary,
  plugin,
  version,
}) {
  assert.equal(matrix.expectation, "candidate");
  assert.equal(matrix.contractAssertionsPassed, true);
  assert.deepEqual(matrix.failures, []);
  assert.equal(baseline.hostVersion, "1.86.0");
  assert.equal(matrix.hostVersion, baseline.hostVersion);
  assert.deepEqual(matrix.manifests[1], baseline.manifests[1]);
  assert.equal(hash(fs.readFileSync(matrix.manifests[1].path)), matrix.manifests[1].sha256);
  assert.equal(matrix.pluginVersion, version);
  assert.equal(fs.realpathSync(matrix.plugin.path), plugin.path);
  assert.equal(matrix.plugin.sha256, plugin.sha256);
  assert.equal(fs.realpathSync(matrix.native.path), fs.realpathSync(binary));
  assert.equal(matrix.native.sha256, receipt.frozen.sha256);
  assert.equal(hash(fs.readFileSync(binary)), receipt.frozen.sha256);
  assert.deepEqual(
    matrix.cells.map((cell) => cell.name).sort(),
    baseline.cells.map((cell) => cell.name).sort(),
  );
  for (const cell of matrix.cells)
    for (const stream of [cell.stdout, cell.stderr]) {
      const raw = fs.readFileSync(stream.path);
      assert.equal(raw.length, stream.bytes);
      assert.equal(hash(raw), stream.sha256);
    }
  for (const project of matrix.projects)
    for (const phase of ["before", "after"])
      for (const input of project[phase]) {
        const raw = fs.readFileSync(path.join(project.evidence, phase, input.archiveFilename));
        assert.equal(raw.length, input.bytes);
        assert.equal(hash(raw), input.sha256);
      }
  assert.deepEqual(matrix.packet, baseline.packet);
  assert.deepEqual(
    matrix.projects.map(({ files, before, after }) => ({ files, before, after })),
    baseline.projects.map(({ files, before, after }) => ({ files, before, after })),
  );
}
