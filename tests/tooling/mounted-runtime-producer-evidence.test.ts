import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { parse, stringify } from "yaml";
import { collectRuntimeProducer } from "./support/mounted-runtime-source.ts";

const keys = [
  "VIZE_MOUNTED_RUNTIME_PRODUCER_EVIDENCE",
  "VIZE_MOUNTED_RUNTIME_PRODUCER_EVIDENCE_FAILURES_ONLY",
] as const;
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const canonical = Buffer.from(
  JSON.stringify({ code: `export const value = ${JSON.stringify("雪🌸".repeat(20_000))};` }) + "\n",
);

function execute(stdout: Buffer, stderr: Buffer, exit: number, signal: string | null = null) {
  const child = spawn(
    process.execPath,
    [
      "-e",
      `
    const fs = require('node:fs');
    const chunks = [];
    process.stdin.on('data', chunk => chunks.push(chunk));
    process.stdin.on('end', () => {
      const input = JSON.parse(Buffer.concat(chunks).toString());
      for (const [fd, encoded] of [[1, input.stdout], [2, input.stderr]]) {
        const bytes = Buffer.from(encoded, 'hex');
        let at = 0;
        while (at < bytes.length) at += fs.writeSync(fd, bytes, at);
      }
      if (input.signal) process.kill(process.pid, input.signal);
      else process.exitCode = input.exit;
    });
  `,
    ],
    { stdio: ["pipe", "pipe", "pipe"] },
  );
  const result = collectRuntimeProducer(child, false);
  child.stdin.end(
    JSON.stringify({
      stdout: stdout.toString("hex"),
      stderr: stderr.toString("hex"),
      exit,
      signal,
    }),
  );
  return { child, result };
}

await test("failed producers retain complete large streams and status before rejection", async (t) => {
  const temporary = mkdtempSync(join(tmpdir(), "vize-mounted-producer-evidence-"));
  const previous = keys.map((key) => [key, process.env[key]] as const);
  t.after(() => {
    for (const [key, value] of previous) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
    rmSync(temporary, { recursive: true, force: true });
  });
  assert(canonical.length > 65_536);
  const cases = [
    { exit: 7, signal: null, stderr: Buffer.alloc(0), phase: "terminal-status", stdout: canonical },
    {
      exit: 0,
      signal: null,
      stderr: Buffer.from([255, 13, 27]),
      phase: "terminal-status",
      stdout: canonical,
    },
    {
      exit: 0,
      signal: null,
      stderr: Buffer.alloc(0),
      phase: "protocol-rejection",
      stdout: Buffer.from('{"code":"x","unknown":true}\n'),
    },
    ...(process.platform === "win32"
      ? []
      : [
          {
            exit: 0,
            signal: "SIGTERM",
            stderr: Buffer.alloc(0),
            phase: "terminal-status",
            stdout: canonical,
          },
        ]),
  ];
  for (const [index, row] of cases.entries()) {
    const directory = join(temporary, String(index));
    process.env[keys[0]] = directory;
    process.env[keys[1]] = "1";
    const { child, result } = execute(row.stdout, row.stderr, row.exit, row.signal);
    await assert.rejects(result, Error);
    const names = readdirSync(directory);
    const manifests = names.filter((name) => name.endsWith(".failure.json"));
    assert.equal(manifests.length, 1);
    assert.equal(names.length, 3);
    const manifest = JSON.parse(readFileSync(join(directory, manifests[0]!), "utf8"));
    assert.equal(manifest.schema, "vize.mounted-runtime-producer-failure.v1");
    assert.equal(manifest.accepted, false);
    assert.equal(manifest.phase, row.phase);
    assert.equal(manifest.protocolValidated, false);
    assert.equal(manifest.pid, child.pid);
    assert.equal(manifest.production, false);
    assert.equal(manifest.selectedExecutable, child.spawnfile);
    assert.equal(manifest.parentExecutable, process.execPath);
    assert.equal(manifest.exitCode, row.signal ? null : row.exit);
    assert.equal(manifest.signal, row.signal);
    assert.equal(manifest.spawnError, null);
    assert.equal(manifest.inputError, null);
    assert.equal(manifest.completeStreamsPersisted, true);
    for (const [kind, expected] of [
      ["stdout", row.stdout],
      ["stderr", row.stderr],
    ] as const) {
      const stream = manifest.streams[kind];
      const actual = readFileSync(join(directory, stream.file));
      assert.deepEqual(actual, expected);
      assert.equal(stream.bytes, expected.length);
      assert.equal(stream.sha256, hash(expected));
      assert.equal(stream.persisted, true);
      assert.equal(stream.error, null);
    }
  }
});

await test("legacy evidence I/O failure is distinct from protocol rejection", async (t) => {
  const temporary = mkdtempSync(join(tmpdir(), "vize-mounted-producer-legacy-io-"));
  const previous = keys.map((key) => [key, process.env[key]] as const);
  t.after(() => {
    for (const [key, value] of previous) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
    rmSync(temporary, { recursive: true, force: true });
  });
  for (const [index, suffix] of ["stdout.bin", "json"].entries()) {
    const directory = join(temporary, String(index));
    process.env[keys[0]] = directory;
    delete process.env[keys[1]];
    const { child, result } = execute(canonical, Buffer.alloc(0), 0);
    mkdirSync(join(directory, `development-${child.pid}.${suffix}`), { recursive: true });
    await assert.rejects(result, (error) => {
      assert(error instanceof Error);
      assert(error.cause instanceof Error);
      assert.match(error.cause.message, /EISDIR|EPERM|EACCES/u);
      return true;
    });
    const filename = readdirSync(directory).find((name) => name.endsWith(".failure.json"));
    assert(filename);
    const manifest = JSON.parse(readFileSync(join(directory, filename), "utf8"));
    assert.equal(manifest.accepted, false);
    assert.equal(manifest.phase, "evidence-io");
    assert.equal(manifest.protocolValidated, suffix === "json");
    assert.equal(manifest.exitCode, 0);
    assert.equal(manifest.signal, null);
    assert.equal(manifest.completeStreamsPersisted, true);
    assert.deepEqual(readFileSync(join(directory, manifest.streams.stdout.file)), canonical);
  }
});

await test("failure-only capture leaves success unchanged and legacy all-mode files intact", async (t) => {
  const temporary = mkdtempSync(join(tmpdir(), "vize-mounted-producer-success-"));
  const previous = keys.map((key) => [key, process.env[key]] as const);
  t.after(() => {
    for (const [key, value] of previous) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
    rmSync(temporary, { recursive: true, force: true });
  });
  const source = JSON.parse(canonical.toString()).code;
  const onlyFailures = join(temporary, "only-failures");
  process.env[keys[0]] = onlyFailures;
  process.env[keys[1]] = "1";
  assert.equal(await execute(canonical, Buffer.alloc(0), 0).result, source);
  assert.equal(existsSync(onlyFailures), false);
  const all = join(temporary, "all");
  process.env[keys[0]] = all;
  delete process.env[keys[1]];
  const { child, result } = execute(canonical, Buffer.alloc(0), 0);
  assert.equal(await result, source);
  assert.deepEqual(
    readdirSync(all).sort(),
    ["json", "stderr.bin", "stdout.bin"]
      .map((suffix) => `development-${child.pid}.${suffix}`)
      .sort(),
  );
  assert.deepEqual(readFileSync(join(all, `development-${child.pid}.stdout.bin`)), canonical);
  assert.deepEqual(readFileSync(join(all, `development-${child.pid}.stderr.bin`)), Buffer.alloc(0));
  const manifest = JSON.parse(readFileSync(join(all, `development-${child.pid}.json`), "utf8"));
  assert.deepEqual(manifest, {
    pid: child.pid,
    production: false,
    exitCode: 0,
    signal: null,
    spawnError: null,
    inputError: null,
    stdout: { bytes: canonical.length, sha256: hash(canonical) },
    stderr: { bytes: 0, sha256: hash(Buffer.alloc(0)) },
  });
});

await test("evidence I/O failure retains the original terminal failure as the first cause", async (t) => {
  const temporary = mkdtempSync(join(tmpdir(), "vize-mounted-producer-io-"));
  const previous = keys.map((key) => [key, process.env[key]] as const);
  t.after(() => {
    for (const [key, value] of previous) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
    rmSync(temporary, { recursive: true, force: true });
  });
  const blocked = join(temporary, "file");
  writeFileSync(blocked, "owned synthetic capture refusal\n");
  process.env[keys[0]] = blocked;
  process.env[keys[1]] = "1";
  await assert.rejects(
    execute(Buffer.from('{"code":"x"}\n'), Buffer.alloc(0), 7).result,
    (error) => {
      assert(error instanceof Error);
      assert(error.cause instanceof AggregateError);
      assert(error.cause.errors[0] instanceof assert.AssertionError);
      assert.equal(error.cause.errors[0].actual, 7);
      assert.equal(error.cause.errors[0].expected, 0);
      assert(error.cause.errors.length > 1);
      return true;
    },
  );
  const partial = join(temporary, "partial");
  process.env[keys[0]] = partial;
  const { child, result } = execute(Buffer.from('{"code":"x"}\n'), Buffer.alloc(0), 7);
  // close handlers cannot run before this synchronous setup yields.
  mkdirSync(join(partial, `development-${child.pid}-parent-${process.pid}.stdout.bin`), {
    recursive: true,
  });
  await assert.rejects(result, Error);
  const manifestName = readdirSync(partial).find((name) => name.endsWith(".failure.json"));
  assert(manifestName);
  const manifest = JSON.parse(readFileSync(join(partial, manifestName), "utf8"));
  assert.equal(manifest.accepted, false);
  assert.equal(manifest.exitCode, 7);
  assert.equal(manifest.completeStreamsPersisted, false);
  assert.equal(manifest.streams.stdout.persisted, false);
  assert.match(manifest.streams.stdout.error, /EISDIR|EPERM|EACCES/u);
  assert.equal(manifest.streams.stderr.persisted, true);
});

type CaptureStep = {
  name?: string;
  env?: Record<string, string>;
  run?: string;
  if?: string;
  with?: Record<string, string>;
};
type RustWorkflow = { jobs: { "pr-rust-shard": { steps: CaptureStep[] } } };

function captureStep(workflow: RustWorkflow, name: string): CaptureStep {
  const steps = workflow.jobs["pr-rust-shard"].steps.filter((step) => step.name === name);
  assert.equal(steps.length, 1, `exactly one ${name} step must retain the owned wiring`);
  return steps[0];
}

function rustCaptureWiring(workflow: RustWorkflow) {
  const invocation = captureStep(workflow, "Build and run the affected Rust test cohort");
  assert.equal(
    invocation.env?.VIZE_MOUNTED_RUNTIME_PRODUCER_EVIDENCE,
    "${{ runner.temp }}/mounted-runtime-producer-failures",
  );
  assert.equal(invocation.env?.VIZE_MOUNTED_RUNTIME_PRODUCER_EVIDENCE_FAILURES_ONLY, "1");
  assert.match(
    invocation.run ?? "",
    /run-affected-rust\.ts "\$RUNNER_TEMP\/rust-plan\.json" cargo nextest run @packages@/u,
  );
  const upload = captureStep(workflow, "Preserve rejected mounted runtime producer streams");
  assert.equal(upload.if, "${{ always() }}");
  assert.equal(
    upload.with?.name,
    "mounted-runtime-producer-failures-${{ matrix.cohort }}-${{ github.sha }}-${{ github.run_id }}-${{ github.run_attempt }}",
  );
  const paths = (upload.with?.path ?? "").trim().split(/\r?\n/u);
  assert.deepEqual(
    paths,
    ["failure.json", "stdout.bin", "stderr.bin"].map(
      (suffix) => "${{ runner.temp }}/mounted-runtime-producer-failures/*." + suffix,
    ),
  );
  assert.equal(upload.with?.["if-no-files-found"], "ignore");
}

await test("the exact Rust invocation captures failures and uploads only attempt-qualified streams", () => {
  const source = readFileSync(
    new URL("../../.github/workflows/pr-rust-products.yml", import.meta.url),
    "utf8",
  );
  const workflow = parse(source) as RustWorkflow;
  rustCaptureWiring(workflow);
  const reformatted = stringify(workflow, { lineWidth: 0 });
  assert.notEqual(reformatted, source);
  rustCaptureWiring(parse(reformatted) as RustWorkflow);
  const invocationName = "Build and run the affected Rust test cohort";
  const uploadName = "Preserve rejected mounted runtime producer streams";
  const mutations: ((changed: RustWorkflow) => void)[] = [
    (changed) => {
      delete captureStep(changed, invocationName).env![keys[0]];
    },
    (changed) => {
      captureStep(changed, invocationName).env![keys[1]] = "0";
    },
    (changed) => {
      captureStep(changed, uploadName).if = "${{ success() }}";
    },
    (changed) => {
      const settings = captureStep(changed, uploadName).with!;
      settings.name = settings.name.replace("-${{ github.run_attempt }}", "");
    },
    (changed) => {
      const settings = captureStep(changed, uploadName).with!;
      settings.path = settings.path.replace(
        "mounted-runtime-producer-failures/*.stdout.bin",
        "mounted-runtime-producer-failures/**",
      );
    },
  ];
  for (const mutate of mutations) {
    const changed = structuredClone(workflow);
    mutate(changed);
    assert.notDeepEqual(changed, workflow);
    assert.throws(() => rustCaptureWiring(changed));
  }
});
