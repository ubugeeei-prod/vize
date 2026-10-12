import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { nativePreparationIsActive } from "../../native/scripts/test-preparation.mjs";

const label = "qualification:js-plugin-host";
const asset = "https://api.github.com/repositories/357728969/releases/assets/633351903";
const archiveSha = "23bfb1b97309801082f2f7dc2f45a19f582352a901b1a21b1673cd843c7a8895";
const revision = "1.4.4-canary.1+b92e2f470";

/** Explicit, finite PR qualification; a default run claims no other host. */
export function referenceQualificationSelected() {
  if (process.env.GITHUB_ACTIONS !== "true" || process.env.GITHUB_EVENT_NAME !== "pull_request")
    return false;
  assert(process.env.GITHUB_EVENT_PATH, "Trusted PR event is required");
  const event = JSON.parse(readFileSync(process.env.GITHUB_EVENT_PATH, "utf8"));
  if (!event.pull_request?.labels.some((entry) => entry.name === label)) return false;
  assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
  assert.equal(event.repository.full_name, "ubugeeei-prod/vize");
  assert.equal(event.pull_request.head.repo.full_name, "ubugeeei-prod/vize");
  assert.equal(event.number, 8769);
  assert.equal(event.pull_request.number, 8769);
  assert.equal(process.env.GITHUB_REF, "refs/pull/8769/merge");
  return true;
}

/** Selected qualification must execute the pinned binary with the fresh addon. */
export function qualifyReferenceHost({ oracle, nativePath, source, clean, before, after }) {
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  const nativeDir = path.dirname(nativePath);
  assert(nativePreparationIsActive(nativeDir), "Current native producer is required");
  const producer = JSON.parse(
    readFileSync(path.join(nativeDir, ".artifacts/native/js-test-preparation.json"), "utf8"),
  );
  assert.equal(producer.head, process.env.GITHUB_SHA);
  const directory = mkdtempSync(path.join(tmpdir(), "vize-js-plugin-host-"));
  const proof = { asset, archiveSha, revision, producer, configs: [], cases: [] };
  try {
    const archive = path.join(directory, "runtime.zip");
    execFileSync(
      "curl",
      [
        "--fail",
        "--location",
        "--silent",
        "--show-error",
        "--max-time",
        "120",
        "--max-filesize",
        "49891968",
        "-H",
        "Accept: application/octet-stream",
        "-o",
        archive,
        asset,
      ],
      { timeout: 125000, maxBuffer: 65536 },
    );
    const bytes = readFileSync(archive);
    assert.equal(bytes.length, 49891968);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), archiveSha);
    const entries = execFileSync("unzip", ["-Z1", archive], { encoding: "utf8", maxBuffer: 65536 })
      .trim()
      .split("\n")
      .filter((entry) => !entry.endsWith("/"));
    assert.equal(entries.length, 1);
    assert.match(entries[0], /^[^./][^/]*\/[^./][^/]*$/);
    const binary = execFileSync("unzip", ["-p", archive, entries[0]], {
      maxBuffer: 256 * 1024 * 1024,
    });
    proof.binarySha = createHash("sha256").update(binary).digest("hex");
    const runtime = path.join(directory, "runtime.target");
    writeFileSync(runtime, binary);
    chmodSync(runtime, 0o755);
    const version = spawnSync(runtime, ["--revision"], { encoding: "utf8", timeout: 30000 });
    proof.actualRevision = version.stdout;
    assert.equal(version.status, 0);
    assert.equal(version.stdout.trim(), revision);
    const modules = path.join(directory, "node_modules");
    mkdirSync(modules);
    symlinkSync(path.dirname(path.dirname(oracle.resolve("eslint"))), path.join(modules, "eslint"));
    symlinkSync(
      path.dirname(oracle.resolve("vue-eslint-parser")),
      path.join(modules, "vue-eslint-parser"),
    );
    const config = path.join(directory, "eslint.config.mjs");
    const plugin = fileURLToPath(new URL("../dist/eslint.mjs", import.meta.url));
    function configure(rule) {
      writeFileSync(
        config,
        `import parser from "vue-eslint-parser";\nimport vize from ${JSON.stringify(plugin)};\nexport default [{files:["**/*.vue"],languageOptions:{parser},plugins:{vize},settings:{vize:{preset:"incremental",helpLevel:"none"}},rules:{${JSON.stringify(rule)}:"error"}}];\n`,
      );
      proof.configs.push({ rule, source: readFileSync(config, "utf8") });
    }
    function run(name, args, input, status) {
      const result = spawnSync(
        runtime,
        [
          "lint",
          "--config",
          config,
          "--no-native-plugin-rules",
          "--ext",
          "vue",
          "--format",
          "json",
          ...args,
        ],
        {
          cwd: directory,
          input,
          encoding: "utf8",
          timeout: 60000,
          maxBuffer: 2 * 1024 * 1024,
        },
      );
      const record = {
        name,
        args,
        input,
        status: result.status,
        stdout: result.stdout,
        stderr: result.stderr,
      };
      proof.cases.push(record);
      assert.equal(result.error, undefined);
      assert.equal(result.status, status);
      const report = JSON.parse(result.stdout);
      assert(Array.isArray(report));
      assert.equal(report.length, 1);
      assert.equal(report[0].filePath, path.join(directory, args.at(-1)));
      return report.flatMap((file) => file.messages);
    }
    configure("vize/a11y/alt-text");
    writeFileSync(path.join(directory, "Unsaved.vue"), clean);
    const messages = run(
      "unsaved-original-source",
      ["--stdin", "--stdin-filename", "Unsaved.vue"],
      source,
      1,
    );
    assert.equal(messages.length, 1);
    assert.deepEqual(
      {
        rule: messages[0].ruleId,
        severity: messages[0].severity,
        line: messages[0].line,
        column: messages[0].column,
        endLine: messages[0].endLine,
        endColumn: messages[0].endColumn,
      },
      { rule: "vize/a11y/alt-text", severity: 2, line: 1, column: 13, endLine: 1, endColumn: 33 },
    );
    assert.equal(messages[0].message, "<img> elements must have an alt attribute");
    assert.equal(messages[0].suggestions, undefined);
    assert.deepEqual(run("clean", ["Unsaved.vue"], undefined, 0), []);
    assert.equal(readFileSync(path.join(directory, "Unsaved.vue"), "utf8"), clean);
    configure("vize/vue/no-multi-spaces");
    writeFileSync(path.join(directory, "Fix.vue"), before);
    assert.deepEqual(run("original-fixed-source", ["--fix", "Fix.vue"], undefined, 0), []);
    const fixed = readFileSync(path.join(directory, "Fix.vue"));
    assert.deepEqual(fixed, Buffer.from(after));
    proof.before = before;
    proof.after = fixed.toString("utf8");
    proof.qualified = true;
    console.log(JSON.stringify({ referenceHost: proof }));
  } catch (error) {
    console.log(JSON.stringify({ referenceHostRefusal: proof }));
    throw error;
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}
