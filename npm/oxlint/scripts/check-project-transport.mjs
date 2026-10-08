// Reuse the existing JS Actions source build; never qualify a downloaded addon.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { beginN8nReplay, replayN8nHost, finishN8nReplay } from "./n8n-replay-qualification.mjs";
import {
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
} from "../../native/scripts/formatter-history-build.mjs";

const packageDir = fileURLToPath(new URL("..", import.meta.url));
const root = path.resolve(packageDir, "../..");
assert.equal(process.env.GITHUB_ACTIONS, "true", "this source qualification runs in Actions");
for (const key of [
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_OXLINT_NATIVE_CUSTODY",
])
  assert.ok(!process.env[key], `ambient ${key} cannot qualify source identity`);
const nativeDir = path.join(root, "npm/native");
const receipt = JSON.parse(fs.readFileSync(nativeHistoryReceipt(nativeDir), "utf8"));
validateNativeHistoryBuild(nativeDir, receipt);
assert.equal(receipt.source.head, process.env.GITHUB_SHA);
const artifacts = path.join(root, "target/oxlint-original-project-transport");
fs.mkdirSync(artifacts, { recursive: true });
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const binary = path.join(artifacts, "source-native.node");
fs.copyFileSync(receipt.frozen.path, binary, fs.constants.COPYFILE_EXCL);
assert.equal(hash(fs.readFileSync(binary)), receipt.frozen.sha256);
fs.copyFileSync(nativeHistoryReceipt(nativeDir), path.join(artifacts, "build-receipt.json"));
const preload = fileURLToPath(new URL("./project-native-custody.cjs", import.meta.url));
const qualifications = [];
const n8nReplay = beginN8nReplay({ root, packageDir, artifacts, receipt, binary });
for (const [version, types] of [
  ["1.78.0", "7.0.2001"],
  ["1.86.0", "7.0.2003"],
]) {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-oxlint-project-host-"));
  const output = path.join(artifacts, version);
  fs.mkdirSync(output);
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
    // Install only the two actual host tools outside the source checkout.
    run("npm", [
      "install",
      "--ignore-scripts",
      "--no-audit",
      "--no-fund",
      "--legacy-peer-deps",
      `oxlint@${version}`,
      `oxlint-tsgolint@${types}`,
    ]);
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
    assert.equal(
      guardLoads.size,
      5,
      "all five guard hosts must load the authenticated source addon",
    );
    assert.equal(
      batchCalls,
      18,
      "the selected 51-rule hint must lint each of nine originals once in each mode",
    );
    qualifications.push({
      version,
      types,
      calls,
      batchCalls,
      processes: loaded.size,
      originalProject: capture.qualified,
    });
    replayN8nHost(n8nReplay, engine, version);
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
}
validateNativeHistoryBuild(nativeDir, receipt);
assert.equal(hash(fs.readFileSync(binary)), receipt.frozen.sha256);
finishN8nReplay(n8nReplay);
fs.writeFileSync(
  path.join(artifacts, "qualification.json"),
  JSON.stringify(
    {
      source: receipt.source,
      toolchain: receipt.toolchain,
      binarySha256: receipt.frozen.sha256,
      qualifications,
      limits: [
        "authored wrapper controls only",
        "full licensed native/Vize-layer replay is qualified separately in n8n/qualification.json",
        "direct51 scriptless, excluded n8n-local plugins and installed acceptance remain unfinished",
      ],
    },
    null,
    2,
  ) + "\n",
);
