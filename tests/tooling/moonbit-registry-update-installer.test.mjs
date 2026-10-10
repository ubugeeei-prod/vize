import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { test } from "node:test";

const actionDir = new URL("../../.github/actions/setup-moonbit/", import.meta.url);
const source = fs.readFileSync(new URL("install-moonbit.mjs", actionDir), "utf8");
const version = fs.readFileSync(new URL("../../../.moonbit-version", actionDir), "utf8").trim();
const originalUnixDigest = "46495f8cdc0050f79b6cb195d66478d101cb3601d68506568fbe377fcdf2a9fe";
const transport =
  "Error: update failed\n\nCaused by:\n  0: failed to clone registry index\n  1: non-zero exit code: exit status: 128\n     git stderr:\n     fatal: unable to access 'https://mooncakes.io/git/index/': The requested URL returned error: 504\n";
const writeAll =
  "function writeAll(fd, bytes) { for (let offset = 0; offset < bytes.length;) offset += fs.writeSync(fd, bytes, offset, bytes.length - offset); }";

function command(file, body) {
  fs.writeFileSync(file, `#!/usr/bin/env node\nimport fs from "node:fs";\n${writeAll}\n${body}\n`);
  fs.chmodSync(file, 0o755);
}

function rawStderr(bytes) {
  return Buffer.from(
    bytes
      .toString("latin1")
      .replace(
        /^MoonBit registry update transport failed \(attempt [12]\/3, exit \d+\); retrying in [12]000ms\n/gm,
        "",
      ),
    "latin1",
  );
}

async function child(file, env) {
  return spawnSync(process.execPath, [file], { env, maxBuffer: 16 * 1024 * 1024 });
}

async function installer({ warm = false, tamper = false, wrongVersion = false } = {}) {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "moon install 日本-"));
  try {
    const tree = path.join(temp, "tree");
    const location = path.join(tree, ".github", "actions", "setup-moonbit", "install-moonbit.mjs");
    const runner = path.join(temp, "runner temp");
    const moonHome = path.join(runner, "moonbit");
    const bin = path.join(temp, "controlled commands");
    const log = path.join(temp, "calls.jsonl");
    const githubPath = path.join(temp, "github-path");
    const githubEnv = path.join(temp, "github-env");
    const fixture = Buffer.from("controlled installer payload\n");
    const digest = createHash("sha256").update(fixture).digest("hex");
    assert.equal(source.split(originalUnixDigest).length, 2);
    fs.mkdirSync(path.dirname(location), { recursive: true });
    fs.mkdirSync(runner, { recursive: true });
    fs.mkdirSync(bin, { recursive: true });
    // Only the expected Unix download digest is injected into this temporary
    // copy. The real hash verifier and every installer stage execute unchanged.
    fs.writeFileSync(location, source.replace(originalUnixDigest, digest));
    fs.writeFileSync(path.join(tree, ".moonbit-version"), `${version}\n`);
    fs.writeFileSync(githubPath, "");
    fs.writeFileSync(githubEnv, "");
    const record = `const record = (stage, stdin = Buffer.alloc(0)) => fs.appendFileSync(${JSON.stringify(log)}, JSON.stringify({ stage, pid: process.pid, argv: process.argv.slice(2), moonHome: process.env.MOON_HOME, path: process.env.PATH, installVersion: process.env.MOONBIT_INSTALL_VERSION, stdinHex: stdin.toString("hex") }) + "\\n");`;
    const moonBody = `${record}
const stdin = fs.readFileSync(0); record("moon", stdin);
if (process.argv[2] === "update") {
  const state = process.env.MOON_HOME + "/controlled-update-count";
  const count = fs.existsSync(state) ? Number(fs.readFileSync(state, "utf8")) : 0;
  fs.writeFileSync(state, String(count + 1));
  writeAll(1, Buffer.from("update-" + (count + 1) + "\\n"));
  if (count === 0) { writeAll(2, Buffer.from(${JSON.stringify(transport)})); process.exit(255); }
} else if (process.argv[2] === "run") writeAll(1, Buffer.from("smoke-ready\\n"));
else process.exit(91);`;
    const mooncBody = `${record} record("moonc"); writeAll(1, Buffer.from(${JSON.stringify(`v${wrongVersion ? "0.invalid" : version} (controlled)\n`)}));`;
    function installFake() {
      fs.mkdirSync(path.join(moonHome, "bin"), { recursive: true });
      command(path.join(moonHome, "bin", "moon"), moonBody);
      command(path.join(moonHome, "bin", "moonc"), mooncBody);
      fs.mkdirSync(path.join(moonHome, "include"), { recursive: true });
      fs.writeFileSync(
        path.join(moonHome, "include", "moonbit.h"),
        "void *memcpy(void *dst, const void *src, size_t n);\n",
      );
    }
    command(
      path.join(bin, "curl"),
      `${record} record("curl"); const args = process.argv.slice(2); fs.writeFileSync(args[args.indexOf("-o") + 1], ${JSON.stringify(tamper ? "tampered payload" : fixture.toString())});`,
    );
    command(
      path.join(bin, "bash"),
      `${record} record("bash"); fs.mkdirSync(process.env.MOON_HOME + "/bin", { recursive: true }); const commands = ${JSON.stringify({ moon: moonBody, moonc: mooncBody })}; for (const [name, body] of Object.entries(commands)) { const file = process.env.MOON_HOME + "/bin/" + name; fs.writeFileSync(file, "#!/usr/bin/env node\\nimport fs from 'node:fs';\\n" + ${JSON.stringify(writeAll)} + "\\n" + body); fs.chmodSync(file, 0o755); } fs.mkdirSync(process.env.MOON_HOME + "/include", { recursive: true }); fs.writeFileSync(process.env.MOON_HOME + "/include/moonbit.h", "void *memcpy(void *dst, const void *src, size_t n);\\n");`,
    );
    if (warm) installFake();
    const env = {
      ...process.env,
      RUNNER_TEMP: runner,
      GITHUB_PATH: githubPath,
      GITHUB_ENV: githubEnv,
      PATH: `${bin}${path.delimiter}${path.dirname(process.execPath)}${path.delimiter}${process.env.PATH ?? ""}`,
    };
    const result = await child(location, env);
    const calls = fs.readFileSync(log, "utf8").trim().split("\n").map(JSON.parse);
    for (const call of calls) {
      assert.ok(call.pid > 0 && call.pid !== result.pid);
      assert.equal(call.moonHome, moonHome);
    }
    const exports = {
      path: fs.readFileSync(githubPath, "utf8"),
      env: fs.readFileSync(githubEnv, "utf8"),
    };
    return { ...result, calls, exports, runner, moonHome, env };
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
}

const unixOnly = { skip: process.platform === "win32" };
test(
  "controlled cold install verifies payload/version, retries update, runs smoke stdin and exports last",
  unixOnly,
  async () => {
    const result = await installer();
    assert.equal(result.status, 0);
    assert.deepEqual(
      result.calls.map((call) => call.stage),
      ["curl", "bash", "moonc", "moon", "moon", "moon"],
    );
    assert.deepEqual(result.calls[0].argv, [
      "-fsSL",
      "https://cli.moonbitlang.com/install/unix.sh",
      "-o",
      path.join(result.runner, "moonbit-install.sh"),
    ]);
    assert.deepEqual(result.calls[1].argv, [path.join(result.runner, "moonbit-install.sh")]);
    assert.equal(result.calls[1].installVersion, version);
    assert.deepEqual(result.calls[2].argv, ["-v"]);
    for (const call of result.calls.slice(3, 5)) {
      assert.deepEqual(call.argv, ["update"]);
      assert.equal(call.stdinHex, "");
      assert.equal(
        call.path,
        `${path.join(result.moonHome, "bin")}${path.delimiter}${result.env.PATH}`,
      );
    }
    const smoke = result.calls[5];
    assert.deepEqual(smoke.argv, ["run", "-q", "--target", "native", "-", "--"]);
    assert.match(Buffer.from(smoke.stdinHex, "hex").toString(), /moonbitlang\/async@0\.20\.1/);
    assert.match(Buffer.from(smoke.stdinHex, "hex").toString(), /moonbitlang\/x@0\.4\.47\/path/);
    assert.match(Buffer.from(smoke.stdinHex, "hex").toString(), /moonbit-setup-ok/);
    assert.equal(
      smoke.path,
      `${path.join(result.runner, "moonbit-shims")}${path.delimiter}${path.join(result.moonHome, "bin")}${path.delimiter}${result.env.PATH}`,
    );
    assert.equal(result.exports.path, `${path.join(result.runner, "moonbit-shims")}\n`);
    assert.equal(
      result.exports.env,
      `MOON_HOME=${result.moonHome}\nMOON_BIN=${path.join(result.runner, "moonbit-shims", "moon")}\n`,
    );
    assert.deepEqual(result.stdout, Buffer.from("update-1\nupdate-2\nsmoke-ready\n"));
    assert.deepEqual(rawStderr(result.stderr), Buffer.from(transport));
  },
);

test(
  "controlled warm cache skips downloads and updates but runs the original smoke",
  unixOnly,
  async () => {
    const result = await installer({ warm: true });
    assert.equal(result.status, 0);
    assert.deepEqual(
      result.calls.map((call) => call.stage),
      ["moonc", "moon"],
    );
    assert.deepEqual(result.calls[1].argv, ["run", "-q", "--target", "native", "-", "--"]);
    assert.ok(
      Buffer.from(result.calls[1].stdinHex, "hex").includes(Buffer.from("moonbit-setup-ok")),
    );
    assert.deepEqual(result.stdout, Buffer.from("smoke-ready\n"));
    assert.deepEqual(result.stderr, Buffer.alloc(0));
  },
);

for (const [name, options, stages, diagnostic] of [
  ["hash mismatch", { tamper: true }, ["curl"], /installer hash mismatch/],
  ["version mismatch", { wrongVersion: true }, ["curl", "bash", "moonc"], /version mismatch/],
]) {
  test(`controlled ${name} fails before registry update, smoke or exports`, unixOnly, async () => {
    const result = await installer(options);
    assert.equal(result.status, 1);
    assert.deepEqual(
      result.calls.map((call) => call.stage),
      stages,
    );
    assert.match(result.stderr.toString(), diagnostic);
    assert.deepEqual(result.exports, { path: "", env: "" });
    assert.doesNotMatch(result.stderr.toString(), /retrying/);
  });
}
