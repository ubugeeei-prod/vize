import { requiredEnv } from "../../../lib/scripts/files.ts";
import { pathToFileURL } from "node:url";
import { resolve as resolveWorkflowInput } from "node:path";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { spawnSync } from "node:child_process";
const { names, executions, reject } = (await import(
  pathToFileURL(resolveWorkflowInput("./tests/tooling/support/content-mapper-editor-inventory.ts"))
    .href
)) as typeof import("../../../../../tests/tooling/support/content-mapper-editor-inventory.ts");

const root = process.cwd();
const capture = path.join(requiredEnv("RUNNER_TEMP"), "content-mapper-editor-stress");
mkdirSync(capture, { recursive: true });
const digest = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const fileHash = (file: string) => digest(readFileSync(file));
const write = (name: string, value: unknown) =>
  writeFileSync(path.join(capture, name), JSON.stringify(value, null, 2) + "\n");
const legacy =
  "ide::diagnostics::editor_typecheck_unknown_options_tests::original_unknown_options_preserve_complete_editor_vectors";
interface InventoryItem {
  target: string;
  name: string;
  ignored: boolean;
  provider: string;
}
interface Target {
  key: string;
  selector: string[];
  executable?: string;
  executableSha256?: string;
  features?: string[];
  inventory: string[];
  ignored: string[];
}
type Completed = ReturnType<typeof executions>[number] & { target: string; provider: string };
interface StressReceipt {
  source: string | undefined;
  run: string | undefined;
  status: string;
  initializationEvidence: string;
  providers: {
    mapper?: unknown;
    original?: {
      binary: string;
      sha256: string;
      manifestPath: string;
      manifestSha256: string;
      version: string;
      gitHead: string;
    };
  };
  inventory: InventoryItem[];
  cycles: { cycle: number; completed: Completed[]; active: number; ignored: number }[];
  targets?: Target[];
  error?: string;
}
const receipt: StressReceipt = {
  source: process.env.GITHUB_SHA,
  run: process.env.GITHUB_RUN_ID,
  status: "RUNNING",
  initializationEvidence:
    "Fixture, source and startup configuration custody; not a raw RPC witness",
  providers: {},
  inventory: [],
  cycles: [],
};
const aliases = [
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
  "VIZE_TEST_TSGO_PATH",
  "VIZE_TEST_DISABLE_TSGO",
  "VIZE_UNKNOWN_TEMPLATE_CAPTURE",
];
function environment(binary?: string, fixtureCapture?: string) {
  const env: NodeJS.ProcessEnv = { ...process.env, SOURCE_SHA: process.env.GITHUB_SHA };
  for (const key of aliases) delete env[key];
  if (binary)
    Object.assign(env, {
      CORSA_PATH: binary,
      TSGO_PATH: binary,
      VIZE_TEST_TSGO_PATH: binary,
      VIZE_TEST_REQUIRE_TSGO: "1",
    });
  if (fixtureCapture) env.VIZE_UNKNOWN_TEMPLATE_CAPTURE = fixtureCapture;
  return env;
}
let sequence = 0;
function run(
  label: string,
  command: string,
  args: string[],
  binary?: string,
  fixtureCapture?: string,
) {
  const directory = `${String(sequence++).padStart(3, "0")}-${label}`;
  const destination = path.join(capture, directory);
  mkdirSync(destination);
  const env = environment(binary, fixtureCapture);
  const result = spawnSync(command, args, {
    cwd: root,
    env,
    maxBuffer: 64 * 1024 * 1024,
    timeout: 180000,
  });
  const stdout = result.stdout ?? Buffer.alloc(0),
    stderr = result.stderr ?? Buffer.alloc(0);
  writeFileSync(path.join(destination, "stdout.bin"), stdout);
  writeFileSync(path.join(destination, "stderr.bin"), stderr);
  writeFileSync(
    path.join(destination, "process.json"),
    JSON.stringify(
      {
        command,
        arguments: args,
        cwd: root,
        provider: binary ?? null,
        providerSha256: binary ? fileHash(binary) : null,
        startupEnvironment: Object.fromEntries(
          [...aliases, "SOURCE_SHA", "VIZE_TEST_REQUIRE_TSGO"].map((key) => [
            key,
            env[key] ?? null,
          ]),
        ),
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        stdoutSha256: digest(stdout),
        stderrSha256: digest(stderr),
      },
      null,
      2,
    ) + "\n",
  );
  assert.equal(result.error, undefined, `${label}: process error; retained under ${directory}`);
  assert.equal(result.signal, null, `${label}: process signal`);
  assert.equal(result.status, 0, `${label}: full output retained under ${directory}`);
  if (label === "original-version") assert.equal(stderr.length, 0);
  return stdout.toString("utf8");
}
// Negative controls guard against empty, omitted, duplicate and widened evidence.
const sample =
  "test a ... ok\ntest b ... ignored\ntest result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out;\n";
names("a: test\nb: test\n");
reject("empty inventory", () => executions(sample, names(""), [], []));
reject("omitted inventory row", () => {
  const partial = names("b: test\n");
  executions(sample, partial, ["b"], partial);
});
reject("duplicate inventory row", () => names("a: test\na: test\n"));
executions(sample, ["a", "b"], ["b"], ["a", "b"]);
reject("missing case", () =>
  executions(sample.replace("test a ... ok\n", ""), ["a", "b"], ["b"], ["a", "b"]),
);
reject("duplicate case", () =>
  executions("test a ... ok\n" + sample, ["a", "b"], ["b"], ["a", "b"]),
);
reject("changed ignore policy", () => executions(sample, ["a", "b"], [], ["a", "b"]));
reject("changed total", () =>
  executions(sample.replace("1 passed", "2 passed"), ["a", "b"], ["b"], ["a", "b"]),
);
reject("undeclared filtering", () =>
  executions(sample.replace("0 filtered out", "1 filtered out"), ["a", "b"], ["b"], ["a", "b"]),
);
const docs = names(
  "doc_a: test\ndoc_b: test\nall doctests ran in 0.01s; merged doctests compilation took 0.00s\n",
  "rustdoc",
);
const docSample =
  "test doc_a - compile ... ok\ntest doc_b - compile ... ok\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;\n";
executions(docSample, docs, [], docs, "rustdoc");
reject("empty Rustdoc inventory", () =>
  executions(docSample, names("", "rustdoc"), [], [], "rustdoc"),
);
reject("omitted Rustdoc row", () => {
  const partial = names("doc_a: test\n", "rustdoc");
  executions(docSample, partial, [], partial, "rustdoc");
});
reject("wrong timing producer", () =>
  names("all doctests ran in 0.01s; merged doctests compilation took 0.00s\n"),
);
reject("invalid Rustdoc timing", () =>
  names("all doctests ran in NaNs; merged doctests compilation took 0.00s\n", "rustdoc"),
);
reject("unknown compile case", () =>
  executions(docSample.replace("doc_a - compile", "doc_c - compile"), docs, [], docs, "rustdoc"),
);
const panicSample =
  "test expected_panic - should panic ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;\n";
executions(panicSample, ["expected_panic"], [], ["expected_panic"]);
reject("unknown panic case", () => executions(panicSample, ["other_panic"], [], ["other_panic"]));
reject("failed expected panic", () =>
  executions(
    panicSample.replace("... ok", "... FAILED"),
    ["expected_panic"],
    [],
    ["expected_panic"],
  ),
);
write("negative-controls.json", {
  emptyInventory: "rejected",
  omittedInventory: "rejected",
  duplicateInventory: "rejected",
  missing: "rejected",
  duplicate: "rejected",
  changedIgnore: "rejected",
  changedTotal: "rejected",
  undeclaredFiltering: "rejected",
  emptyRustdoc: "rejected",
  omittedRustdoc: "rejected",
  wrongTimingProducer: "rejected",
  invalidRustdocTiming: "rejected",
  unknownCompileCase: "rejected",
  unknownPanicCase: "rejected",
  failedExpectedPanic: "rejected",
});

try {
  const mapper = realpathSync(path.join(requiredEnv("RUNNER_TEMP"), "tsgo"));
  const mapperReceipt = JSON.parse(
    readFileSync(path.join(requiredEnv("RUNNER_TEMP"), "content-mapper-native-host.json"), "utf8"),
  );
  assert.equal(fileHash(mapper), mapperReceipt.sha256);
  assert.equal(mapperReceipt.upstream, process.env.CONTENT_MAPPER_TYPESCRIPT_SHA);
  receipt.providers.mapper = mapperReceipt;
  const require = createRequire(path.join(root, "npm/cli/package.json"));
  const manifestPath = require.resolve("@typescript/typescript-linux-x64/package.json");
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  assert.equal(manifest.name, "@typescript/typescript-linux-x64");
  assert.equal(manifest.version, "7.0.2");
  const original = realpathSync(path.join(path.dirname(manifestPath), "lib/tsc"));
  assert.equal(readFileSync(original).subarray(0, 4).toString("hex"), "7f454c46");
  assert.equal(
    fileHash(original),
    "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
  );
  assert.equal(run("original-version", original, ["--version"]), "Version 7.0.2\n");
  receipt.providers.original = {
    binary: original,
    sha256: fileHash(original),
    manifestPath,
    manifestSha256: fileHash(manifestPath),
    version: manifest.version,
    gitHead: manifest.gitHead,
  };
  const metadata = JSON.parse(
    run("metadata", "cargo", ["metadata", "--locked", "--no-deps", "--format-version=1"]),
  );
  const pkg = metadata.packages.find((pkg: { name: string }) => pkg.name === "vize_maestro");
  assert.ok(pkg);
  const build = run(
    "build-once",
    "cargo",
    ["test", "--locked", "-p", "vize_maestro", "--no-run", "--message-format=json"],
    mapper,
  );
  const messages = build
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line));
  assert.equal(messages.at(-1).reason, "build-finished");
  assert.equal(messages.at(-1).success, true);
  const binaries = messages.filter(
    (message) =>
      message.reason === "compiler-artifact" &&
      message.package_id === pkg.id &&
      message.profile.test &&
      message.executable,
  );
  assert.ok(binaries.length > 0);
  const targets: Target[] = binaries.map((artifact) => {
    const kind = artifact.target.kind;
    assert.equal(kind.length, 1);
    assert.ok(kind[0] === "lib" || kind[0] === "test");
    return {
      key: `${kind[0]}:${artifact.target.name}`,
      selector: kind[0] === "lib" ? ["--lib"] : ["--test", artifact.target.name],
      inventory: [],
      ignored: [],
      executable: artifact.executable,
      executableSha256: fileHash(artifact.executable),
      features: artifact.features,
    };
  });
  targets.push({ key: "doctest:vize_maestro", selector: ["--doc"], inventory: [], ignored: [] });
  assert.equal(new Set(targets.map((target) => target.key)).size, targets.length);
  for (const target of targets) {
    const command = ["test", "--locked", "-p", "vize_maestro", ...target.selector, "--"];
    const producer = target.key.startsWith("doctest:") ? "rustdoc" : "libtest";
    target.inventory = names(
      run(
        `list-${target.key.replaceAll(":", "-")}`,
        "cargo",
        [...command, "--list", "--format", "terse"],
        mapper,
      ),
      producer,
    );
    target.ignored = names(
      run(
        `ignored-${target.key.replaceAll(":", "-")}`,
        "cargo",
        [...command, "--ignored", "--list", "--format", "terse"],
        mapper,
      ),
      producer,
    );
    assert.ok(target.ignored.every((name) => target.inventory.includes(name)));
    for (const name of target.inventory)
      receipt.inventory.push({
        target: target.key,
        name,
        ignored: target.ignored.includes(name),
        provider: name === legacy ? "original" : "mapper",
      });
  }
  assert.equal(receipt.inventory.filter((test) => test.name.includes(legacy)).length, 1);
  assert.equal(receipt.inventory.find((test) => test.name === legacy)?.ignored, false);
  const legacyTarget = targets.find((target) => target.inventory.includes(legacy));
  assert.ok(legacyTarget);
  assert.equal(legacyTarget.key, "lib:vize_maestro");
  receipt.targets = targets;
  const identity = (test: { target: string; name: string }) => `${test.target}\0${test.name}`;
  const expectedByIdentity = new Map(receipt.inventory.map((test) => [identity(test), test]));
  assert.equal(expectedByIdentity.size, receipt.inventory.length);
  write("receipt.json", receipt);
  for (let cycle = 1; cycle <= 20; cycle++) {
    const completed = [];
    for (const target of targets) {
      const selected = target.inventory.filter((name: string) => name !== legacy);
      const command = [
        "test",
        "--locked",
        "-p",
        "vize_maestro",
        ...target.selector,
        "--",
        "--format",
        "pretty",
        "--color",
        "never",
        "--skip",
        legacy,
      ];
      const stdout = run(
        `cycle-${cycle}-${target.key.replaceAll(":", "-")}`,
        "cargo",
        command,
        mapper,
      );
      const producer = target.key.startsWith("doctest:") ? "rustdoc" : "libtest";
      completed.push(
        ...executions(stdout, target.inventory, target.ignored, selected, producer).map(
          (event) => ({ target: target.key, ...event, provider: "mapper" }),
        ),
      );
    }
    const fixtureCapture = path.join(capture, `cycle-${cycle}-original-fixtures`);
    const stdout = run(
      `cycle-${cycle}-original`,
      "cargo",
      [
        "test",
        "--locked",
        "-p",
        "vize_maestro",
        ...legacyTarget.selector,
        "--",
        "--exact",
        legacy,
        "--format",
        "pretty",
        "--color",
        "never",
      ],
      original,
      fixtureCapture,
    );
    completed.push(
      ...executions(stdout, legacyTarget.inventory, legacyTarget.ignored, [legacy]).map(
        (event) => ({ target: legacyTarget.key, ...event, provider: "original" }),
      ),
    );
    assert.deepEqual(
      completed.map(identity).sort(),
      receipt.inventory.map(identity).sort(),
      "combined providers must execute the complete original inventory exactly once per cycle",
    );
    assert.equal(completed.filter((test) => test.provider === "original").length, 1);
    for (const test of completed)
      assert.equal(test.provider, expectedByIdentity.get(identity(test))?.provider);
    assert.equal(fileHash(mapper), mapperReceipt.sha256);
    assert.equal(fileHash(original), receipt.providers.original.sha256);
    for (const target of targets.filter((target) => target.executable))
      assert.equal(
        fileHash(target.executable!),
        target.executableSha256,
        "test executable must remain the once-built image",
      );
    receipt.cycles.push({
      cycle,
      completed,
      active: completed.filter((test) => test.state === "ok").length,
      ignored: completed.filter((test) => test.state === "ignored").length,
    });
    write("receipt.json", receipt);
    console.log(
      `Editor lifecycle cycle ${cycle}/20 passed: ${receipt.cycles.at(-1)!.active} active + ${receipt.cycles.at(-1)!.ignored} declared ignores; both exact providers covered`,
    );
  }
  assert.equal(receipt.cycles.length, 20);
  receipt.status = "COMPLETE_INVENTORY_PASSED";
} catch (error) {
  receipt.status = "FAILED";
  receipt.error = error instanceof Error ? error.stack : String(error);
  throw error;
} finally {
  write("receipt.json", receipt);
}
