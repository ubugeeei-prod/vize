import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { compileFunction } from "node:vm";

export const REVISION = "5489aee433cd1054b9d72973457498544da7c467";
export const SUBMODULE = "tests/_fixtures/_git/vue-benchmarks";
export const SUITE_HASH = "3a09ab6b2b314ce5be7d60bf17fa06941f895f25fe0e305adf88c0a1b0ee06a2";
export const PROFILES = ["vize-lint-1t", "vize-lint-max"];
export const CONFIG_NAMES = ["eslint.config.mjs", "biome.json", ".oxlintrc.json", "package.json"];
export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

// Full source pins precede executing the private, fixed-marker extraction.
export const SOURCE_PINS = {
  "scripts/lib/lint-validity-child.mjs":
    "ce711aae5c99fdbc3f16faa630779a6caf14acde379c2d5babfec2ea7df19e2b",
  "scripts/lib/lint-validity-plants.mjs":
    "7f069c2b588cc11438deb0d2fcedb89de5342fe75f237eed656a8d19d510914b",
  "scripts/lib/lint-row-specs.mjs":
    "9936d99aca8008c9fbbcabd4352e286e25935b05ed3977cf216614766b504cb5",
  "scripts/lib/fixtures.mjs": "c6660eb0e8e34eb5fe4c2c40f8fc6dec61260a0160f70a827410037006b9d3aa",
  "scripts/lib/real-world/ansi.mjs":
    "d51f5c28a8d05d69b4394032f26827f954a880811c171d882459c3c35f133296",
  LICENSE: "c3081f92362a167af8f17890c6876dcdf06c1dbbd328347e9ff696103a538467",
};
export const PARSER_HASH = "634d5b0faebd8cf8c51fd14b1c588b9eddc4be60bc129f11bd0b67f6ad1a4a1a";
export const CONFIG_PINS = {
  "eslint.config.mjs": "cffda0e6be954fa743ebd37314c8dcb4faf79c70ab6c9920ed0c6065a5e39313",
  "biome.json": "35fa5b8377ea30b5ed97933668776a7fd4eb78cb7d2871971de520b838c6e84d",
  ".oxlintrc.json": "4ed20a8307cb06f36c76ca4c66721fddbb31224d73291841349fd250e37005ac",
  "package.json": "3bd0fee1ca8e2bce461870c9477702e4db1a0b12a37b9597755223484795a294",
};

// Authored from the pinned prepareLintDir body, before any current CLI run.
export const CONFIG_BYTES = {
  "eslint.config.mjs": `import pluginVue from "eslint-plugin-vue";
import tsParser from "@typescript-eslint/parser";

// parserOptions.parser is required for <script setup lang="ts"> — without it
// ESLint fatally fails to parse and silently skips those files.
export default [
  ...pluginVue.configs["flat/recommended"],
  {
    files: ["**/*.vue"],
    languageOptions: {
      parserOptions: {
        parser: tsParser,
        ecmaVersion: "latest",
        sourceType: "module",
      },
    },
    rules: {
      "vue/multi-word-component-names": "off",
      "vue/require-default-prop": "off",
      "vue/require-explicit-emits": "off",
    },
  },
];
`,
  "biome.json":
    '{\n  "formatter": {\n    "enabled": false\n  },\n  "linter": {\n    "enabled": true,\n    "rules": {\n      "recommended": true\n    }\n  }\n}\n',
  ".oxlintrc.json":
    '{\n  "plugins": [\n    "unicorn",\n    "typescript",\n    "oxc",\n    "vue"\n  ]\n}\n',
  "package.json": '{\n  "private": true,\n  "type": "module",\n  "name": "bench-lint-n200"\n}\n',
};

function git(cwd, args) {
  const result = spawnSync("git", args, { cwd, encoding: "utf8" });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.signal, null);
  return result.stdout.trim();
}

export function verifyUpstream(repoRoot) {
  const upstream = path.join(repoRoot, SUBMODULE);
  const manifest = JSON.parse(
    readFileSync(path.join(repoRoot, "tests/_fixtures/vue-benchmarks-upstream.json"), "utf8"),
  );
  assert.equal(manifest.fixturePath, SUBMODULE);
  assert.equal(manifest.revision, REVISION);
  assert.equal(manifest.publishedSnapshot.toolVersions.vize, "0.429.1");
  assert.equal(manifest.publishedSnapshot.toolVersions["@vizejs/native"], "0.429.1");
  assert.equal(
    git(repoRoot, ["ls-tree", "HEAD", "--", SUBMODULE]),
    `160000 commit ${REVISION}\t${SUBMODULE}`,
  );
  assert.equal(git(upstream, ["rev-parse", "HEAD"]), REVISION);
  assert.equal(git(upstream, ["status", "--porcelain", "--untracked-files=all"]), "");
  const sources = {};
  for (const [file, digest] of Object.entries(SOURCE_PINS)) {
    const bytes = readFileSync(path.join(upstream, file));
    assert.equal(sha256(bytes), digest, `upstream source drift: ${file}`);
    sources[file] = bytes.toString("utf8");
  }
  return { upstream, sources, revision: REVISION, sourcePins: SOURCE_PINS };
}

export function extractCliDiagnostics(source) {
  const startMarker = "function cliDiagnostics(raw) {";
  const endMarker = "\nasync function runEntrypoint";
  assert.equal(source.split(startMarker).length, 2);
  assert.equal(source.split(endMarker).length, 2);
  const start = source.indexOf(startMarker);
  const end = source.indexOf(endMarker, start);
  assert.ok(end > start);
  const extracted = source.slice(start, end);
  assert.equal(sha256(extracted), PARSER_HASH, "private CLI parser body drift");
  return extracted;
}

export async function loadUpstream(repoRoot) {
  const custody = verifyUpstream(repoRoot);
  const load = (file) => import(pathToFileURL(path.join(custody.upstream, file)).href);
  // These four modules import only Node builtins; their entire sources are pinned.
  const [plants, rows, fixtures, ansi] = await Promise.all([
    load("scripts/lib/lint-validity-plants.mjs"),
    load("scripts/lib/lint-row-specs.mjs"),
    load("scripts/lib/fixtures.mjs"),
    load("scripts/lib/real-world/ansi.mjs"),
  ]);
  assert.equal(plants.LINT_VALIDITY_SUITE_VERSION, "2026-09-12.2");
  assert.equal(plants.LINT_VALIDITY_SUITE_HASH, SUITE_HASH);
  assert.equal(plants.LINT_VALIDITY_PLANTS.length, 11);
  assert.equal(new Set(plants.LINT_VALIDITY_PLANTS.map((plant) => plant.id)).size, 11);
  assert.deepEqual(rows.lintCliCommand(PROFILES[0]), {
    bin: "vize",
    args: ["lint", "."],
    env: { RAYON_NUM_THREADS: "1" },
  });
  assert.deepEqual(rows.lintCliCommand(PROFILES[1]), { bin: "vize", args: ["lint", "."], env: {} });
  const body = extractCliDiagnostics(custody.sources["scripts/lib/lint-validity-child.mjs"]);
  // This executes only the unchanged parser, not the child's other entrypoints.
  const cliDiagnostics = compileFunction(`${body}\nreturn cliDiagnostics;`, ["stripAnsi"])(
    ansi.stripAnsi,
  );
  return { custody, ...plants, ...rows, ...fixtures, cliDiagnostics };
}

export function relativeFiles(plants) {
  return plants.map(
    (plant, index) => `nested/${String(index).padStart(2, "0")}/${plant.id}/Plant.vue`,
  );
}

export function verifyConfigs(cwd) {
  return CONFIG_NAMES.map((file) => {
    const bytes = readFileSync(path.join(cwd, file));
    assert.deepEqual(bytes, Buffer.from(CONFIG_BYTES[file]), `generated config drift: ${file}`);
    assert.equal(sha256(bytes), CONFIG_PINS[file]);
    return { file, bytes: bytes.length, sha256: sha256(bytes) };
  });
}

export function machineDiagnostics(rows) {
  assert.ok(Array.isArray(rows));
  const seen = new Set();
  return rows.flatMap((row) => {
    assert.equal(typeof row.file, "string");
    assert.ok(!seen.has(row.file), "duplicate CLI file row");
    seen.add(row.file);
    assert.ok(Array.isArray(row.messages));
    assert.equal(row.errorCount, row.messages.filter((message) => message.severity === 2).length);
    assert.equal(row.warningCount, row.messages.filter((message) => message.severity === 1).length);
    return row.messages.map((message) => {
      assert.equal(typeof message.ruleId, "string");
      assert.equal(typeof message.message, "string");
      assert.ok(message.severity === 1 || message.severity === 2);
      for (const field of ["line", "column", "endLine", "endColumn"]) {
        assert.ok(Number.isInteger(message[field]) && message[field] > 0, `invalid ${field}`);
      }
      assert.ok(message.endLine >= message.line);
      assert.ok(message.endLine !== message.line || message.endColumn >= message.column);
      return {
        file: row.file,
        line: message.line,
        column: message.column,
        rule: message.ruleId,
        message: message.message,
        raw: JSON.stringify(message),
      };
    });
  });
}
