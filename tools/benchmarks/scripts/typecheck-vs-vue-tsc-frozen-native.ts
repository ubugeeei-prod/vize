/** Untimed forwarding custody only. Unknown invocations stay visible and cannot qualify. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { hash, identity, save } from "./typecheck-vs-vue-tsc-frozen-evidence.ts";
import { classifyRaw, validateNativeArgs } from "./typecheck-vs-vue-tsc-frozen-inputs.ts";
const files = [
  "typecheck-vs-vue-tsc-frozen-native-forward.ts",
  "typecheck-vs-vue-tsc-frozen-inputs.ts",
];
export function forwarder(out: string) {
  const directory = path.join(out, "native-observer/.bin");
  fs.mkdirSync(directory, { recursive: true });
  const bodies = files.map((file) => {
    const body = fs.readFileSync(new URL("./" + file, import.meta.url));
    fs.writeFileSync(path.join(directory, file), body, { flag: "wx" });
    return { file, bytes: body.length, sha256: hash(body) };
  });
  fs.writeFileSync(path.join(directory, "package.json"), '{"type":"module"}\n', { flag: "wx" });
  const executable = path.join(directory, files[0]);
  fs.chmodSync(executable, 0o755);
  save(path.join(out, "forwarder.json"), {
    executable,
    bodies,
    node: identity(process.execPath),
    classification:
      "regular canonical physical .bin outside node_modules; OS Node24 shebang; CLI/probe/version-only observation, not API qualification",
  });
  return executable;
}
export function readCall(directory: string, native: string, project: string, provider: string) {
  const read = (name: string) => JSON.parse(fs.readFileSync(path.join(directory, name), "utf8"));
  const invocation = read("invocation.json");
  assert.equal(invocation.imageBefore, identity(native).sha256);
  assert.equal(invocation.executable, fs.realpathSync(native));
  assert.equal(invocation.nodeExecutable, fs.realpathSync(process.execPath));
  assert.equal(invocation.nodeVersion, process.version);
  assert.equal(invocation.nodeSha256, identity(process.execPath).sha256);
  assert.equal(invocation.canonicalForwarder, fs.realpathSync(invocation.canonicalForwarder));
  assert.equal(path.basename(path.dirname(invocation.canonicalForwarder)), ".bin");
  assert.notEqual(
    path.basename(path.dirname(path.dirname(invocation.canonicalForwarder))),
    "node_modules",
  );
  const request = read("request.json"),
    status = read("native-replay.status.json");
  assert.deepEqual(request.argv, invocation.argv);
  assert.equal(request.cwd, invocation.cwd);
  assert.equal(request.executable, invocation.executable);
  assert.equal(request.imageBefore, invocation.imageBefore);
  assert.equal(request.canonicalForwarder, invocation.canonicalForwarder);
  assert.equal(status.signal, null);
  assert.equal(status.error, null);
  assert([0, 1, 2].includes(status.status));
  if (request.kind === "main-cli") {
    validateNativeArgs(request.argv, request.config);
    assert.equal(request.buildInfo, request.argv[4]);
    assert.equal(request.buildInfoInitiallyAbsent, true);
  } else if (request.kind === "option-probe")
    assert.deepEqual(request.argv, ["--pretty", "false", "--project", request.config]);
  else {
    assert.equal(request.kind, "version");
    assert.deepEqual(request.argv, ["--version"]);
    assert.equal(status.status, 0);
  }
  const before = read("inputs-before.json"),
    after = read("inputs-after.json");
  assert.deepEqual(
    before.roots,
    (request.config ? [project, provider, request.cwd] : [project, provider]).map((root) =>
      fs.realpathSync(root),
    ),
  );
  assert.deepEqual(after, before);
  assert.equal(hash(JSON.stringify(before.rows)), before.sha256);
  assert.equal(read("image-after.json").sha256, request.imageBefore);
  const stdout = fs.readFileSync(path.join(directory, "native-replay.stdout.bin")),
    stderr = fs.readFileSync(path.join(directory, "native-replay.stderr.bin"));
  return {
    directory,
    invocation,
    request,
    status,
    inputs: before,
    stdout: identity(path.join(directory, "native-replay.stdout.bin")),
    stderr: identity(path.join(directory, "native-replay.stderr.bin")),
    classification: classifyRaw(Buffer.concat([stdout, stderr]), request.cwd, project, provider),
  };
}
export function collect(out: string, native: string, project: string, provider: string) {
  const calls = [],
    refusals: { directory: string; error: string }[] = [];
  for (const name of fs.readdirSync(out).sort()) {
    const directory = path.join(out, name);
    try {
      assert(/^call-[A-Za-z0-9]+$/u.test(name));
      calls.push(readCall(directory, native, project, provider));
    } catch (error) {
      refusals.push({ directory, error: String(error) });
    }
  }
  const main = calls.filter((call) => call.request.kind === "main-cli");
  const result = {
    calls,
    refusals,
    observedMainCalls: main.length,
    complete: main.length > 0 && refusals.length === 0,
    nativeClean:
      main.length > 0 &&
      main.every((call) => call.status.status === 0 && call.classification.groups.length === 0),
    loadedNativeFileVector:
      "UNOBSERVED; complete potential-input/materialized seals are not loaded-file membership",
    historicalDebt:
      "prior retained2500 observed624 TS5097/status2 while authored CLI0; current whole packets classified independently; no native-clean transfer",
  };
  save(path.join(path.dirname(out), "native-observations.json"), result);
  return result;
}
