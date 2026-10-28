/** One-SFC, untimed opt-in custody admission. Full emitted TS and CLI packets stay auditable. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import { prepareCacheCarrier } from "./typechecker-cache-carrier.ts";
import { coldScaleCommand } from "./cold-typechecker-scale-command.ts";
import { createColdBackendObserver } from "./typechecker-cache-backend.ts";
import { createColdScaleRuntime } from "./typechecker-cache-runtime.ts";
import { asArray, asNumber, asRecord, asString } from "../../support/json-value.ts";
const { values } = parseArgs({ options: { provider: { type: "string" }, builds: { type: "string" }, out: { type: "string" } } });
assert.ok(values.provider && values.builds && values.out);
const provider = fs.realpathSync(values.provider), out = path.resolve(values.out);
assert.equal(fs.existsSync(out), false); fs.mkdirSync(out, { recursive: true });
const sha = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
const fileSha = (p: string) => sha(fs.readFileSync(p));
const write = (name: string, value: unknown) => fs.writeFileSync(path.join(out, name), JSON.stringify(value, null, 2) + "\n");
const buildPacket = asRecord(JSON.parse(fs.readFileSync(values.builds, "utf8")));
assert.equal(buildPacket.schema, "vize.cache-custody-build-admission.v1");
const head = asString(buildPacket.sourceHead), builds = asArray(buildPacket.builds).map(v => asRecord(v));
assert.equal(builds.length, 3);
const project = path.join(out, "project");
prepareCacheCarrier(project, provider, 1);
const input = path.join(project, "src/Carrier0.vue"), good = fs.readFileSync(input, "utf8");
const bad = good.replace('const title: string = "carrier-0";', "const title: string = 1;");
assert.notEqual(good, bad);
const expectedDiagnostic = "error:3:7 [TS2322] Type 'number' is not assignable to type 'string'.";
const results: Record<string, unknown[]> = {};
const retained = [];
for (const build of builds) {
  const role = asString(build.role), binary = asString(build.binary);
  assert.equal(fileSha(binary), build.binarySha256);
  const packets = [];
  for (const phase of ["valid", "invalid", "repaired"]) {
    fs.writeFileSync(input, phase === "invalid" ? bad : good);
    const leaf = path.join(out, role + "-" + phase); fs.mkdirSync(leaf);
    const runtime = createColdScaleRuntime(leaf, path.join(out, "runtime-base"));
    const capture = path.join(leaf, "source-capture"); fs.mkdirSync(capture);
    assert.deepEqual(fs.readdirSync(capture), []);
    const observer = createColdBackendObserver(coldScaleCommand({ provider, out: leaf, tool: "cli", app: project, binary }).backend, leaf);
    // Preserve all emitted TypeScript bytes before the backend starts, excluding package symlinks.
    const injection = `
function collectInputs(directory, relative='') {
  const rows=[];
  for(const name of fs.readdirSync(directory).sort()) {
    const full=path.join(directory,name), rel=path.join(relative,name), stat=fs.lstatSync(full);
    if(stat.isSymbolicLink()) { rows.push({path:rel,kind:'symlink',target:fs.readlinkSync(full)}); continue; }
    if(stat.isDirectory()) { if(name!=='node_modules'&&name!=='.vize-buildinfo')rows.push(...collectInputs(full,rel)); continue; }
    if(stat.isFile()&&(/\\.tsx?$/.test(name))) {
      const bytes=fs.readFileSync(full);
      rows.push({path:rel,kind:'typescript',bytes:bytes.length,sha256:crypto.createHash('sha256').update(bytes).digest('hex'),base64:bytes.toString('base64')});
    }
  }
  return rows;
}
if(project) fs.writeFileSync(path.join(directory,'virtual-inputs.json'),JSON.stringify(collectInputs(process.cwd()),null,2)+'\\n');
`;
    let wrapper = fs.readFileSync(observer.wrapper, "utf8");
    assert.equal(wrapper.split("const before=snapshot(buildInfo)").length, 2);
    wrapper = wrapper.replace("const before=snapshot(buildInfo)", injection + "\nconst before=snapshot(buildInfo)");
    fs.writeFileSync(observer.wrapper, wrapper);
    const wrapperSha = fileSha(observer.wrapper), { command, backend } = coldScaleCommand({ provider, out: leaf, tool: "cli", app: project, binary });
    const backendSha = fileSha(backend), args = [...command.args];
    args[args.indexOf("--corsa-path") + 1] = observer.wrapper;
    const env: NodeJS.ProcessEnv = { ...process.env, ...command.env, ...runtime.env, VIZE_CACHE_CUSTODY_CAPTURE_DIR: capture };
    delete env.VIZE_LEXICAL_DOLLAR_CAPTURE_DIR; delete env.VIZE_LEXICAL_DOLLAR_CAPTURE_METADATA_ONLY;
    const tracePath = path.join(leaf, "exec.trace"), started = process.hrtime.bigint();
    const timePath = path.join(leaf, "time.txt");
    const result = spawnSync("/usr/bin/time", ["-v", "-o", timePath, "strace", "-f", "-qq", "-s", "65536", "-e", "trace=execve,chdir", "-e", "signal=none", "-o", tracePath, binary, ...args],
      { cwd: command.cwd, env, encoding: "utf8", timeout: 120_000, maxBuffer: 32 * 1024 * 1024 });
    const elapsedWallNs = String(process.hrtime.bigint() - started);
    fs.writeFileSync(path.join(leaf, "stdout.json"), result.stdout ?? "");
    fs.writeFileSync(path.join(leaf, "stderr.txt"), result.stderr ?? "");
    assert.equal(result.error, undefined); assert.equal(result.signal, null);
    assert.equal(result.status, phase === "invalid" ? 1 : 0, result.stderr);
    const cli = asRecord(JSON.parse(result.stdout));
    const diagnostics = asArray(cli.files).flatMap(f => {
      const row = asRecord(f); return asArray(row.diagnostics).map(message => [row.file, message]);
    });
    assert.deepEqual(diagnostics, phase === "invalid" ? [["src/Carrier0.vue", expectedDiagnostic]] : []);
    assert.equal(cli.errorCount, phase === "invalid" ? 1 : 0); assert.equal(cli.warningCount, 0);
    const metadata = fs.readdirSync(capture).flatMap(owner => {
      assert.match(owner, /^process-[0-9]+$/u);
      return fs.readdirSync(path.join(capture, owner)).sort().map(name => ({
        path: path.join(capture, owner, name), packet: asRecord(JSON.parse(fs.readFileSync(path.join(capture, owner, name), "utf8"))),
      }));
    });
    const traces = fs.readFileSync(tracePath, "utf8");
    const calls = fs.readdirSync(observer.calls).map(name => {
      const directory = path.join(observer.calls, name), before = asRecord(JSON.parse(fs.readFileSync(path.join(directory, "before.json"), "utf8")));
      const after = asRecord(JSON.parse(fs.readFileSync(path.join(directory, "after.json"), "utf8")));
      return { directory, before, after };
    }).filter(call => call.before.project !== null);
    assert.ok(calls.length > 0);
    const sourceRequired = role === "observer";
    assert.equal(metadata.length, sourceRequired ? calls.length * 2 : 0, "normal feature-off path must not observe capture env");
    const used = new Set<string>(), virtualPackets = [];
    for (const call of calls) {
      assert.equal(call.after.error, null); assert.equal(call.after.signal, null);
      assert.equal(asRecord(call.before.before).exists, false, "fresh runtime first use must have no build info");
      assert.ok(asString(call.before.physicalCwd).startsWith(asString(runtime.evidence.ownedRoot) + path.sep));
      const inputs = asArray(JSON.parse(fs.readFileSync(path.join(call.directory, "virtual-inputs.json"), "utf8"))).map(row => asRecord(row));
      assert.equal(inputs.filter(row => asString(row.path).endsWith("Carrier0.vue.ts")).length, 1);
      for (const row of inputs) if (row.kind === "typescript") {
        const bytes = Buffer.from(asString(row.base64), "base64"); assert.equal(bytes.length, row.bytes); assert.equal(sha(bytes), row.sha256);
      }
      const childPid = asNumber(call.after.backendPid);
      const execPrefix = childPid + " execve(" + JSON.stringify(backend) + ",";
      assert.ok(traces.split("\n").some(line => line.replace(/^\s+/u, "").startsWith(execPrefix)), "OS trace must witness real Go child exec, not wrapper identity");
      if (sourceRequired) {
        const pair = metadata.filter(row => row.packet.requestedConfigPath === call.before.project && row.packet.requestedConfigSha256 === call.before.configSha256);
        assert.equal(pair.length, 2);
        const requested = pair.find(row => row.packet.stage === "requested")!, completed = pair.find(row => row.packet.stage === "completed")!;
        assert.ok(requested && completed); assert.equal(requested.packet.completion, "PENDING");
        for (const row of pair) {
          assert.equal(used.has(row.path), false); used.add(row.path);
          assert.equal(row.packet.schema, "vize.cache-runtime-custody"); assert.equal(row.packet.version, 2);
          assert.equal(row.packet.metadataOnly, true); assert.equal(row.packet.timingEligible, false);
          assert.deepEqual(row.packet.errors, []); assert.equal(row.packet.compiledSourceRevision, head);
          assert.equal(row.packet.compiledSourceRevision, build.compilerSourceSeal);
          assert.equal(row.packet.projectRoot, fs.realpathSync(project));
          assert.equal(row.packet.virtualRoot, call.before.physicalCwd); assert.equal(row.packet.actualCwd, call.before.physicalCwd);
          assert.equal(row.packet.invokedProgram, observer.wrapper); assert.equal(row.packet.resolvedInvokedProgram, fs.realpathSync(observer.wrapper));
          assert.equal(row.packet.invokedProgramSha256, wrapperSha); assert.notEqual(row.packet.invokedProgramSha256, backendSha);
          assert.equal(row.packet.invokedPid, null);
          assert.deepEqual(row.packet.actualArgs, call.before.argv);
          assert.equal(sha(asString(row.packet.requestedConfigBytes)), call.before.configSha256);
          assert.ok(traces.split("\n").some(line => line.startsWith(asNumber(row.packet.receiptOwnerPid) + " execve(" + JSON.stringify(binary) + ",")), "receipt owner must be actual source CLI PID");
        }
        assert.equal(completed.packet.completion, "INVOKED_PROCESS_REAPED");
        for (const key of ["requestedConfigPath", "requestedConfigBytes", "requestedConfigSha256", "projectRoot", "virtualRoot", "actualArgs", "actualCwd", "compiledSourceRevision", "invokedProgramSha256"]) assert.deepEqual(requested.packet[key], completed.packet[key]);
        assert.deepEqual(Buffer.from(asArray(completed.packet.rawStdoutBytes).map(x => asNumber(x))), fs.readFileSync(path.join(call.directory, "stdout.txt")));
        assert.deepEqual(Buffer.from(asArray(completed.packet.rawStderrBytes).map(x => asNumber(x))), fs.readFileSync(path.join(call.directory, "stderr.txt")));
      }
      const normalize = (text: string) => text.split(asString(call.before.physicalCwd)).join("$OBSERVED_VIRTUAL_ROOT");
      virtualPackets.push({ config: normalize(fs.readFileSync(path.join(call.directory, "generated-tsconfig.json"), "utf8")),
        inputs: inputs.map(row => row.kind === "typescript" ? { path: row.path, text: normalize(Buffer.from(asString(row.base64), "base64").toString("utf8")) } : row),
        backendStdout: normalize(fs.readFileSync(path.join(call.directory, "stdout.txt"), "utf8")), backendStderr: normalize(fs.readFileSync(path.join(call.directory, "stderr.txt"), "utf8")) });
    }
    packets.push({ phase, cli, virtualPackets });
    assert.equal(fileSha(binary), build.binarySha256); assert.equal(fileSha(observer.wrapper), wrapperSha); assert.equal(fileSha(backend), backendSha);
    const timeRaw = fs.readFileSync(timePath, "utf8");
    const maxRssKiB = Number(timeRaw.match(/Maximum resident set size \(kbytes\): ([0-9]+)/u)?.[1]);
    assert.ok(Number.isSafeInteger(maxRssKiB) && maxRssKiB > 0);
    retained.push({ role, phase, observedWallNs: elapsedWallNs, rusageMaxRssKiB: maxRssKiB, rssScope: "GNU time child maximum; not simultaneous process-tree RSS or Rust live allocation", timingEligible: false, sourceWitnessRequired: sourceRequired, runtime: runtime.evidence, wrapper: { path: observer.wrapper, sha256: wrapperSha }, actualGo: { path: backend, sha256: backendSha, pids: calls.map(call => call.after.backendPid), authority: "separate wrapper spawn plus OS exec trace; not Rust invoked-program hash" },
      emittedTypeScriptFiles: virtualPackets.map(packet => packet.inputs.length), metadataFiles: metadata.map(row => row.path), coldCacheQualifiedOnlyForThisCarrier: true });
    runtime.cleanup();
  }
  results[role] = packets;
}
assert.deepEqual(results["normal-head"], results["normal-base"], "whole CLI/VTS/backend packets changed under default features");
assert.deepEqual(results.observer, results["normal-head"], "observer perturbed whole CLI/VTS/backend packets");
write("acceptance.json", { schema: "vize.cache-custody-admission.v1", accepted: true, sourceHead: head, base: buildPacket.base, sourceBuilds: builds,
  fixture: { sfcs: 1, goodSha256: sha(good), invalidSha256: sha(bad) }, normalWholePacketsEqual: true, observerWholePacketsEqual: true, retained,
  scope: "untimed one-SFC carrier only; observer whole binary hash, output JSON, wrapper snapshots and strace perturb CPU/RSS",
  noCacheAuthority: "fresh owned UUID + actual pre-backend buildInfo ENOENT; OS page cache not flushed",
  fullCorpusQualified: false, shippingPerformanceQualified: false, memoryOverheadMeasured: false });
console.log(JSON.stringify({ accepted: true, sfcs: 1, executions: retained.length, wholePacketsEqual: true, fullCorpusQualified: false }));
