/** Buffered-pipe regression: preserve >1 MiB on both streams and nonzero status. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmodSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import test from "node:test";

await test("native forwarding drains stdout/stderr pipes and preserves nonzero status", () => {
  const dir = mkdtempSync(join(os.tmpdir(), "native-phase-forwarding-"));
  try {
    const runtime = join(dir, "mock-native.mjs");
    const member = join(dir, "member.ts");
    const config = join(dir, "tsconfig.json");
    const settings = join(dir, "settings.json");
    writeFileSync(member, "export const value: number = 1;\n");
    writeFileSync(config, JSON.stringify({ compilerOptions: { noEmit: true }, files: [member] }));
    writeFileSync(settings, JSON.stringify({ runtime, directory: join(dir, "captured") }));
    const stdout = "member.ts(1,1): error TS2322: phase-雪\n".repeat(40_000);
    const stderr = "native-stderr-🌸\n".repeat(80_000);
    const footer =
      "Files: 1\nLines: 1\nIdentifiers: 1\nSymbols: 1\nTypes: 1\nInstantiations: 0\nMemory used: 1K\nMemory allocs: 1\nConfig time: 0.001s\nParse time: 0.001s\nBind time: 0.001s\nCheck time: 0.001s\nEmit time: 0.000s\nTotal time: 0.004s\n";
    assert(Buffer.byteLength(stdout) > 1024 * 1024 && Buffer.byteLength(stderr) > 1024 * 1024);
    // This mock is a transport control, not native phase/performance evidence.
    writeFileSync(
      runtime,
      `#!/usr/bin/env node
const args = process.argv.slice(2);
if (args.includes("--listFilesOnly")) {
  process.stdout.write(${JSON.stringify(`${member}\n`)});
} else {
  process.stdout.write(${JSON.stringify(stdout)});
  if (args.includes("--extendedDiagnostics")) process.stdout.write(${JSON.stringify(footer)});
  process.stderr.write(${JSON.stringify(stderr)});
  process.exitCode = 3;
}
`,
    );
    chmodSync(runtime, 0o755);
    const wrapper = fileURLToPath(
      new URL("./typechecker-native-phase-capture.mjs", import.meta.url),
    );
    const hash = (value) => createHash("sha256").update(value).digest("hex");
    for (const args of [["--project", config, "--checkers", "1"], ["--version"]]) {
      const result = spawnSync(process.execPath, [wrapper, ...args], {
        cwd: dir,
        env: { ...process.env, NATIVE_PHASE_CONFIG: settings },
        timeout: 60_000,
        maxBuffer: 32 * 1024 * 1024,
      });
      assert.equal(result.error, undefined, result.error?.message);
      assert.equal(result.signal, null);
      assert.equal(result.status, 3, "native nonzero status was lost");
      assert.equal(result.stdout.length, Buffer.byteLength(stdout));
      assert.equal(result.stderr.length, Buffer.byteLength(stderr));
      assert.equal(hash(result.stdout), hash(stdout), "stdout bytes were truncated or altered");
      assert.equal(hash(result.stderr), hash(stderr), "stderr bytes were truncated or altered");
    }
    const receipts = readdirSync(join(dir, "captured")).filter((file) => /^\d+\.json$/u.test(file));
    assert.equal(receipts.length, 1);
    const receipt = JSON.parse(readFileSync(join(dir, "captured", receipts[0]), "utf8"));
    assert.equal(receipt.runs.baseline.status, 3);
    assert.equal(receipt.nativeGraphBytesUnchanged, true);
    assert.equal(hash(receipt.phases.diagnosticText), hash(stdout));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

await test("pprof forwarding preserves product bytes and retains validated container and graph custody", () => {
  const dir = mkdtempSync(join(os.tmpdir(), "native-phase-pprof-forwarding-"));
  try {
    const runtime = join(dir, "mock-native.mjs");
    const member = join(dir, "member.ts");
    const config = join(dir, "tsconfig.json");
    const settings = join(dir, "settings.json");
    const trace = join(dir, "runtime-invocations.ndjson");
    const captured = join(dir, "captured");
    writeFileSync(member, "export const value: number = 1;\n");
    writeFileSync(config, JSON.stringify({ compilerOptions: { noEmit: true }, files: [member] }));
    writeFileSync(settings, JSON.stringify({ runtime, directory: captured, profile: "pprof" }));
    const stdout = "member.ts(1,1): error TS2322: profile-雪\n".repeat(40_000);
    const stderr = "native-profile-stderr-🌸\n".repeat(80_000);
    const footer =
      "Files: 1\nLines: 1\nIdentifiers: 1\nSymbols: 1\nTypes: 1\nInstantiations: 0\nMemory used: 1K\nMemory allocs: 1\nConfig time: 0.001s\nParse time: 0.001s\nBind time: 0.001s\nCheck time: 0.001s\nEmit time: 0.000s\nTotal time: 0.004s\n";
    // Deliberately fake gzip payloads prove container custody, never pprof semantics.
    writeFileSync(
      runtime,
      `#!/usr/bin/env node
import { appendFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { gzipSync } from 'node:zlib';
const args = process.argv.slice(2);
appendFileSync(${JSON.stringify(trace)}, JSON.stringify({ pid: process.pid, args }) + '\\n');
if (args.includes('--listFilesOnly')) {
  process.stdout.write(${JSON.stringify(`${member}\n`)});
} else {
  process.stdout.write(${JSON.stringify(stdout)});
  if (args.includes('--extendedDiagnostics')) process.stdout.write(${JSON.stringify(footer)});
  const profile = args.indexOf('--pprofDir');
  if (profile >= 0) {
    const cpu = join(args[profile + 1], process.pid + '-cpuprofile.pb.gz');
    const mem = join(args[profile + 1], process.pid + '-memprofile.pb.gz');
    writeFileSync(cpu, gzipSync(Buffer.from('fake CPU container')));
    writeFileSync(mem, gzipSync(Buffer.from('fake allocations container')));
    process.stdout.write('Memory profile: ' + mem + '\\nCPU profile: ' + cpu + '\\n');
  }
  process.stderr.write(${JSON.stringify(stderr)});
  process.exitCode = 3;
}
`,
    );
    chmodSync(runtime, 0o755);
    const wrapper = fileURLToPath(
      new URL("./typechecker-native-phase-capture.mjs", import.meta.url),
    );
    const args = ["--checkers", "1", "--pretty", "false", "--project", config];
    const run = (argv) =>
      spawnSync(process.execPath, [wrapper, ...argv], {
        cwd: dir,
        env: { ...process.env, NATIVE_PHASE_CONFIG: settings },
        timeout: 60_000,
        maxBuffer: 32 * 1024 * 1024,
      });
    const result = run(args);
    assert.equal(result.error, undefined, result.error?.message);
    assert.equal(result.signal, null);
    assert.equal(result.status, 3);
    assert.deepEqual(result.stdout, Buffer.from(stdout));
    assert.deepEqual(result.stderr, Buffer.from(stderr));
    const files = readdirSync(captured).filter((file) => /^\d+\.json$/u.test(file));
    assert.equal(files.length, 1);
    const receipt = JSON.parse(readFileSync(join(captured, files[0]), "utf8"));
    const hash = (value) => createHash("sha256").update(value).digest("hex");
    assert.deepEqual(receipt.args, args);
    assert.equal(receipt.orderedDiagnosticsEqual, true);
    assert.equal(receipt.nativeGraphBytesUnchanged, true);
    assert.equal(receipt.nativeGraphArchive.graphClosureClaimed, false);
    const graphBytes = readFileSync(receipt.nativeGraphArchive.record);
    assert.equal(hash(graphBytes), receipt.nativeGraphArchive.sha256);
    const graph = JSON.parse(graphBytes.toString("utf8"));
    assert.equal(graph.graphClosureClaimed, false);
    assert.deepEqual(graph.args, args);
    assert.equal(graph.membersBefore.length, 1);
    assert.deepEqual(graph.membersAfter, graph.membersBefore);
    const object = readFileSync(
      join(dirname(receipt.nativeGraphArchive.record), graph.membersBefore[0].object),
    );
    assert.deepEqual(object, readFileSync(member));
    assert.equal(hash(object), graph.membersBefore[0].sha256);
    assert.deepEqual(
      readFileSync(join(dirname(receipt.nativeGraphArchive.record), graph.config.rawFile)),
      readFileSync(config),
    );
    const profile = receipt.nativeProfile;
    assert.equal(profile.validation, "passed");
    assert.equal(profile.instrumentationOnly, true);
    assert.equal(profile.excludedFromTimings, true);
    assert.deepEqual(profile.original.command, [runtime, ...args]);
    assert.deepEqual(profile.replay.command, [
      runtime,
      ...args,
      "--pprofDir",
      profile.profileDirectory,
    ]);
    assert.equal(profile.original.status, 3);
    assert.equal(profile.replay.status, 3);
    assert.equal(profile.stdoutBytesEqualAfterExactProfileSuffix, true);
    assert.equal(profile.stderrBytesEqual, true);
    assert.equal(profile.nativeBinaryUnchanged, true);
    assert.deepEqual(readFileSync(profile.original.stdout.path), Buffer.from(stdout));
    assert.deepEqual(readFileSync(profile.original.stderr.path), Buffer.from(stderr));
    const expectedPayload = {
      cpu: "fake CPU container",
      allocations: "fake allocations container",
    };
    assert.deepEqual(
      profile.profileFiles.map((file) => file.kind),
      ["cpu", "allocations"],
    );
    for (const file of profile.profileFiles) {
      const suffix = file.kind === "cpu" ? "cpuprofile.pb.gz" : "memprofile.pb.gz";
      assert.equal(basename(file.path), `${profile.replay.pid}-${suffix}`);
      const bytes = readFileSync(file.path);
      assert.equal(hash(bytes), file.sha256);
      assert.equal(bytes.length, file.bytes);
      const unpacked = gunzipSync(bytes);
      assert.equal(unpacked.toString("utf8"), expectedPayload[file.kind]);
      assert.equal(unpacked.length, file.unpackedBytes);
    }
    assert.deepEqual(
      JSON.parse(readFileSync(profile.receiptPath, "utf8")),
      (({ receiptPath: _path, ...saved }) => saved)(profile),
    );
    assert.equal(receipt.profileSemantics.cpuSamples, null);
    assert.equal(receipt.profileSemantics.allocationSamples, null);
    assert.equal(receipt.profileSemantics.phaseAttribution, null);
    assert.match(receipt.profileSemantics.decoding, /not implemented.*gzip container/u);
    assert.equal(receipt.phases.unsupportedFields.startupTimeMs.value, null);
    const invocations = () =>
      readFileSync(trace, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line));
    const calls = invocations();
    assert.equal(calls.length, 5);
    assert.deepEqual(calls[0].args, [...args, "--listFilesOnly"]);
    assert.deepEqual(calls[1].args, args);
    assert.deepEqual(calls[2].args, [...args, "--extendedDiagnostics"]);
    assert.deepEqual(calls[3].args, [...args, "--pprofDir", profile.profileDirectory]);
    assert.equal(calls[3].pid, profile.replay.pid);
    assert.deepEqual(calls[4].args, [...args, "--listFilesOnly"]);
    for (const wrongArgs of [
      ["--checkers", "2", ...args.slice(2)],
      [...args, "--skipLibCheck"],
      ["--project", config, "--checkers", "1", "--pretty", "false"],
    ]) {
      const refused = run(wrongArgs);
      assert.equal(refused.error, undefined, refused.error?.message);
      assert.equal(refused.status, 1);
      assert.equal(refused.stdout.length, 0);
      assert.match(refused.stderr.toString("utf8"), /exact known production native argv/u);
      assert.deepEqual(invocations(), calls, "rejected argv still invoked the runtime");
    }
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
