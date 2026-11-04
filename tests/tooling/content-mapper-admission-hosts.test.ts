import assert from "node:assert/strict";
import test from "node:test";
import {
  fixture,
  frames,
  qualifyWire,
  diagnosticPacket,
  hostConfig,
  semanticPacket,
} from "../../tools/support/compat/content-mapper/admission-hosts-core.ts";
const encode = (packet: unknown) => {
  const body = Buffer.from(JSON.stringify(packet));
  return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
};
function pair() {
  const source = fixture(false, 65_536),
    expected = new Map<string, string>();
  const input: Record<string, unknown>[] = [
      { jsonrpc: "2.0", id: 1, method: "initialize", params: { positionEncodings: ["utf-8"] } },
      {
        jsonrpc: "2.0",
        id: 2,
        method: "openProject",
        params: { projectHandle: "owned", configFileName: "/project/tsconfig.json" },
      },
    ],
    output: Record<string, unknown>[] = [
      { jsonrpc: "2.0", id: 1, result: { positionEncoding: "utf-8" } },
      { jsonrpc: "2.0", id: 2, result: { configIdentity: "fixture" } },
    ];
  for (let index = 0; index < 8; index++) {
    const fileName = `/project/src/File${index}.vue`;
    expected.set(fileName, source);
    input.push({
      jsonrpc: "2.0",
      id: index + 3,
      method: "transform",
      params: { fileName, content: source, projectHandle: "owned" },
    });
    output.push({
      jsonrpc: "2.0",
      id: index + 3,
      result: {
        text: " ".repeat(70_000),
        mappings: [[0, 1, 0, 1, 0, 0]],
        diagnostics: [],
        diagnosticDirectives: { ignored: [] },
      },
    });
  }
  return {
    input: Buffer.concat(input.map(encode)),
    output: Buffer.concat(output.map(encode)),
    expected,
    responses: output,
    requests: input,
  };
}
void test("complete Unicode frames retain every ordered packet and authored byte", () => {
  const value = pair();
  assert.equal(qualifyWire(value.input, value.output, value.expected, 65_536).transforms.length, 8);
  assert.deepEqual(
    frames(value.output).map((row) => row.packet),
    value.responses,
  );
});
void test("partial headers, bodies, invalid UTF8 and duplicate headers are refused", () => {
  for (const bytes of [
    Buffer.from("Content-Length: 2"),
    Buffer.from("Content-Length: 2\r\n\r\n{"),
    Buffer.from("Content-Length: 2\r\nContent-Length: 2\r\n\r\n{}"),
    Buffer.from([...Buffer.from("Content-Length: 1\r\n\r\n"), 255]),
  ])
    assert.throws(() => frames(bytes));
});
void test("reordered, missing, duplicate and RPC-error responses never acquire liveness credit", () => {
  const value = pair();
  for (const rows of [
    [...value.responses].reverse(),
    value.responses.slice(1),
    [...value.responses.slice(0, 7), value.responses[0]],
    [{ jsonrpc: "2.0", id: 1, error: {} }, ...value.responses.slice(1)],
  ])
    assert.throws(() =>
      qualifyWire(value.input, Buffer.concat(rows.map(encode)), value.expected, 65_536),
    );
});
void test("changed authored source, missing mappings and small responses are refused", () => {
  const value = pair();
  assert.throws(() =>
    qualifyWire(
      value.input,
      value.output,
      new Map([...value.expected].map(([name]) => [name, "changed"])),
      65_536,
    ),
  );
  for (const result of [
    { text: "small", mappings: [[0, 1, 0, 1, 0, 0]], diagnostics: [] },
    { text: " ".repeat(70_000), mappings: [], diagnostics: [] },
  ])
    assert.throws(() =>
      qualifyWire(
        value.input,
        Buffer.concat(
          value.responses.map((row, index) =>
            encode(index < 2 ? row : { jsonrpc: "2.0", id: index + 1, result }),
          ),
        ),
        value.expected,
        65_536,
      ),
    );
});
void test("normal positive and complete authored negative diagnostic packets are distinct", () => {
  assert.deepEqual(diagnosticPacket("", "", "/project", []), []);
  const text =
    Array.from(
      { length: 8 },
      (_, index) =>
        `src/File${index}.vue(2,7): error TS2322: Type 'string' is not assignable to type 'number'.`,
    ).join("\n") + "\n";
  assert.equal(diagnosticPacket(text, "", "/project", [0, 1, 2, 3, 4, 5, 6, 7]).length, 8);
  assert.equal(semanticPacket(1, text, "", "/project", [0, 1, 2, 3, 4, 5, 6, 7]).length, 8);
  assert.throws(
    () => semanticPacket(2, text, "", "/project", [0, 1, 2, 3, 4, 5, 6, 7]),
    /semantic status/u,
  );
  assert.throws(() => diagnosticPacket("", "", "/project", [0, 1, 2, 3, 4, 5, 6, 7]));
  assert.throws(() =>
    diagnosticPacket(text.replace("(2,7)", "(2,8)"), "", "/project", [0, 1, 2, 3, 4, 5, 6, 7]),
  );
  assert.throws(() => diagnosticPacket(text, "timeout", "/project", [0, 1, 2, 3, 4, 5, 6, 7]));
});

void test("host config follows the existing public root mapper schema", async () => {
  const { readFileSync } = await import("node:fs");
  const suite = JSON.parse(
    readFileSync(
      new URL("../_fixtures/content-mapper/escaton-component-props.json", import.meta.url),
      "utf8",
    ),
  );
  const oracle = JSON.parse(suite.originals[0].files["tsconfig.json"]);
  const config = hostConfig({ strict: true, skipLibCheck: false });
  assert.deepEqual(config.contentMappers, oracle.contentMappers);
  assert.equal(Object.hasOwn(config.compilerOptions, "contentMappers"), false);
  assert.deepEqual(config.contentMappers[0].extensions, [".vue"]);
  assert.throws(
    () => hostConfig({ contentMappers: [{ extension: ".vue", package: "vize" }] }),
    /project root/u,
  );
});

void test("host raw descriptors and argv are durable before a controlled Node child starts", async () => {
  const { mkdtempSync, readFileSync, rmSync } = await import("node:fs");
  const { tmpdir } = await import("node:os");
  const { join } = await import("node:path");
  const { run } =
    await import("../../tools/support/compat/content-mapper/admission-hosts-process.ts");
  const root = mkdtempSync(join(tmpdir(), "mapper-admission-fds-"));
  const previous = process.env.ADMISSION_DEADLINE_MS;
  try {
    process.env.ADMISSION_DEADLINE_MS = String(Date.now() + 10_000);
    const output = join(root, "process");
    const code = `const fs=require('node:fs');const p=${JSON.stringify(output)};for(const name of ['stdout.raw','stderr.raw','spawn.json','baseline.json'])if(!fs.existsSync(p+'/'+name))process.exit(9);process.stdout.write('full output');process.exit(2);`;
    const result = await run(output, [process.execPath, "-e", code], root);
    assert.equal(result.status, 2);
    assert.equal(result.text, "full output");
    assert.equal(result.timedOut, false);
    assert.equal(JSON.parse(readFileSync(join(output, "spawn.json"), "utf8")).argv[2], code);
  } finally {
    if (previous === undefined) delete process.env.ADMISSION_DEADLINE_MS;
    else process.env.ADMISSION_DEADLINE_MS = previous;
    if (!process.env.ADMISSION_RETAIN_CONTROLS) rmSync(root, { recursive: true });
  }
});

void test("only exact pre-spawn protected identities can be excluded from ownership errors", async () => {
  const { observationDisposition } =
    await import("../../tools/support/compat/content-mapper/admission-hosts-owned.ts");
  const identity = {
    pid: 42,
    ppid: 1,
    group: 1,
    startTicks: "100",
    state: "S",
    ownerMarker: false,
  };
  const baseline = new Set(["42:100"]),
    owned = new Set<string>();
  assert.deepEqual(observationDisposition(identity, baseline, owned, "EACCES"), {
    scope: "preexisting-unrelated",
    excluded: true,
  });
  assert.equal(
    observationDisposition({ ...identity, startTicks: "101" }, baseline, owned, "EACCES").excluded,
    false,
    "PID reuse must not inherit baseline authority",
  );
  assert.equal(
    observationDisposition({ ...identity, pid: 43 }, baseline, owned, "EACCES").excluded,
    false,
    "post-spawn identity is unknown",
  );
  assert.deepEqual(observationDisposition(identity, baseline, new Set(["42:100"]), "EACCES"), {
    scope: "known-owned",
    excluded: false,
  });
  assert.deepEqual(
    observationDisposition({ ...identity, ownerMarker: true }, baseline, owned, "EACCES"),
    { scope: "known-owned", excluded: false },
  );
  assert.equal(
    observationDisposition(undefined, baseline, owned, "EACCES").excluded,
    false,
    "missing kernel identity cannot establish preexistence",
  );
  assert.equal(
    observationDisposition(identity, baseline, owned, "EIO").excluded,
    false,
    "non-permission observation failures remain fatal",
  );
});

void test("a process deadline retains partial raw output and fails outside semantic diagnostics", async () => {
  const { mkdtempSync, readFileSync, rmSync } = await import("node:fs");
  const { tmpdir } = await import("node:os");
  const { join } = await import("node:path");
  const { run } =
    await import("../../tools/support/compat/content-mapper/admission-hosts-process.ts");
  const root = mkdtempSync(join(tmpdir(), "mapper-admission-deadline-"));
  const previous = process.env.ADMISSION_DEADLINE_MS;
  try {
    process.env.ADMISSION_DEADLINE_MS = String(Date.now() + 200);
    const output = join(root, "process");
    await assert.rejects(
      run(
        output,
        [process.execPath, "-e", "process.stdout.write('partial');setInterval(()=>{},1000);"],
        root,
      ),
      /timeout\/crash\/survivor/u,
    );
    const result = JSON.parse(readFileSync(join(output, "result.json"), "utf8"));
    assert.equal(result.timedOut, true);
    assert.equal(result.signal, "SIGKILL");
    assert.equal(readFileSync(join(output, "stdout.raw"), "utf8"), "partial");
  } finally {
    if (previous === undefined) delete process.env.ADMISSION_DEADLINE_MS;
    else process.env.ADMISSION_DEADLINE_MS = previous;
    if (!process.env.ADMISSION_RETAIN_CONTROLS) rmSync(root, { recursive: true });
  }
});

void test("request version is optional and opaque extra fields remain in complete packets", () => {
  const value = pair();
  const requests = value.requests.map(({ jsonrpc: _version, ...row }) => ({
    ...row,
    opaque: { retained: true },
  }));
  const output = value.responses.map((row) => ({ ...row, extra: { untouched: [1, 2] } }));
  const captured = qualifyWire(
    Buffer.concat(requests.map(encode)),
    Buffer.concat(output.map(encode)),
    value.expected,
    65_536,
  );
  assert.deepEqual(
    captured.requests.map((row) => row.packet),
    requests,
  );
  assert.deepEqual(
    captured.responses.map((row) => row.packet),
    output,
  );
  assert.throws(() =>
    qualifyWire(
      Buffer.concat(requests.map((row) => encode({ ...row, jsonrpc: "wrong" }))),
      value.output,
      value.expected,
      65_536,
    ),
  );
});

void test("successful transforms require initialized UTF8 and a matching opened project", () => {
  const value = pair();
  const check = (requests: unknown[], responses: unknown[]) =>
    qualifyWire(
      Buffer.concat(requests.map(encode)),
      Buffer.concat(responses.map(encode)),
      value.expected,
      65_536,
    );
  assert.throws(() => check(value.requests.slice(2), value.responses.slice(2)), /initialize/u);
  assert.throws(
    () =>
      check(
        [value.requests[0], ...value.requests.slice(2)],
        [value.responses[0], ...value.responses.slice(2)],
      ),
    /not opened/u,
  );
  const wrong = value.requests.map((row, index) =>
    index < 2
      ? row
      : { ...row, params: { ...(row.params as object), projectHandle: "never-opened" } },
  );
  assert.throws(() => check(wrong, value.responses), /not opened/u);
  assert.throws(
    () =>
      check(value.requests, [
        { ...value.responses[0], result: { positionEncoding: "utf-16" } },
        ...value.responses.slice(1),
      ]),
    /UTF8/u,
  );
  assert.equal(
    check(value.requests, value.responses).transforms.length,
    8,
    "host closeProject is not required for an otherwise completed transcript",
  );
  const close = { id: 11, method: "closeProject", params: { projectHandle: "owned" } };
  const closed = { jsonrpc: "2.0", id: 11, result: null };
  assert.equal(
    check([...value.requests, close], [...value.responses, closed]).transforms.length,
    8,
  );
  assert.equal(
    check(
      [...value.requests, { ...close, params: { projectHandle: "unknown" } }],
      [...value.responses, closed],
    ).transforms.length,
    8,
    "closing an unknown handle is idempotent in the mapper API",
  );
  assert.throws(
    () =>
      check(
        [...value.requests, close, { ...value.requests[2], id: 12 }],
        [...value.responses, closed, { ...value.responses[2], id: 12 }],
      ),
    /not opened/u,
  );
});

void test("a detached immediate orphan remains owned after its parent exits", async () => {
  const { existsSync, mkdtempSync, readFileSync, rmSync } = await import("node:fs");
  const { tmpdir } = await import("node:os");
  const { join } = await import("node:path");
  const { run } =
    await import("../../tools/support/compat/content-mapper/admission-hosts-process.ts");
  const root = mkdtempSync(join(tmpdir(), "mapper-admission-orphan-"));
  const previous = process.env.ADMISSION_DEADLINE_MS;
  let escaped: number | undefined;
  try {
    process.env.ADMISSION_DEADLINE_MS = String(Date.now() + 15_000);
    const marker = join(root, "escaped.json"),
      output = join(root, "process");
    const descendant = `require('node:fs').writeFileSync(${JSON.stringify(marker)},JSON.stringify({pid:process.pid}));setInterval(()=>{},1000);`;
    const parent = `const child=require('node:child_process').spawn(process.execPath,['-e',${JSON.stringify(descendant)}],{detached:true,stdio:'ignore'});child.unref();`;
    await assert.rejects(
      run(output, [process.execPath, "-e", parent], root),
      /timeout\/crash\/survivor/u,
    );
    escaped = JSON.parse(readFileSync(marker, "utf8")).pid;
    const result = JSON.parse(readFileSync(join(output, "result.json"), "utf8"));
    assert.equal(result.status, 0);
    assert.equal(result.survivors, true);
    assert.ok(
      result.observedProcesses.some(
        (row: { pid: number; startTicks: string; ownerMarker: boolean }) =>
          row.pid === escaped && row.startTicks && row.ownerMarker,
      ),
    );
    assert.ok(result.cleanup.some((row: { pid: number }) => row.pid === escaped));
  } finally {
    if (existsSync(join(root, "process/spawn.json"))) {
      const { OwnedProcesses } =
        await import("../../tools/support/compat/content-mapper/admission-hosts-owned.ts");
      const { ownerNonce } = JSON.parse(readFileSync(join(root, "process/spawn.json"), "utf8"));
      new OwnedProcesses(undefined, ownerNonce).killAll();
    }
    if (previous === undefined) delete process.env.ADMISSION_DEADLINE_MS;
    else process.env.ADMISSION_DEADLINE_MS = previous;
    if (!process.env.ADMISSION_RETAIN_CONTROLS) rmSync(root, { recursive: true });
  }
});

void test("known child cleanup survives a failed whole-process inventory", async () => {
  const { spawn } = await import("node:child_process");
  const { randomUUID } = await import("node:crypto");
  const { OwnedProcesses, ownerVariable } =
    await import("../../tools/support/compat/content-mapper/admission-hosts-owned.ts");
  const nonce = randomUUID();
  const child = spawn(process.execPath, ["-e", "setInterval(()=>{},1000)"], {
    env: { ...process.env, [ownerVariable]: nonce },
    stdio: "ignore",
  });
  const closed = new Promise((resolve) =>
    child.once("close", (status, signal) => resolve({ status, signal })),
  );
  const owned = new OwnedProcesses(child.pid, nonce);
  const original = owned.scan.bind(owned);
  let deadline: ReturnType<typeof setTimeout> | undefined;
  try {
    owned.scan = () => {
      owned.error = "controlled process inventory unavailable";
      return [];
    };
    const cleanup = owned.killAll();
    assert.ok(
      cleanup.some((row) => row.pid === child.pid && row.startTicks),
      "retained owned identity was not terminated",
    );
    const result = await Promise.race([
      closed,
      new Promise((_resolve, reject) => {
        deadline = setTimeout(() => reject(new Error("known child remained alive")), 2000);
      }),
    ]);
    assert.deepEqual(result, { status: null, signal: "SIGKILL" });
    assert.equal(
      owned.error,
      "controlled process inventory unavailable",
      "observation errors must not acquire qualification",
    );
  } finally {
    if (deadline) clearTimeout(deadline);
    owned.scan = original;
    owned.killAll();
  }
});

void test("missing child close is bounded and keeps an incomplete raw result", async () => {
  const { existsSync, mkdtempSync, readFileSync, rmSync } = await import("node:fs");
  const { tmpdir } = await import("node:os");
  const { join } = await import("node:path");
  const { OwnedProcesses } =
    await import("../../tools/support/compat/content-mapper/admission-hosts-owned.ts");
  const { run } =
    await import("../../tools/support/compat/content-mapper/admission-hosts-process.ts");
  const root = mkdtempSync(join(tmpdir(), "mapper-admission-missing-close-"));
  const output = join(root, "process");
  const previous = process.env.ADMISSION_DEADLINE_MS;
  const original = Object.getOwnPropertyDescriptor(OwnedProcesses.prototype, "killAll")!;
  try {
    process.env.ADMISSION_DEADLINE_MS = String(Date.now() + 500);
    OwnedProcesses.prototype.killAll = () => [];
    await assert.rejects(
      run(
        output,
        [process.execPath, "-e", "process.stdout.write('partial');setInterval(()=>{},1000)"],
        root,
      ),
      /timeout\/crash\/survivor/u,
    );
    const result = JSON.parse(readFileSync(join(output, "result.json"), "utf8"));
    assert.equal(result.closeObserved, false);
    assert.equal(result.timedOut, true);
    assert.equal(result.status, null);
    assert.equal(result.signal, null, "an unobserved exit must not invent a signal");
    assert.equal(readFileSync(join(output, "stdout.raw"), "utf8"), "partial");
  } finally {
    Object.defineProperty(OwnedProcesses.prototype, "killAll", original);
    if (existsSync(join(output, "spawn.json"))) {
      const { ownerNonce } = JSON.parse(readFileSync(join(output, "spawn.json"), "utf8"));
      const pid = existsSync(join(output, "result.json"))
        ? JSON.parse(readFileSync(join(output, "result.json"), "utf8")).pid
        : undefined;
      const owned = new OwnedProcesses(pid, ownerNonce);
      owned.killAll();
      const until = Date.now() + 2000;
      while (owned.scan().length && Date.now() < until)
        await new Promise((done) => setTimeout(done, 20));
      assert.equal(owned.scan().length, 0, "owned control child remains after cleanup");
    }
    if (previous === undefined) delete process.env.ADMISSION_DEADLINE_MS;
    else process.env.ADMISSION_DEADLINE_MS = previous;
    if (!process.env.ADMISSION_RETAIN_CONTROLS) rmSync(root, { recursive: true });
  }
});
