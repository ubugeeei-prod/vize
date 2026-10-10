import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";
import { replayN8nHost } from "./n8n-replay-qualification.mjs";

const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

export function verifyProjectHostCalls(events, custody, receipt) {
  const loaded = new Set();
  let calls = 0;
  let batchCalls = 0;
  const guardLoads = new Set();
  for (const event of events) {
    assert.deepEqual(event.source, receipt.source);
    assert.equal(event.binary, custody.binary.path);
    assert.equal(event.binarySha256, receipt.frozen.sha256);
    if (event.kind === "load") {
      loaded.add(event.pid);
      if (/^Guard-[0-4]\.vue$/u.test(event.control ?? "")) guardLoads.add(event.control);
    } else {
      assert.equal(event.kind, "call");
      assert.ok(loaded.has(event.pid));
      assert.equal(event.outcome, "return");
      assert.equal(hash(Buffer.from(event.args[0], "utf8")), event.originalSha256);
      assert.ok(!path.basename(event.args[1].filename).startsWith("Guard-"));
      assert.ok(
        !/^Guard-[0-4]\.vue$/u.test(event.control ?? ""),
        "a tampered carrier reached native linting",
      );
      calls++;
      if (event.hasRuleHint && event.args[1].enabledRules?.length === 51) batchCalls++;
    }
  }
  assert.ok(calls > 0, "no physical source-addon lint call was observed");
  assert.equal(guardLoads.size, 5, "all five guard hosts must load the authenticated source addon");
  assert.equal(
    batchCalls,
    18,
    "the selected 51-rule hint must lint each of nine originals once in each mode",
  );
  return { calls, batchCalls, processes: loaded.size };
}

function runProjectHost({ packageDir, artifacts, binary, replay, version, types, temporary }) {
  const receipt = JSON.parse(fs.readFileSync(path.join(artifacts, "build-receipt.json"), "utf8"));
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(receipt.source.head, process.env.GITHUB_SHA);
  assert.equal(hash(fs.readFileSync(binary)), receipt.frozen.sha256);
  const output = path.join(artifacts, version);
  const phaseWalltimeMs = {};
  const started = performance.now();
  let status = "failed";
  const timed = (name, action) => {
    const phaseStart = performance.now();
    try {
      return action();
    } finally {
      phaseWalltimeMs[name] = performance.now() - phaseStart;
    }
  };
  const run = (command, args, options = {}) => {
    const result = spawnSync(command, args, {
      cwd: temporary,
      encoding: "utf8",
      timeout: 180_000,
      maxBuffer: 64 * 1024 * 1024,
      ...options,
    });
    fs.writeFileSync(
      path.join(output, `${command === "npm" ? "install" : "tests"}.json`),
      JSON.stringify({ command, args, ...result }, null, 2) + "\n",
    );
    process.stdout.write(result.stdout ?? "");
    process.stderr.write(result.stderr ?? "");
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, `Oxlint ${version} original-project qualification failed`);
  };
  try {
    // These are the actual two pinned host tools, installed outside the source
    // checkout. No package is published and no downloaded addon is qualified.
    timed("install", () =>
      run("npm", [
        "install",
        "--ignore-scripts",
        "--no-audit",
        "--no-fund",
        "--legacy-peer-deps",
        `oxlint@${version}`,
        `oxlint-tsgolint@${types}`,
      ]),
    );
    const engine = path.join(temporary, "node_modules/oxlint/bin/oxlint");
    assert.equal(
      JSON.parse(fs.readFileSync(path.join(temporary, "node_modules/oxlint/package.json"))).version,
      version,
    );
    assert.equal(
      JSON.parse(fs.readFileSync(path.join(temporary, "node_modules/oxlint-tsgolint/package.json")))
        .version,
      types,
    );
    fs.copyFileSync(
      path.join(temporary, "package-lock.json"),
      path.join(output, "package-lock.json"),
    );
    const custody = {
      schema: "vize.oxlint.source-native",
      version: 1,
      source: receipt.source,
      toolchain: receipt.toolchain,
      binary: { path: fs.realpathSync(binary), sha256: receipt.frozen.sha256 },
      calls: path.join(output, "native-calls.jsonl"),
    };
    const configuration = path.join(output, "custody.json");
    fs.writeFileSync(configuration, JSON.stringify(custody, null, 2) + "\n");
    const preload = fileURLToPath(new URL("./project-native-custody.cjs", import.meta.url));
    const qualification = timed("transport", () => {
      run(
        process.execPath,
        [
          "--test",
          "src/project-transport.test.ts",
          "src/script-safe-transport.test.ts",
          "src/original-ignore.test.ts",
        ],
        {
          cwd: packageDir,
          env: {
            ...process.env,
            NODE_OPTIONS:
              `${process.env.NODE_OPTIONS ?? ""} --require=${JSON.stringify(preload)}`.trim(),
            VIZE_OXLINT_NATIVE_CUSTODY: configuration,
            VIZE_OXLINT_TEST_ENTRYPOINT: engine,
            VIZE_REQUIRE_REAL_OXLINT_TYPE_AWARE: "1",
            VIZE_OXLINT_PROJECT_CAPTURE: path.join(output, "original-project.json"),
            VIZE_OXLINT_SCRIPT_SAFE_CAPTURE: path.join(output, "script-safe.json"),
            VIZE_OXLINT_ORIGINAL_IGNORE_CAPTURE: path.join(output, "original-ignore.json"),
          },
        },
      );
      const capture = JSON.parse(fs.readFileSync(path.join(output, "original-project.json")));
      assert.deepEqual(capture.qualified, { imports: true, typeAware: true });
      const events = fs
        .readFileSync(custody.calls, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line));
      return {
        version,
        types,
        ...verifyProjectHostCalls(events, custody, receipt),
        originalProject: capture.qualified,
      };
    });
    const { host: n8nHost, custody: n8nCustody } = timed("n8nReplay", () =>
      replayN8nHost({ ...replay, hosts: [], hostCustodies: [] }, engine, version),
    );
    assert.equal(hash(fs.readFileSync(binary)), receipt.frozen.sha256);
    fs.writeFileSync(
      path.join(output, "host-worker-result.json"),
      JSON.stringify({ qualification, n8nHost, n8nCustody }, null, 2) + "\n",
    );
    status = "passed";
  } finally {
    phaseWalltimeMs.total = performance.now() - started;
    fs.writeFileSync(
      path.join(output, "phase-walltime.json"),
      JSON.stringify({ version, types, status, phaseWalltimeMs }, null, 2) + "\n",
    );
    fs.rmSync(temporary, { recursive: true, force: true });
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.argv.length, 3, "a host worker takes only its owned configuration path");
  runProjectHost(JSON.parse(fs.readFileSync(process.argv[2], "utf8")));
}
