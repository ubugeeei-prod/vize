/** Untimed whole-packet admission for real 1/2/8-shard Corsa executions. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join, relative, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const hash = (value: Buffer | string) => createHash("sha256").update(value).digest("hex");
const record = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === "object" && !Array.isArray(value);
const quote = (value: string) => "'" + value.replaceAll("'", "'\\''") + "'";
export const partitionBackendObserver = (binary: string, trace: string) =>
  String.raw`
import fs from "node:fs"; import path from "node:path"; import cp from "node:child_process";
const binary=BINARY, trace=TRACE, args=process.argv.slice(2);
const out=fs.mkdtempSync(path.join(trace,"call-"));
const projectIndex=args.indexOf("--project"), config=projectIndex<0?null:args[projectIndex+1];
const before=config?fs.readFileSync(config):null;
const buildIndex=args.indexOf("--tsBuildInfoFile"), build=buildIndex<0?null:args[buildIndex+1];
const buildExisted=build?fs.existsSync(build):null;
const child=cp.spawnSync(binary,args,{stdio:["ignore","pipe","pipe"],maxBuffer:32*1024*1024});
fs.writeFileSync(path.join(out,"stdout.raw"),child.stdout??Buffer.alloc(0));
fs.writeFileSync(path.join(out,"stderr.raw"),child.stderr??Buffer.alloc(0));
if(before)fs.writeFileSync(path.join(out,"config.json"),before);
fs.writeFileSync(path.join(out,"receipt.json"),JSON.stringify({binary,args,config,buildInfoExisted:buildExisted,status:child.status,signal:child.signal,error:child.error?.message??null}));
process.stdout.write(child.stdout??Buffer.alloc(0));process.stderr.write(child.stderr??Buffer.alloc(0));
if(child.error||child.signal)process.exitCode=1;else process.exitCode=child.status??1;
`
    .replace("BINARY", JSON.stringify(binary))
    .replace("TRACE", JSON.stringify(trace));
type Input = { path: string; valid: string; errorLine: number | null };
export function partitionWeightInputs(root: string) {
  const inputs: Input[] = [],
    add = (name: string, valid: string, errorLine: number | null) => {
      const path = join(root, name);
      inputs.push({ path, valid, errorLine });
    };
  for (let index = 0; index < 8; index++) {
    add("src/g" + index + "/leaf.ts", "export const n = 1;\n", 1);
    add("src/g" + index + "/barrel.ts", "import {n} from './leaf'; export const value = n;\n", 1);
    add(
      "src/g" + index + "/Tiny.vue",
      '<script setup lang="ts">\nimport { value } from "@/g' +
        index +
        '/barrel";\n' +
        'import { n } from "./leaf";\n' +
        "const row: { id: number } = { id: value + n };\n</script><template><p>{{ row.id }}</p></template>\n",
      2,
    );
  }
  add("src/shared.ts", "export interface SharedWeight { id: number }\n", null);
  add(
    "src/globals.d.ts",
    "export {}; declare global { interface Window { partitionWeight: number } }\n",
    null,
  );
  return inputs;
}
export function verifyPartitionWeightPacket(
  output: Buffer,
  root: string,
  inputs: Input[],
  invalid: boolean,
) {
  const packet: unknown = JSON.parse(output.toString("utf8"));
  assert.ok(record(packet) && Array.isArray(packet.files));
  assert.equal(packet.fileCount, inputs.length);
  assert.equal(packet.files.length, inputs.length);
  const names = new Set<string>(),
    actual: string[] = [];
  for (const entry of packet.files) {
    assert.ok(record(entry) && typeof entry.file === "string" && Array.isArray(entry.diagnostics));
    const name = relative(root, resolve(root, entry.file)).replaceAll("\\", "/");
    assert.ok(!names.has(name), "duplicate file row");
    names.add(name);
    for (const value of entry.diagnostics) {
      assert.equal(typeof value, "string");
      assert.ok(typeof value === "string");
      const match = /^error:(\d+):(\d+) \[TS(\d+)\] (.+)$/u.exec(value);
      assert.ok(match, "unexpected warning or malformed diagnostic: " + value);
      assert.match(match[4] ?? "", /not assignable to type 'number'/u);
      actual.push([name, match[1], match[2], match[3]].join(":"));
    }
  }
  assert.deepEqual(
    [...names].sort(),
    inputs.map((value) => relative(root, value.path).replaceAll("\\", "/")).sort(),
  );
  const expected = invalid
    ? inputs
        .filter((value) => value.errorLine !== null)
        .map((value) =>
          [relative(root, value.path).replaceAll("\\", "/"), value.errorLine, 7, 2322].join(":"),
        )
        .sort()
    : [];
  assert.equal(actual.length, expected.length);
  assert.deepEqual(actual.sort(), expected);
  assert.equal(packet.errorCount, expected.length);
  assert.equal(packet.warningCount, 0);
  return packet;
}
export function runPartitionWeightCli(binary: string, provider: string, out: string) {
  binary = resolve(binary);
  provider = resolve(provider);
  out = resolve(out);
  assert.ok(!existsSync(out), "fresh functional evidence required");
  mkdirSync(out, { recursive: true });
  const project = join(out, "project");
  mkdirSync(project);
  symlinkSync(join(provider, "node_modules"), join(project, "node_modules"), "dir");
  writeFileSync(
    join(project, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        target: "ES2022",
        module: "ESNext",
        moduleResolution: "Bundler",
        strict: true,
        noEmit: true,
        skipLibCheck: true,
        jsx: "preserve",
        paths: { "@/*": ["./src/*"] },
      },
      include: ["src/**/*"],
    }),
  );
  const native = join(provider, "node_modules/@typescript/typescript-linux-x64/lib/tsc");
  const binaryHash = hash(readFileSync(binary)),
    nativeHash = hash(readFileSync(native));
  assert.ok(process.env.PARTITION_WEIGHT_BUILD_RECEIPT, "bound source build receipt required");
  const build: unknown = JSON.parse(
    readFileSync(process.env.PARTITION_WEIGHT_BUILD_RECEIPT, "utf8"),
  );
  assert.ok(
    record(build) && typeof build.source === "string" && /^[0-9a-f]{40}$/u.test(build.source),
  );
  assert.ok(record(build.artifact) && typeof build.artifact.executable === "string");
  assert.equal(resolve(build.artifact.executable), binary);
  assert.equal(build.sha256, binaryHash);
  assert.equal(build.allocation, false);
  assert.equal(build.targetStartedEmpty, true);
  assert.equal(build.artifact.fresh, false);
  const inputs = partitionWeightInputs(project),
    results: unknown[] = [];
  for (const state of ["valid", "invalid", "repaired"]) {
    const invalid = state === "invalid";
    for (const input of inputs) {
      const lines = input.valid.split("\n");
      if (invalid && input.errorLine !== null)
        lines.splice(input.errorLine - 1, 0, "const badWeight: number = 'invalid';");
      mkdirSync(resolve(input.path, ".."), { recursive: true });
      writeFileSync(input.path, lines.join("\n"));
    }
    let reference: Buffer | undefined;
    for (const servers of [1, 2, 8]) {
      const run = join(out, state + "-" + servers),
        trace = join(run, "backend");
      mkdirSync(trace, { recursive: true });
      const script = join(run, "observe.mjs"),
        launcher = join(run, "tsgo");
      writeFileSync(script, partitionBackendObserver(native, trace));
      writeFileSync(
        launcher,
        "#!/bin/sh\nexec " + quote(process.execPath) + " " + quote(script) + ' "$@"\n',
        { mode: 0o755 },
      );
      const cache = join(run, "cache"),
        tmp = join(run, "tmp");
      mkdirSync(cache);
      mkdirSync(tmp);
      const args = [
        "check",
        "--no-config",
        "--format",
        "json",
        "--tsconfig",
        "tsconfig.json",
        "--servers",
        String(servers),
        "--checkers",
        "1",
        "--corsa-path",
        launcher,
      ];
      const child = spawnSync(binary, args, {
        cwd: project,
        timeout: 120_000,
        maxBuffer: 32 * 1024 * 1024,
        env: {
          ...process.env,
          LC_ALL: "C",
          NO_COLOR: "1",
          XDG_CACHE_HOME: cache,
          TMPDIR: tmp,
          GOMAXPROCS: "2",
          RAYON_NUM_THREADS: "2",
        },
      });
      const stdout = child.stdout ?? Buffer.alloc(0),
        stderr = child.stderr ?? Buffer.alloc(0);
      writeFileSync(join(run, "stdout.json"), stdout);
      writeFileSync(join(run, "stderr.txt"), stderr);
      assert.equal(child.error, undefined, child.error?.message);
      assert.equal(child.signal, null);
      assert.ok(invalid ? child.status === 1 || child.status === 2 : child.status === 0);
      verifyPartitionWeightPacket(stdout, project, inputs, invalid);
      if (reference)
        assert.deepEqual(stdout, reference, "complete packet changed with shard width");
      else reference = stdout;
      const calls = readdirSync(trace).map(
        (name) => JSON.parse(readFileSync(join(trace, name, "receipt.json"), "utf8")) as unknown,
      );
      const programs = calls.filter((value) => record(value) && typeof value.config === "string");
      assert.equal(programs.length, servers, "cost model did not enter the requested shard plan");
      for (const value of programs) {
        assert.ok(record(value) && Array.isArray(value.args));
        assert.equal(value.args[value.args.indexOf("--checkers") + 1], "1");
        assert.equal(value.buildInfoExisted, false);
        assert.equal(value.signal, null);
        assert.equal(value.error, null);
      }
      results.push({
        state,
        servers,
        backendPrograms: programs.length,
        status: child.status,
        fullPacketSha256: hash(stdout),
        stderrSha256: hash(stderr),
        inputs: inputs.length,
        expectedErrors: invalid ? 24 : 0,
        argv: args,
        calls,
      });
    }
  }
  assert.equal(hash(readFileSync(binary)), binaryHash);
  assert.equal(hash(readFileSync(native)), nativeHash);
  const summary = {
    schemaVersion: 1,
    productSource: build.source,
    observerEventSource: process.env.GITHUB_SHA ?? null,
    binary,
    binaryHash,
    native,
    nativeHash,
    results,
    timingUsable: false,
    allocationCounts: null,
    scope:
      "unprofiled source CLI functional admission with byte-forwarding backend argv observer; no latency/RSS claim",
  };
  writeFileSync(join(out, "summary.json"), JSON.stringify(summary, null, 2) + "\n");
  console.log(
    JSON.stringify({
      cases: results.length,
      authoredFiles: inputs.length,
      mutationErrors: 24,
      shardWidths: [1, 2, 8],
      passed: true,
    }),
  );
  return summary;
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [binary, provider, out] = process.argv.slice(2);
  assert.ok(binary && provider && out);
  runPartitionWeightCli(binary, provider, out);
}
