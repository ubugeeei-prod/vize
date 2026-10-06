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
