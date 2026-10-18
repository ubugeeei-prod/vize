import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { inspect } from "node:util";
import {
  createSandboxRunner,
  SANDBOX_IMAGE,
  SandboxRuntimeError,
} from "../../npm/plugin-sdk/sandbox.js";

const batch = JSON.stringify({
  schema: 1,
  plugin: "sandbox-proof",
  file: "Proof.vue",
  parents: [-1],
  nodes: [{ id: 0, kind: "ui.element", tag: "button", span: { start: 0, end: 17 } }],
  facts: {},
});
const rule = (
  source: string,
  configuration?: unknown,
  limits?: { timeoutMs?: number; maxBytes?: number },
) =>
  createSandboxRunner(
    {
      name: "sandbox-proof",
      version: "1",
      rules: { check: source },
      configuration: configuration as never,
      cacheInputs: [],
    },
    limits,
  );

function assertNoContainers() {
  const result = spawnSync(
    "docker",
    [
      "ps",
      "-a",
      "--filter",
      `label=vize.plugin-sandbox.host=${process.pid}`,
      "--format",
      "{{.Names}}",
    ],
    { encoding: "utf8", timeout: 5000, maxBuffer: 16384 },
  );
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout.trim(), "", "sandbox containers must be removed after every run");
}

test("sandbox rejects malformed manifests and never evaluates callback sources on the host", () => {
  assert.throws(
    () => createSandboxRunner({ name: "x", version: "1", rules: { check: "42" } }),
    /function source/,
  );
  assert.throws(
    () => createSandboxRunner({ name: "x", version: "1", rules: { check: "() => {" } }),
    SyntaxError,
  );
  assert.throws(
    () => createSandboxRunner({ name: "x", version: "1", family: "unknown" } as never),
    /family/,
  );
  assert.throws(
    () =>
      createSandboxRunner({ name: "x", version: "1", family: "provider", callback: "() => ({})" }),
    /provides/,
  );
  assert.throws(() => rule("() => {}", { secret: () => 1 }), /JSON data/);
  assert.throws(() => rule("() => {}", null, { timeoutMs: Infinity }), /timeoutMs/);
  assert.throws(
    () =>
      createSandboxRunner({
        name: "x",
        version: "1",
        rules: { check: "() => {}" },
        mounts: ["/"],
      } as never),
    /field/,
  );
  const callback =
    "() => { process.getBuiltinModule('fs').writeFileSync('/host-evaluation-must-not-happen', 'bad'); }";
  assert.equal(typeof rule(callback).run, "function");
});

test("real Docker sandbox executes every hook family over serialized batches deterministically", () => {
  try {
    const docker = spawnSync("docker", ["image", "inspect", SANDBOX_IMAGE], {
      encoding: "utf8",
      timeout: 5000,
    });
    assert.equal(
      docker.status,
      0,
      `real sandbox proof requires the pinned image: ${docker.stderr}`,
    );
    const report = rule("(ctx, config) => { ctx.report(ctx.nodes[0], config.message); }", {
      message: "日本語😀",
    });
    const expected = [{ rule: "check", node: 0, message: "日本語😀" }];
    assert.deepEqual(JSON.parse(report.run(batch)), expected);
    assert.equal(report.run(batch), report.run(batch));
    const transform = createSandboxRunner({
      name: "x",
      version: "1",
      family: "transform",
      callback:
        "batch => batch.nodes.map(node => ({ kind: 'replace-static-attribute', node: node.id, name: 'disabled', value: null }))",
    });
    assert.deepEqual(
      JSON.parse(
        transform.run(
          JSON.stringify({
            schema: 1,
            stage: "s2-precanonical-static-attributes",
            nodes: [{ id: 0 }],
          }),
        ),
      ),
      {
        schema: 1,
        edits: [{ kind: "replace-static-attribute", node: 0, name: "disabled", value: null }],
      },
    );
    const provider = createSandboxRunner({
      name: "x",
      version: "1",
      family: "provider",
      provides: ["x/labels"],
      callback: "batch => ({ 'x/labels': batch.nodes.map(node => [node.id, 'known']) })",
    });
    assert.deepEqual(JSON.parse(provider.run(batch)), { "x/labels": [[0, "known"]] });
    for (const family of ["formatter", "output"] as const) {
      const hook = createSandboxRunner({
        name: "x",
        version: "1",
        family,
        callback:
          family === "formatter"
            ? "() => [{start: 0, end: 0, text: '// hi\\n'}]"
            : "() => [{placement: 'append', comment: 'verified'}]",
      });
      const outputBatch = JSON.stringify({
        schema: 1,
        family,
        offsetEncoding: "utf8",
        compiled: { code: "hello" },
      });
      assert.equal(JSON.parse(hook.run(outputBatch)).length, 1);
    }
    assertNoContainers();
  } catch (error) {
    console.error(
      JSON.stringify({
        phase: "plugin-sandbox-hook-families",
        image: SANDBOX_IMAGE,
        batch,
        error: inspect(error, { depth: null, breakLength: Infinity, compact: false }),
      }),
    );
    throw error;
  }
});

test("real sandbox cannot read host secrets, write its root, access Docker or host network", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-sandbox-host-secret-"));
  const secret = join(root, "private-secret");
  writeFileSync(secret, "host-secret-must-not-cross-the-boundary");
  process.env.VIZE_SANDBOX_HOST_SECRET = "environment-secret-must-not-cross";
  try {
    const probe = rule(
      `(ctx, config) => {
      const fs = process.getBuiltinModule('fs');
      for (const path of [config.secret, '/var/run/docker.sock']) {
        let denied = false;
        try { fs.readFileSync(path); } catch { denied = true; }
        if (!denied) throw new Error('host path became readable');
      }
      let readonly = false;
      try { fs.writeFileSync('/tmp/sandbox-write-probe', 'bad'); } catch (e) { readonly = e.code === 'EROFS'; }
      if (!readonly) throw new Error('container root became writable');
      if (process.env.VIZE_SANDBOX_HOST_SECRET) throw new Error('host environment leaked');
      const interfaces = process.getBuiltinModule('os').networkInterfaces();
      if (Object.values(interfaces).flat().some(entry => !entry.internal)) throw new Error('external network attached');
      const net = process.getBuiltinModule('child_process').spawnSync(process.execPath, ['-e',
        "const s=require('node:net').connect({host:'1.1.1.1',port:443}); s.on('connect',()=>process.exit(9)); s.on('error',()=>process.exit(0)); setTimeout(()=>process.exit(8),1000);"]);
      if (net.status !== 0) throw new Error('network denial was not observed: ' + net.status);
      const dns = process.getBuiltinModule('child_process').spawnSync(process.execPath, ['-e',
        "require('node:dns').lookup('example.com', e=>process.exit(e?0:9));"], {timeout: 1500});
      if (dns.status !== 0 && dns.error?.code !== 'ETIMEDOUT') throw new Error('DNS resolved externally');
      if (process.getuid() === 0) throw new Error('sandbox ran as root');
      const status = fs.readFileSync('/proc/self/status', 'utf8');
      if (!/^CapEff:\\s+0+$/m.test(status) || !/^NoNewPrivs:\\s+1$/m.test(status)) throw new Error('privilege limits missing');
      const limit = file => fs.readFileSync('/sys/fs/cgroup/' + file, 'utf8').trim();
      if (limit('memory.max') !== '134217728' || limit('memory.swap.max') !== '0') throw new Error('memory limits missing');
      if (limit('pids.max') !== '32' || limit('cpu.max') !== '100000 100000') throw new Error('process/CPU limits missing');
      ctx.report(ctx.nodes[0], 'isolated');
    }`,
      { secret },
      { timeoutMs: 10000 },
    );
    assert.deepEqual(JSON.parse(probe.run(batch)), [
      { rule: "check", node: 0, message: "isolated" },
    ]);
    assert.equal(readFileSync(secret, "utf8"), "host-secret-must-not-cross-the-boundary");
    assertNoContainers();
  } finally {
    delete process.env.VIZE_SANDBOX_HOST_SECRET;
    rmSync(root, { recursive: true, force: true });
  }
});

test("real sandbox kills infinite callbacks and oversized output, then confirms cleanup", () => {
  assert.throws(
    () => rule("() => { for (;;) {} }", null, { timeoutMs: 500 }).run(batch),
    /ETIMEDOUT/,
  );
  assertNoContainers();
  assert.throws(
    () =>
      rule("ctx => { ctx.report(ctx.nodes[0], 'x'.repeat(1024 * 1024)); }", null, {
        maxBytes: 65536,
      }).run(batch),
    /ENOBUFS|output limit/,
  );
  assertNoContainers();
  assert.throws(
    () =>
      rule("() => { process.stdout.write('x'.repeat(1024 * 1024)); }", null, {
        maxBytes: 65536,
      }).run(batch),
    /ENOBUFS|output limit/,
  );
  assertNoContainers();
  assert.throws(() => rule("() => hostClosure() ").run(batch), /hostClosure is not defined/);
  assertNoContainers();
  assert.throws(() => rule("async () => []").run(batch), /asynchronous/);
  assertNoContainers();
});

test("repeated real output overflow preserves its execution error across container auto-removal races", () => {
  for (let attempt = 0; attempt < 4; attempt++) {
    for (const callback of [
      "ctx => { ctx.report(ctx.nodes[0], 'x'.repeat(1024 * 1024)); }",
      "() => { process.stdout.write('x'.repeat(1024 * 1024)); }",
    ]) {
      assert.throws(
        () => rule(callback, null, { maxBytes: 65536 }).run(batch),
        (error: unknown) => {
          assert.ok(error instanceof SandboxRuntimeError);
          assert.equal(error.code, "execution_stopped");
          assert.match(error.message, /ENOBUFS|output limit/);
          return true;
        },
      );
      assertNoContainers();
    }
  }
});

test("stopped sandbox results retain private captured output and original cleanup semantics", () => {
  // An isolated Node process replaces the builtin before importing the runner.
  // An empty PATH also refuses any accidentally unmocked Docker invocation.
  const fixture = String.raw`
import assert from "node:assert/strict";
import childProcess from "node:child_process";
import { readFileSync } from "node:fs";
import { syncBuiltinESMExports } from "node:module";
import { inspect } from "node:util";
const input = JSON.parse(readFileSync(0, "utf8"));
const worker = readFileSync(new URL("./sandbox-worker.js", input.moduleUrl), "utf8");
process.env.SANDBOX_CUSTODY_ENV = "environment-must-not-be-copied";
let state;
childProcess.spawnSync = (command, args, options) => {
  assert.equal(command, "docker");
  state.calls.push({ args, options });
  if (args[0] === "image") return { status: 0, stdout: "a".repeat(64), stderr: "" };
  if (args[0] === "run") return state.result;
  if (args[0] === "rm") return { status: state.cleanupFails ? 1 : 0, stderr: "daemon reply" };
  assert.equal(args[0], "container");
  return { error: Object.assign(new Error("inspect unavailable"), { code: "EIO" }),
    status: null, stderr: "daemon unavailable" };
};
syncBuiltinESMExports();
const { createSandboxRunner, SandboxRuntimeError, SANDBOX_IMAGE } = await import(input.moduleUrl);
const definition = { name: "custody-proof", version: "1", configuration: "request-must-not-be-copied",
  rules: { check: "() => []" } };
const batch = '{"schema":1,"plugin":"custody-proof"}';
for (const mode of ["ETIMEDOUT", "ENOBUFS", "ENOENT", "cleanup", "success", "exit", "json"]) {
  try {
    const code = mode === "cleanup" ? "ETIMEDOUT" : mode;
    const stopped = ["ETIMEDOUT", "ENOBUFS", "ENOENT", "cleanup"].includes(mode);
    const cause = Object.assign(new Error("stopped docker"), { code });
    const stdout = mode === "ENOENT" ? null : "\nAVAILABLE_STDOUT 日本語😀\n ";
    const stderr = mode === "ENOENT" ? null : "\tAVAILABLE_STDERR 🎨\n ";
    state = { calls: [], cleanupFails: mode === "cleanup", result: stopped
      ? { error: cause, status: null, signal: mode === "ENOENT" ? null : "SIGKILL", stdout, stderr }
      : { status: mode === "exit" ? 2 : 0, signal: null,
        stdout: mode === "json" ? "not-json" : "[]", stderr: "child failed\n" } };
    const runner = createSandboxRunner(definition);
    let failure;
    try {
      assert.equal(runner.run(batch), "[]");
    } catch (error) { failure = error; }
    if (mode === "success") assert.equal(failure, undefined);
    else if (mode === "exit") {
      assert.ok(failure instanceof SandboxRuntimeError);
      assert.equal(failure.code, "execution_failed");
      assert.equal(failure.execution, undefined);
    } else if (mode === "json") assert.ok(failure instanceof SyntaxError);
    else {
      assert.ok(failure instanceof SandboxRuntimeError);
      if (state.cleanupFails) {
        assert.equal(failure.code, "cleanup_failed");
        assert.ok(failure.cause instanceof AggregateError);
        assert.equal(failure.cause.errors.length, 2);
        failure = failure.cause.errors[0];
      }
      assert.equal(failure.code, "execution_stopped");
      assert.equal(failure.message, "sandbox execution stopped: " + code);
      assert.equal(failure.cause, cause);
      const observation = failure.execution;
      assert.deepEqual(Object.keys(observation),
        ["phase", "elapsedMs", "status", "signal", "stdout", "stderr"]);
      assert.equal(observation.phase, "run");
      assert.ok(Number.isFinite(observation.elapsedMs) && observation.elapsedMs >= 0);
      assert.equal(observation.status, state.result.status);
      assert.equal(observation.signal, state.result.signal);
      assert.equal(observation.stdout, stdout);
      assert.equal(observation.stderr, stderr);
      assert.ok(Object.isFrozen(observation));
      assert.throws(() => { observation.stderr = "changed"; }, TypeError);
      assert.throws(() => { failure.execution = {}; }, TypeError);
      state.result.stdout = "changed after capture";
      assert.equal(observation.stdout, stdout, "snapshot must not alias the mutable result object");
      assert.deepEqual(Object.getOwnPropertyDescriptor(failure, "execution"),
        { value: observation, writable: false, enumerable: false, configurable: false });
      assert.ok(!Object.keys(failure).includes("execution"));
      assert.doesNotMatch(JSON.stringify(failure) + inspect(failure, { depth: null }),
        /AVAILABLE_STDOUT|AVAILABLE_STDERR|request-must-not-be-copied|environment-must-not-be-copied/);
      assert.doesNotMatch(JSON.stringify(observation),
        /request-must-not-be-copied|environment-must-not-be-copied/);
    }
    const [prerequisite, run, remove, confirm] = state.calls;
    assert.equal(state.calls.length, state.cleanupFails ? 4 : 3, "one execution, never a retry");
    assert.deepEqual(prerequisite.args, ["image", "inspect", "--format={{.Id}}", SANDBOX_IMAGE]);
    assert.deepEqual(prerequisite.options, { encoding: "utf8", timeout: 5000, maxBuffer: 16384 });
    const name = run.args[4];
    assert.match(name, /^vize-plugin-[0-9a-f-]{36}$/);
    assert.deepEqual(run.args, ["run", "--rm", "--pull=never", "--name", name,
      "--label", "vize.plugin-sandbox=1", "--label", "vize.plugin-sandbox.host=" + process.pid,
      "--network=none", "--read-only", "--user=65534:65534", "--cap-drop=ALL",
      "--security-opt=no-new-privileges:true", "--pids-limit=32", "--memory=128m", "--memory-swap=128m",
      "--cpus=1", "--ulimit=nofile=64:64", "--ipc=none", "--log-driver=none", "--workdir=/",
      "--entrypoint=node", "-i", SANDBOX_IMAGE, "--max-old-space-size=64", "--input-type=module", "--eval", worker]);
    const { input: request, ...options } = run.options;
    assert.deepEqual(options, { encoding: "utf8", timeout: 5000, maxBuffer: 1048576,
      killSignal: "SIGKILL", windowsHide: true });
    const payload = JSON.parse(request);
    assert.deepEqual(payload.definition, { ...definition, family: "rule" });
    assert.equal(payload.batchJson, batch);
    assert.match(payload.sdk, /^data:text\/javascript;base64,/);
    assert.deepEqual(remove.args, ["rm", "--force", name]);
    assert.equal(remove.options.encoding, "utf8");
    assert.equal(remove.options.maxBuffer, 16384);
    assert.equal(remove.options.killSignal, "SIGKILL");
    assert.ok(remove.options.timeout >= 1 && remove.options.timeout <= 5000);
    if (confirm) assert.deepEqual(confirm.args, ["container", "inspect", "--format={{.Id}}", name]);
  } catch (error) {
    throw new Error("sandbox custody mode " + mode + " failed", { cause: error });
  }
}
process.stdout.write("seven sandbox custody controls passed\n");
`;
  const result = spawnSync(process.execPath, ["--input-type=module", "--eval", fixture], {
    input: JSON.stringify({
      moduleUrl: new URL("../../npm/plugin-sdk/sandbox.js", import.meta.url).href,
    }),
    env: { ...process.env, PATH: "" },
    encoding: "utf8",
    timeout: 10000,
    maxBuffer: 1048576,
  });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, "seven sandbox custody controls passed\n");
});
