import assert from "node:assert/strict";
import { validateProviderEnvelope } from "../../../../tooling/support/public-vue-imports-reference-contract.mjs";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";
import { sha256, wholeJson } from "../../../../tooling/lint-default-correctness-reference.mjs";

// All vectors predate execution. This records whole released/provider packets;
// it neither executes candidate source nor captures any expected vector.
const fixture = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(fixture, "../../../../..");
const read = (name) => fs.readFileSync(path.join(fixture, name));
const primaryBytes = read("primary-sources.json");
const primary = JSON.parse(primaryBytes);
const corpusBytes = read(primary.corpus.path);
assert.equal(sha256(corpusBytes), primary.corpus.sha256);
const corpus = JSON.parse(corpusBytes);
assert.equal(corpus.cases.length, 29);
const supplementBytes = read(primary.providerEnvelope.path);
assert.equal(sha256(supplementBytes), primary.providerEnvelope.sha256);
const supplement = JSON.parse(supplementBytes);
assert.equal(supplement.initialCorpusSha256, sha256(corpusBytes));
assert.equal(sha256(read(supplement.serializer.fixture)), supplement.serializer.sha256);
const fixBytes = read(primary.releasedFixContract.path);
assert.equal(sha256(fixBytes), primary.releasedFixContract.sha256);
const fixContract = JSON.parse(fixBytes);
assert.equal(fixContract.corpusSha256, sha256(corpusBytes));
validateProviderEnvelope(corpus, supplement);
assert.equal(sha256(fs.readFileSync(fileURLToPath(import.meta.url))), primary.observer.sha256);
assert.equal(
  sha256(fs.readFileSync(path.join(root, primary.observer.wholeJson.path))),
  primary.observer.wholeJson.sha256,
);
const binary = process.env.VIZE_PUBLIC_BINARY ?? primary.publicBefore.binary;
const binaryBefore = sha256(fs.readFileSync(binary));
assert.equal(binaryBefore, primary.publicBefore.binarySha256);
function packageIdentity() {
  const identities = [];
  for (const entry of primary.provider.packages) {
    const packageJson = fs.readFileSync(path.join(entry.packageRoot, "package.json"));
    assert.equal(sha256(packageJson), entry.packageJsonSha256);
    assert.equal(JSON.parse(packageJson).version, entry.version);
    const files = (directory) =>
      fs.readdirSync(directory, { withFileTypes: true }).flatMap((item) => {
        if (item.name === "node_modules") return [];
        const filename = path.join(directory, item.name);
        return item.isDirectory() ? files(filename) : item.isFile() ? [filename] : [];
      });
    const names = files(entry.packageRoot).sort();
    const tree = names
      .map(
        (filename) =>
          `${sha256(fs.readFileSync(filename))}  ${path.relative(entry.packageRoot, filename).split(path.sep).join("/")}\n`,
      )
      .join("");
    assert.equal(names.length, entry.fileCount);
    assert.equal(sha256(tree), entry.fileTreeSha256);
    identities.push({ name: entry.name, version: entry.version, treeSha256: sha256(tree) });
  }
  return identities;
}
const packagesBefore = packageIdentity();
const requireProvider = createRequire(path.join(primary.provider.root, "package.json"));
const { ESLint } = requireProvider("eslint");
const vue = requireProvider("eslint-plugin-vue");
const parser = requireProvider("vue-eslint-parser");
const scriptParser = requireProvider("@typescript-eslint/parser");
const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-vue-import-reference-"));
const config = JSON.stringify(primary.nativeConfig, null, 2) + "\n";
fs.writeFileSync(path.join(directory, "vize.config.json"), config);
const eslint = new ESLint({
  cwd: directory,
  overrideConfigFile: true,
  ignore: false,
  overrideConfig: [
    {
      files: ["**/*.vue", "**/*.ts"],
      languageOptions: {
        parser,
        ecmaVersion: "latest",
        sourceType: "module",
        parserOptions: { parser: scriptParser, ecmaVersion: "latest", sourceType: "module" },
      },
      plugins: { vue },
      rules: { "vue/prefer-import-from-vue": "error" },
    },
  ],
});
const custody = {
  corpusSha256: sha256(corpusBytes),
  primarySourcesSha256: sha256(primaryBytes),
  helperSha256: primary.observer.sha256,
  wholeJsonSha256: primary.observer.wholeJson.sha256,
};
function prior(name) {
  if (!fs.existsSync(path.join(fixture, name))) return [];
  const bytes = read(name);
  return [
    {
      sha256: sha256(bytes),
      bytes: bytes.length,
      encoding: "gzip-base64",
      completePacket: gzipSync(bytes).toString("base64"),
    },
  ];
}
const released = {
  schema: "vize.lint.public-vue-imports-released-before",
  version: 1,
  authority:
    "Whole authenticated v0.443.0 CLI packets checked against independently authored current-parent vectors. Current-parent and released producer identities are verified separately; this is not candidate-source qualification.",
  custody,
  baseline: { ...primary.publicBefore, binary, verifiedBinaryBeforeSha256: binaryBefore },
  config: { text: config, sha256: sha256(config) },
  priorAttempts: prior("released-before.json"),
  runs: [],
};
const official = {
  schema: "vize.lint.public-vue-imports-official-provider",
  version: 1,
  authority:
    "Complete pinned ESLint lintText results checked against source-authored provider vectors, including fixes, empty suppressed messages and all counts. Native/provider span, severity, string and unfinished producer differences remain explicit.",
  custody,
  provider: primary.provider,
  verifiedPackagesBefore: packagesBefore,
  priorAttempts: prior("official-provider.json"),
  runs: [],
};
const fixedBefore = {
  schema: "vize.lint.public-vue-imports-released-fix-before",
  version: 1,
  authority:
    "Six old released --fix whole packets and final bytes checked against separately pre-authored unconditional-old-producer contracts; no candidate observation.",
  custody,
  baseline: released.baseline,
  config: released.config,
  contractSha256: sha256(fixBytes),
  priorAttempts: prior("released-fix-before.json"),
  runs: [],
};
const failures = [];
function compare(row, assertions) {
  try {
    assertions();
    row.matched = true;
  } catch (error) {
    row.matched = false;
    row.mismatch = { name: error.name, message: error.message };
    failures.push(row.id);
  }
}
function persist() {
  fs.writeFileSync(
    path.join(fixture, "released-before.json"),
    JSON.stringify(released, null, 2) + "\n",
  );
  fs.writeFileSync(
    path.join(fixture, "released-fix-before.json"),
    JSON.stringify(fixedBefore, null, 2) + "\n",
  );
  fs.writeFileSync(
    path.join(fixture, "official-provider.json"),
    JSON.stringify(official, null, 2) + "\n",
  );
}
try {
  for (const entry of corpus.cases) {
    const bytes = read(entry.sourceFile);
    assert.equal(bytes.toString("utf8"), entry.source);
    assert.equal(sha256(bytes), entry.sourceSha256);
    const filename = path.join(directory, entry.filename);
    fs.writeFileSync(filename, bytes);
    const input = { source: entry.source, sha256: sha256(bytes) };
    const argv = [
      "lint",
      "-c",
      "vize.config.json",
      "--format",
      "json",
      "--locale",
      "en",
      "--help-level",
      "none",
      entry.filename,
    ];
    const run = spawnSync(binary, argv, {
      cwd: directory,
      timeout: 60_000,
      maxBuffer: 8 * 1024 * 1024,
    });
    const before = {
      id: entry.id,
      input,
      argv,
      expected: wholeJson(entry, entry.currentParentDiagnostics),
      expectedStatus: 0,
      status: run.status,
      signal: run.signal,
      error: run.error ? { name: run.error.name, message: run.error.message } : null,
      stdout: run.stdout?.toString("utf8") ?? "",
      stdoutBase64: run.stdout?.toString("base64") ?? "",
      stderr: run.stderr?.toString("utf8") ?? "",
      stderrBase64: run.stderr?.toString("base64") ?? "",
    };
    released.runs.push(before);
    compare(before, () => {
      assert.equal(before.error, null);
      assert.equal(before.signal, null);
      assert.equal(before.status, before.expectedStatus);
      assert.equal(before.stderrBase64, "");
      before.actual = JSON.parse(before.stdout);
      assert.deepEqual(before.actual, before.expected);
    });
    persist();
    const messages = supplement.cases.find((row) => row.id === entry.id).diagnostics;
    const expected = [
      {
        filePath: filename,
        messages,
        suppressedMessages: [],
        errorCount: messages.filter((finding) => finding.severity === 2).length,
        fatalErrorCount: 0,
        warningCount: messages.filter((finding) => finding.severity === 1).length,
        fixableErrorCount: messages.filter((finding) => finding.severity === 2 && finding.fix)
          .length,
        fixableWarningCount: messages.filter((finding) => finding.severity === 1 && finding.fix)
          .length,
        ...(messages.length ? { source: entry.source } : {}),
        usedDeprecatedRules: [],
      },
    ];
    const row = {
      id: entry.id,
      input,
      expected,
      declaredProviderDifferences: entry.providerDifferences,
    };
    official.runs.push(row);
    persist();
    row.actual = JSON.parse(
      JSON.stringify(await eslint.lintText(entry.source, { filePath: entry.filename })),
    );
    compare(row, () => assert.deepEqual(row.actual, row.expected));
    before.inputAfterSha256 = row.inputAfterSha256 = sha256(fs.readFileSync(filename));
    assert.equal(before.inputAfterSha256, entry.sourceSha256);
    fs.unlinkSync(filename);
    persist();
  }
  for (const contract of fixContract.cases) {
    const entry = corpus.cases.find((row) => row.id === contract.id);
    assert.equal(entry.sourceSha256, contract.sourceSha256);
    const filename = path.join(directory, entry.filename);
    fs.writeFileSync(filename, read(entry.sourceFile));
    const argv = [
      "lint",
      "-c",
      "vize.config.json",
      "--format",
      "json",
      "--locale",
      "en",
      "--help-level",
      "none",
      "--fix",
      entry.filename,
    ];
    const run = spawnSync(binary, argv, {
      cwd: directory,
      timeout: 60_000,
      maxBuffer: 8 * 1024 * 1024,
    });
    const after = fs.readFileSync(filename);
    const row = {
      id: entry.id,
      input: { source: entry.source, sha256: entry.sourceSha256 },
      argv,
      expected: wholeJson({ ...entry, source: contract.fixedSource }, contract.diagnostics),
      expectedStatus: 0,
      expectedFinalSource: contract.fixedSource,
      status: run.status,
      signal: run.signal,
      error: run.error ? { name: run.error.name, message: run.error.message } : null,
      stdout: run.stdout?.toString("utf8") ?? "",
      stdoutBase64: run.stdout?.toString("base64") ?? "",
      stderr: run.stderr?.toString("utf8") ?? "",
      stderrBase64: run.stderr?.toString("base64") ?? "",
      finalSource: after.toString("utf8"),
      finalSourceSha256: sha256(after),
    };
    fixedBefore.runs.push(row);
    compare(row, () => {
      assert.equal(row.error, null);
      assert.equal(row.signal, null);
      assert.equal(row.status, row.expectedStatus);
      assert.equal(row.stderrBase64, "");
      row.actual = JSON.parse(row.stdout);
      assert.deepEqual(row.actual, row.expected);
      assert.equal(row.finalSource, contract.fixedSource);
      assert.equal(row.finalSourceSha256, contract.fixedSourceSha256);
    });
    fs.unlinkSync(filename);
    persist();
  }
  for (const receipt of [released, official, fixedBefore]) {
    receipt.summary = {
      cases: receipt.runs.length,
      matchedCases: receipt.runs.filter((row) => row.matched).length,
      diagnostics: receipt.runs.reduce(
        (sum, row) => sum + (row.actual?.[0]?.messages.length ?? 0),
        0,
      ),
    };
  }
  assert.equal(sha256(read("cases.json")), custody.corpusSha256);
  released.baseline.verifiedBinaryAfterSha256 = sha256(fs.readFileSync(binary));
  assert.equal(released.baseline.verifiedBinaryAfterSha256, binaryBefore);
  official.verifiedPackagesAfter = packageIdentity();
  assert.deepEqual(official.verifiedPackagesAfter, packagesBefore);
  persist();
  console.log(
    JSON.stringify(
      {
        released: released.summary,
        official: official.summary,
        fixedBefore: fixedBefore.summary,
        failedIds: failures,
      },
      null,
      2,
    ),
  );
  assert.equal(
    failures.length,
    0,
    "Reference failures are retained; no expected vectors are recaptured",
  );
} finally {
  persist();
  fs.rmSync(directory, { recursive: true, force: true });
}
