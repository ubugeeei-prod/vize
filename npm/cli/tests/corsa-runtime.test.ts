import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "vite-plus/test";

const environmentNames = [
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
] as const;

function fixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-cli-user-runtime-"));
  const packageRoot = path.join(root, "node_modules", "vize");
  const platformPackage = "@typescript/typescript-" + process.platform + "-" + process.arch;
  const stalePackage = path.join(packageRoot, "node_modules", platformPackage);
  fs.mkdirSync(path.join(stalePackage, "lib"), { recursive: true });
  fs.writeFileSync(
    path.join(stalePackage, "package.json"),
    JSON.stringify({
      name: platformPackage,
      version: "7.0.2",
    }),
  );
  fs.writeFileSync(
    path.join(stalePackage, "lib", process.platform === "win32" ? "tsc.exe" : "tsc"),
    "stale bundled runtime must not be selected",
  );
  fs.writeFileSync(
    path.join(packageRoot, "package.json"),
    JSON.stringify({
      name: "vize",
      version: "0.0.0",
      type: "module",
      optionalDependencies: { [platformPackage]: "7.0.2" },
    }),
  );
  const stub = path.join(packageRoot, "node_modules", "@vizejs", "native");
  fs.mkdirSync(stub, { recursive: true });
  fs.writeFileSync(
    path.join(stub, "package.json"),
    JSON.stringify({ name: "@vizejs/native", version: "0.0.0", main: "index.cjs" }),
  );
  fs.writeFileSync(
    path.join(stub, "index.cjs"),
    "exports.runCli = (args) => process.stdout.write(JSON.stringify({args, environment: Object.fromEntries(" +
      JSON.stringify(environmentNames) +
      ".map(name => [name, process.env[name] ?? null]))}));\n",
  );
  const installed = path.join(packageRoot, "dist", "cli.mjs");
  fs.cpSync(fileURLToPath(new URL("../dist", import.meta.url)), path.dirname(installed), {
    recursive: true,
  });
  const run = (args: string[], override: Record<string, string> = {}) => {
    const environment = { ...process.env };
    for (const name of environmentNames) delete environment[name];
    Object.assign(environment, override);
    // The launcher answers the mapper's `initialize` itself (#12530).
    const initialize = JSON.stringify({
      id: 1,
      method: "initialize",
      params: { positionEncodings: ["utf-8"] },
    });
    const result = spawnSync(process.execPath, [installed, ...args], {
      cwd: root,
      env: environment,
      encoding: "utf8",
      input:
        args[0] === "content-mapper"
          ? `Content-Length: ${initialize.length}\r\n\r\n${initialize}`
          : "",
    });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stderr, "");
    const stdout =
      args[0] === "content-mapper"
        ? result.stdout.replace(/^Content-Length: \d+\r\n\r\n\{[^\n]*?\}\}/, "")
        : result.stdout;
    return JSON.parse(stdout) as {
      args: string[];
      environment: Record<string, string | null>;
    };
  };
  return { run, remove: () => fs.rmSync(root, { force: true, recursive: true }) };
}

test("CLI, LSP, mapper and syntax-only commands delegate without selecting a stale bundled runtime", () => {
  const f = fixture();
  try {
    for (const command of ["check", "lsp", "content-mapper", "lint", "fmt", "--version"]) {
      assert.deepEqual(f.run([command]), {
        args: command === "content-mapper" ? [command, "--initialized"] : [command],
        environment: Object.fromEntries(environmentNames.map((name) => [name, null])),
      });
    }
  } finally {
    f.remove();
  }
});

test("every explicit runtime environment and CLI path reaches the shared native resolver unchanged", () => {
  const f = fixture();
  try {
    for (const name of environmentNames) {
      const value = "/project/日本語/native runtime";
      assert.deepEqual(f.run(["check"], { [name]: value }), {
        args: ["check"],
        environment: Object.fromEntries(
          environmentNames.map((key) => [key, key === name ? value : null]),
        ),
      });
    }
    const args = ["check", "--corsa-path", "/project/config/native runtime"];
    assert.deepEqual(f.run(args), {
      args,
      environment: Object.fromEntries(environmentNames.map((name) => [name, null])),
    });
  } finally {
    f.remove();
  }
});

test("published CLI manifest does not automatically install a native TypeScript runtime", () => {
  const manifest = JSON.parse(
    fs.readFileSync(fileURLToPath(new URL("../package.json", import.meta.url)), "utf8"),
  ) as {
    optionalDependencies?: Record<string, string>;
  };
  assert.deepEqual(
    Object.keys(manifest.optionalDependencies ?? {}).filter(
      (name) => name === "typescript" || name.startsWith("@typescript/"),
    ),
    [],
  );
});

// #12530: the TypeScript host waits a fixed 5 s for `initialize`, so the
// mapper launcher answers it before loading the native addon.

const HOST_INITIALIZE_BUDGET_MS = 5_000;
const ADDON_LOAD_MS = 5_500;

// A stand-in addon whose load outlasts the host budget. Once loaded, its
// `runCli` answers one frame from the stream the launcher left behind.
const stubAddon = `const fs = require("node:fs");
Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, Number(process.env.STUB_LOAD_MS));
fs.writeFileSync(process.env.STUB_LOADED, "loaded");
function read(length) {
  const buffer = Buffer.alloc(length);
  for (let offset = 0; offset < length; ) offset += fs.readSync(0, buffer, offset, length - offset, null);
  return buffer;
}
exports.runCli = (args) => {
  let header = "";
  while (!header.endsWith("\\r\\n\\r\\n")) header += read(1).toString("latin1");
  const request = JSON.parse(read(Number(/Content-Length: (\\d+)/.exec(header)[1])).toString());
  const body = JSON.stringify({ id: request.id, result: { args, method: request.method } });
  fs.writeSync(1, \`Content-Length: \${Buffer.byteLength(body)}\\r\\n\\r\\n\${body}\`);
};
`;

function install() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-mapper-launch-"));
  const packageRoot = path.join(root, "node_modules", "vize");
  const stub = path.join(packageRoot, "node_modules", "@vizejs", "native");
  fs.mkdirSync(stub, { recursive: true });
  const manifest = (name: string, extra = {}) =>
    JSON.stringify({ name, version: "0.0.0", ...extra });
  fs.writeFileSync(path.join(packageRoot, "package.json"), manifest("vize", { type: "module" }));
  fs.writeFileSync(
    path.join(stub, "package.json"),
    manifest("@vizejs/native", { main: "index.cjs" }),
  );
  fs.writeFileSync(path.join(stub, "index.cjs"), stubAddon);
  fs.cpSync(fileURLToPath(new URL("../dist", import.meta.url)), path.join(packageRoot, "dist"), {
    recursive: true,
  });
  return {
    root,
    cli: path.join(packageRoot, "dist", "cli.mjs"),
    loaded: path.join(root, "loaded"),
  };
}

function frame(value: unknown): string {
  const body = JSON.stringify(value);
  return `Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`;
}

function mapper(fixture: ReturnType<typeof install>) {
  const child = spawn(process.execPath, [fixture.cli, "content-mapper"], {
    cwd: fixture.root,
    env: { ...process.env, STUB_LOAD_MS: String(ADDON_LOAD_MS), STUB_LOADED: fixture.loaded },
  });
  let stdout = "";
  let stderr = "";
  const waiters: (() => void)[] = [];
  child.stdout.on("data", (chunk: Buffer) => {
    stdout += chunk.toString();
    for (const wake of waiters.splice(0)) wake();
  });
  child.stderr.on("data", (chunk: Buffer) => (stderr += chunk.toString()));
  const exited = new Promise<number | null>((resolve) => child.on("exit", resolve));
  /** The next whole reply frame, parsed. */
  async function reply(): Promise<Record<string, unknown>> {
    for (;;) {
      const header = /^Content-Length: (\d+)\r\n\r\n/.exec(stdout);
      if (header && Buffer.byteLength(stdout) >= header[0].length + Number(header[1])) {
        const body = stdout.slice(header[0].length, header[0].length + Number(header[1]));
        stdout = stdout.slice(header[0].length + body.length);
        return JSON.parse(body) as Record<string, unknown>;
      }
      await new Promise<void>((resolve) => waiters.push(resolve));
    }
  }
  return { child, reply, exited, stderr: () => stderr };
}

test(
  "initialize is answered before an addon load that outlasts the host budget, which then serves the stream",
  { timeout: 30_000 },
  async () => {
    const fixture = install();
    try {
      const run = mapper(fixture);
      const started = performance.now();
      run.child.stdin.write(
        frame({
          jsonrpc: "2.0",
          id: 1,
          method: "initialize",
          params: { positionEncodings: ["utf-16", "utf-8"] },
        }),
      );
      assert.deepEqual(await run.reply(), {
        id: 1,
        jsonrpc: "2.0",
        result: { diagnosticSource: "vize", positionEncoding: "utf-8" },
      });
      const answered = performance.now() - started;
      // The reply precedes the load, so it never spends the load's time.
      assert.equal(fs.existsSync(fixture.loaded), false);
      assert.ok(answered < HOST_INITIALIZE_BUDGET_MS / 2, `initialize took ${answered} ms`);
      run.child.stdin.write(
        frame({ jsonrpc: "2.0", id: 2, method: "openProject", params: { projectHandle: "p" } }),
      );
      assert.deepEqual(await run.reply(), {
        id: 2,
        result: { args: ["content-mapper", "--initialized"], method: "openProject" },
      });
      assert.ok(performance.now() - started >= ADDON_LOAD_MS);
      run.child.stdin.end();
      assert.equal(await run.exited, 0, run.stderr());
    } finally {
      fs.rmSync(fixture.root, { force: true, recursive: true });
    }
  },
);

test("an uninitialized stream gets the native replies and never loads the addon", async () => {
  const fixture = install();
  try {
    const run = mapper(fixture);
    run.child.stdin.write(frame({ jsonrpc: "2.0", id: 1, method: "transform", params: {} }));
    run.child.stdin.write("Content-Length: 5\r\n\r\n{nope");
    run.child.stdin.write(frame({ jsonrpc: "2.0", id: 2, method: "shutdown" }));
    run.child.stdin.write(
      frame({
        jsonrpc: "2.0",
        id: 3,
        method: "initialize",
        params: { positionEncodings: ["utf-16"] },
      }),
    );
    run.child.stdin.write(frame({ jsonrpc: "2.0", id: 4, method: "initialize", params: {} }));
    const error = async () => (await run.reply()).error as { code: number; message: string };
    assert.deepEqual(await error(), { code: -32002, message: "Content mapper is not initialized" });
    assert.equal((await error()).code, -32700);
    assert.deepEqual(await error(), { code: -32601, message: "Method not found: shutdown" });
    assert.deepEqual(await error(), {
      code: -32602,
      message: "Vize requires the utf-8 position encoding",
    });
    assert.deepEqual(await error(), {
      code: -32602,
      message: "Invalid initialize params: missing field `positionEncodings`",
    });
    run.child.stdin.end();
    assert.equal(await run.exited, 0, run.stderr());
    assert.equal(fs.existsSync(fixture.loaded), false);
  } finally {
    fs.rmSync(fixture.root, { force: true, recursive: true });
  }
});

test("a broken frame fails like the native server", async () => {
  const fixture = install();
  try {
    const run = mapper(fixture);
    run.child.stdin.end("X-Other: 1\r\n\r\n");
    assert.equal(await run.exited, 1);
    assert.equal(run.stderr(), "vize content-mapper: missing Content-Length header\n");
    assert.equal(fs.existsSync(fixture.loaded), false);
  } finally {
    fs.rmSync(fixture.root, { force: true, recursive: true });
  }
});
