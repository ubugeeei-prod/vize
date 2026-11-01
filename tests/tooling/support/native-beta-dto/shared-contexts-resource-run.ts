import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import {
  ORDER,
  IMAGE,
  SELECTOR,
  object,
  memory,
  controls,
} from "./shared-contexts-resource-proc.ts";

const OUTPUT_LIMIT = 8 * 1024 * 1024;
const hash = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
const required = (name: string): string => {
  const value = process.env[name];
  assert.ok(value, `missing ${name}`);
  return value;
};
const write = (file: string, value: unknown): void =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
const read = (file: string): Record<string, unknown> =>
  object(JSON.parse(fs.readFileSync(file, "utf8")));
function main(): void {
  assert.equal(required("ROLE"), "after");
  assert.equal(required("VIZE_NATIVE_DTO_ROLE"), "after");
  assert.equal(required("SDK_VERSION"), "7.1.0-beta");
  const source = required("HEAD_SHA"),
    root = required("VIZE_NATIVE_DTO_SOURCE_ROOT"),
    capture = required("VIZE_NATIVE_DTO_CAPTURE");
  assert.match(source, /^[0-9a-f]{40}$/);
  assert.equal(required("VIZE_NATIVE_DTO_SOURCE_SHA"), source);
  assert.equal(
    execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim(),
    source,
  );
  assert.equal(fs.realpathSync(root), fs.realpathSync(required("GITHUB_WORKSPACE")));
  const previous = path.join(capture, "shared-contexts");
  assert.equal(read(path.join(previous, "source.json")).source, source);
  assert.equal(read(path.join(previous, "execution.json")).status, 0);
  assert.match(
    fs.readFileSync(path.join(previous, "stdout.bin"), "utf8"),
    /test result: ok\. 1 passed; 0 failed; 0 ignored;/,
  );
  const native = required("VIZE_NATIVE_DTO_ACTUAL_RUNTIME_PATH");
  assert.equal(hash(fs.readFileSync(native)), IMAGE);
  const artifacts = fs
    .readFileSync(path.join(capture, "build.jsonl"), "utf8")
    .trim()
    .split("\n")
    .map((line) => object(JSON.parse(line)))
    .filter(
      (row) =>
        row.reason === "compiler-artifact" &&
        object(row.target).name === "vize_canon" &&
        object(row.profile).test === true &&
        row.executable,
    );
  assert.equal(artifacts.length, 1);
  const artifact = artifacts[0]!,
    executable = String(artifact.executable),
    testHash = hash(fs.readFileSync(executable));
  assert.equal(artifact.fresh, false);
  assert.equal(
    fs.realpathSync(String(object(artifact.target).src_path)),
    fs.realpathSync(path.join(root, "crates/vize_canon/src/lib.rs")),
  );
  const relative = path.relative(
    fs.realpathSync(required("CARGO_TARGET_DIR")),
    fs.realpathSync(executable),
  );
  assert.ok(relative && !relative.startsWith("..") && !path.isAbsolute(relative));
  const output = path.join(capture, "shared-contexts-resource");
  assert.ok(!fs.existsSync(output));
  fs.mkdirSync(output);
  const space = fs.statfsSync(output);
  assert.ok(space.bavail * space.bsize >= 1024 * 1024 * 1024 + OUTPUT_LIMIT);
  const files = [
    "Cargo.toml",
    "Cargo.lock",
    ".github/workflows/native-beta-diagnostic-dto.yml",
    "tests/tooling/support/native-beta-dto/shared-contexts-run.ts",
    "tests/tooling/support/native-beta-dto/shared-contexts-resource-run.ts",
    "tests/tooling/support/native-beta-dto/shared-contexts-resource-proc.ts",
    "crates/vize_canon/src/lsp_client/editor_lsp/completion_resolve.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/configured_project/shared_transport/tests/native.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/configured_project/shared_transport/tests/native/shared_contexts.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/configured_project/shared_transport/tests/native/shared_contexts/tests.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/configured_project/shared_transport/tests/native/shared_contexts/production.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/configured_project/shared_transport/tests/native/shared_contexts/production/resource.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/configured_project/shared_transport/tests/native/shared_contexts/production/resource/proc.rs",
  ];
  const sourceRows = files.map((file) => {
    const bytes = fs.readFileSync(path.join(root, file));
    assert.deepEqual(
      bytes,
      execFileSync("git", ["show", `${source}:${file}`], {
        cwd: root,
        maxBuffer: 16 * 1024 * 1024,
      }),
    );
    return { path: file, bytes: bytes.length, sha256: hash(bytes) };
  });
  const input = path.join(output, "input");
  fs.mkdirSync(input);
  fs.mkdirSync(path.join(input, "a"));
  fs.mkdirSync(path.join(input, "b"));
  const fixture = {
    "a/source.ts": "export const answer: number = 'wrong';\nanswer.toFixed();\n",
    "b/source.ts": "export const label: string = 'clean';\nlabel.toUpperCase();\n",
    "tsconfig.json": JSON.stringify({
      compilerOptions: {
        strict: true,
        noEmit: true,
        types: [],
        target: "ESNext",
        module: "ESNext",
        moduleResolution: "Bundler",
      },
      files: ["a/source.ts"],
    }),
    "b/tsconfig.json": JSON.stringify({
      compilerOptions: {
        strict: false,
        noEmit: true,
        types: [],
        target: "ESNext",
        module: "ESNext",
        moduleResolution: "Bundler",
      },
      files: ["source.ts"],
    }),
  };
  const fixtureRows = Object.entries(fixture).map(([file, text]) => {
    fs.writeFileSync(path.join(input, file), text, { flag: "wx" });
    return { path: file, bytes: Buffer.byteLength(text), sha256: hash(Buffer.from(text)) };
  });
  write(path.join(output, "source.json"), {
    source,
    sdkVersion: "7.1.0-beta",
    native,
    nativeSHA256: IMAGE,
    buildArtifact: artifact,
    testExecutable: executable,
    testExecutableSHA256: testHash,
    files: sourceRows,
    fixtureRoot: input,
    fixture: fixtureRows,
    selector: SELECTOR,
    order: ORDER,
    scope:
      "two configured projects in actual workspace clients; Rust test-root plus native descendants, not Maestro pool; no trace; OS page cache uncontrolled",
  });
  const commonEnvironment = {
    ...process.env,
    CORSA_PATH: native,
    VIZE_TEST_REQUIRE_TSGO: "1",
    VIZE_RESOURCE_INPUT_ROOT: input,
    VIZE_RESOURCE_TEST_IMAGE_SHA256: testHash,
  };
  const results: unknown[] = [];
  let decoded = 0;
  for (const [index, mode] of ORDER.entries()) {
    const directory = path.join(output, `${index + 1}-${mode}`);
    fs.mkdirSync(directory);
    const stdout = fs.openSync(path.join(directory, "stdout.bin"), "wx"),
      stderr = fs.openSync(path.join(directory, "stderr.bin"), "wx");
    const args = [SELECTOR, "--ignored", "--exact", "--nocapture", "--test-threads=1"];
    const started = Date.now();
    let result: ReturnType<typeof spawnSync>;
    try {
      result = spawnSync(executable, args, {
        cwd: root,
        env: {
          ...commonEnvironment,
          VIZE_RESOURCE_MODE: mode,
          VIZE_PRIVATE_SHARED_CONTEXT_CAPTURE: path.join(directory, "packets"),
        },
        stdio: ["ignore", stdout, stderr],
        timeout: 120_000,
      });
    } finally {
      fs.closeSync(stdout);
      fs.closeSync(stderr);
    }
    write(path.join(directory, "execution.json"), {
      executable,
      args,
      mode,
      source,
      native,
      pid: result.pid,
      image: IMAGE,
      testImage: testHash,
      input,
      startedUnixMs: started,
      endedUnixMs: Date.now(),
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      caseEnvironmentDifferences: ["VIZE_RESOURCE_MODE", "VIZE_PRIVATE_SHARED_CONTEXT_CAPTURE"],
    });
    assert.equal(result.status, 0);
    assert.equal(result.signal, null);
    assert.equal(result.error, undefined);
    assert.match(
      fs.readFileSync(path.join(directory, "stdout.bin"), "utf8"),
      /test result: ok\. 1 passed; 0 failed; 0 ignored;/,
    );
    assert.equal(fs.statSync(path.join(directory, "stderr.bin")).size, 0);
    assert.equal(hash(fs.readFileSync(native)), IMAGE);
    assert.equal(hash(fs.readFileSync(executable)), testHash);
    for (const row of fixtureRows)
      assert.equal(hash(fs.readFileSync(path.join(input, row.path))), row.sha256);
    const packets = path.join(directory, "packets");
    const roster = fs.readdirSync(packets).sort();
    const memoryRows: unknown[] = [];
    for (const file of roster) {
      assert.match(file, /^[a-z0-9-]+\.json$/);
      const bytes = fs.readFileSync(path.join(packets, file));
      decoded += bytes.length;
      assert.ok(bytes.length < 1024 * 1024 && decoded <= OUTPUT_LIMIT);
      if (file.startsWith("resource-memory-")) {
        const row = object(JSON.parse(bytes.toString("utf8")));
        const phase = String(row.milestone);
        memory(
          row,
          ["root-before", "owners-retired", "final"].includes(phase)
            ? 0
            : mode === "baseline"
              ? 2
              : 1,
          executable,
          testHash,
          native,
          input,
          result.pid,
          root,
        );
        memoryRows.push(row);
      }
    }
    assert.equal(memoryRows.length, 11);
    assert.deepEqual(
      memoryRows.map((value) => String(object(value).milestone)).sort(),
      [
        "root-before",
        "initial",
        "strict-null",
        "repaired",
        "current-boolean",
        "a-detached",
        "a-reopened-b-error",
        "repaired-leases-dropped",
        "owners-retired",
        "restarted",
        "final",
      ].sort(),
    );
    write(
      path.join(directory, "roster.json"),
      roster.map((file) => {
        const bytes = fs.readFileSync(path.join(packets, file));
        return { file, bytes: bytes.length, sha256: hash(bytes) };
      }),
    );
    results.push({
      index: index + 1,
      mode,
      memoryRows,
      firstUsable: read(path.join(packets, "resource-first-usable.json")),
      packetCount: roster.length,
    });
  }
  write(path.join(output, "result.json"), {
    source,
    order: ORDER,
    results,
    decodedPacketBytes: decoded,
    scope:
      "prospective same-image finite memory discriminator; all raw packets preserved; not a production pool, cold OS cache, peak or speedup claim",
  });
  console.log(
    JSON.stringify({ source, order: ORDER, functionalCases: 4, rawPacketBytes: decoded }),
  );
}
if (process.argv[2] === "--controls") controls();
else main();
