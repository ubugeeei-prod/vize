import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { gunzipSync, gzipSync } from "node:zlib";
import { sha256, wholeJson } from "../../../../tooling/lint-default-correctness-reference.mjs";

// Expectations are frozen in cases.json before this observer executes. This
// helper records observations and checks them; it never rewrites those vectors.
const fixture = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(fixture, "../../../../..");
const primary = JSON.parse(fs.readFileSync(path.join(fixture, "primary-sources.json"), "utf8"));
const corpusBytes = fs.readFileSync(path.join(fixture, "cases.json"));
const corpus = JSON.parse(corpusBytes);
const originalCaseBytes = fs.readFileSync(path.join(fixture, primary.realFile.expectationsPath));
assert.equal(sha256(originalCaseBytes), primary.realFile.expectationsSha256);
const originalCase = JSON.parse(originalCaseBytes);
const originalEntry = {
  ...originalCase,
  source: fs.readFileSync(path.join(fixture, originalCase.sourceFile), "utf8"),
};
const binary = process.env.VIZE_PUBLIC_BINARY;
const providerRoot = process.env.VIZE_PROVIDER_ROOT;
assert.ok(binary, "VIZE_PUBLIC_BINARY must name the authenticated v0.443.0 binary");
assert.ok(providerRoot, "VIZE_PROVIDER_ROOT must name an isolated pinned package installation");
assert.equal(sha256(corpusBytes), primary.corpus.sha256);
assert.equal(sha256(fs.readFileSync(fileURLToPath(import.meta.url))), primary.observer.sha256);
assert.equal(
  sha256(fs.readFileSync(path.join(root, primary.observer.wholeJson.path))),
  primary.observer.wholeJson.sha256,
);
assert.equal(sha256(fs.readFileSync(binary)), corpus.beforeBaseline.binarySha256);
assert.equal(corpus.cases.length, 16);
const matrixBytes = fs.readFileSync(path.join(fixture, primary.matrix.path));
assert.equal(sha256(matrixBytes), primary.matrix.sha256);
const originalMatrix = gunzipSync(matrixBytes);
assert.equal(sha256(originalMatrix), primary.matrix.uncompressedSha256);
const matrix = JSON.parse(originalMatrix);
assert.equal(matrix.project, corpus.originalMatrix.project);
assert.equal(matrix.revision, corpus.originalMatrix.revision);
assert.equal(matrix.evidence.commitSha, corpus.originalMatrix.sourceCommit);
assert.equal(matrix.divergence.falsePositives.length, 83);
assert.equal(matrix.divergence.falseNegatives.length, 0);
assert.ok(matrix.divergence.falsePositives.every((finding) => finding.ruleId === corpus.rule));

const requireProvider = createRequire(path.join(providerRoot, "package.json"));
for (const entry of primary.provider.packages) {
  const installed = JSON.parse(
    fs.readFileSync(path.join(providerRoot, "node_modules", entry.name, "package.json"), "utf8"),
  );
  assert.equal(installed.version, entry.version, entry.name);
  const packageRoot = path.join(providerRoot, "node_modules", entry.name);
  function packageFiles(directory) {
    return fs.readdirSync(directory, { withFileTypes: true }).flatMap((item) => {
      if (item.name === "node_modules") return [];
      const filename = path.join(directory, item.name);
      return item.isDirectory() ? packageFiles(filename) : item.isFile() ? [filename] : [];
    });
  }
  const files = packageFiles(packageRoot).sort();
  const tree = files
    .map(
      (filename) =>
        `${sha256(fs.readFileSync(filename))}  ${path.relative(packageRoot, filename).split(path.sep).join("/")}\n`,
    )
    .join("");
  assert.equal(files.length, entry.installedFileCount, entry.name);
  assert.equal(sha256(tree), entry.installedFileTreeSha256, entry.name);
}
for (const entry of primary.provider.files) {
  assert.equal(
    sha256(fs.readFileSync(path.join(providerRoot, entry.path))),
    entry.sha256,
    entry.path,
  );
}
assert.equal(
  sha256(fs.readFileSync(path.join(providerRoot, "package-lock.json"))),
  primary.provider.packageLockSha256,
);
const { ESLint } = requireProvider("eslint");
const vue = requireProvider("eslint-plugin-vue");
const parser = requireProvider("vue-eslint-parser");
const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bare-reserved-observation-"));
const nativeConfig = JSON.stringify(primary.observer.nativeConfig, null, 2) + "\n";
fs.writeFileSync(path.join(directory, "vize.config.json"), nativeConfig);
const eslint = new ESLint({
  cwd: directory,
  overrideConfigFile: true,
  ignore: false,
  overrideConfig: [
    {
      files: ["**/*.vue"],
      languageOptions: {
        parser,
        ecmaVersion: 2022,
        sourceType: "module",
        parserOptions: { ecmaVersion: 2022, sourceType: "module" },
      },
      plugins: { vue },
      rules: { "vue/no-reserved-keys": "error" },
    },
  ],
});
const custody = {
  casesSha256: sha256(corpusBytes),
  primarySourcesSha256: sha256(fs.readFileSync(path.join(fixture, "primary-sources.json"))),
  helperSha256: primary.observer.sha256,
  wholeJsonSha256: primary.observer.wholeJson.sha256,
  originalExpectationsSha256: primary.realFile.expectationsSha256,
};
const previousPath = path.join(fixture, "original-public-before.json");
const previousBytes = fs.existsSync(previousPath) ? fs.readFileSync(previousPath) : null;
const previous = previousBytes ? JSON.parse(previousBytes) : null;
const priorAttempts = previous?.priorAttempts ?? [];
if (previous?.summary && previous.summary.matchedCases !== previous.summary.cases) {
  priorAttempts.push({
    reason: primary.observer.projectionCorrection.reason,
    sha256: sha256(previousBytes),
    encoding: "gzip-base64",
    rawBytes: previousBytes.length,
    completePacket: gzipSync(previousBytes).toString("base64"),
    summary: previous.summary,
    custody: previous.custody,
  });
}
const nativeReceipt = {
  schema: "vize.lint.bare-reserved-public-before",
  version: 1,
  authority:
    "Observed authenticated released artifact; expected complete packets derive from pre-authored beforeDiagnostics via the pinned wholeJson helper. The pinned rule unconditionally attaches help and the pinned JSON serializer retains it, including with help-level none.",
  custody,
  priorAttempts,
  baseline: {
    ...corpus.beforeBaseline,
    binary,
    verifiedBinarySha256: sha256(fs.readFileSync(binary)),
  },
  config: nativeConfig,
  runs: [],
};
const providerReceipt = {
  schema: "vize.lint.bare-reserved-official-provider",
  version: 1,
  authority:
    "Complete ESLint lintText result packets checked against independently authored providerDiagnostics. Runtime output does not supply expectations.",
  custody,
  provider: primary.provider,
  configuration: primary.observer.providerConfig,
  retainedProviderDifferences: corpus.retainedProviderDifferences,
  runs: [],
};
const failures = [];
function compare(observation, assertions) {
  try {
    assertions();
    observation.matched = true;
  } catch (error) {
    observation.matched = false;
    observation.mismatch = { name: error.name, message: error.message };
    failures.push({ id: observation.id, message: error.message });
  }
}
function persist() {
  fs.writeFileSync(
    path.join(fixture, "original-public-before.json"),
    JSON.stringify(nativeReceipt, null, 2) + "\n",
  );
  fs.writeFileSync(
    path.join(fixture, "official-provider.json"),
    JSON.stringify(providerReceipt, null, 2) + "\n",
  );
}
try {
  for (const { entry, realFile } of [
    ...corpus.cases.map((entry) => ({ entry, realFile: false })),
    { entry: originalEntry, realFile: true },
  ]) {
    const bytes = fs.readFileSync(
      path.join(fixture, entry.sourceFile ?? entry.filename + ".fixture"),
    );
    assert.equal(bytes.toString("utf8"), entry.source, entry.id);
    assert.equal(sha256(bytes), entry.sourceSha256, entry.id);
    fs.writeFileSync(path.join(directory, entry.filename), bytes);
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
      encoding: "utf8",
      timeout: 60_000,
      maxBuffer: 4 * 1024 * 1024,
    });
    const expected = wholeJson(entry, entry.beforeDiagnostics);
    const observed = {
      id: entry.id,
      sourceSha256: entry.sourceSha256,
      argv,
      status: run.status,
      signal: run.signal,
      error: run.error ? { name: run.error.name, message: run.error.message } : null,
      stdout: run.stdout ?? "",
      stderr: run.stderr ?? "",
      expected,
      expectedStatus: entry.beforeDiagnostics.some((finding) => finding.severity === "error")
        ? 1
        : 0,
    };
    if (realFile) nativeReceipt.realFile = { ...primary.realFile, observation: observed };
    else nativeReceipt.runs.push(observed);
    compare(observed, () => {
      assert.equal(observed.error, null);
      assert.equal(observed.signal, null);
      assert.equal(observed.status, observed.expectedStatus);
      assert.equal(observed.stderr, "");
      observed.actual = JSON.parse(observed.stdout);
      assert.deepEqual(observed.actual, observed.expected);
    });
    const messages = entry.providerDiagnostics;
    const expectedProvider = [
      {
        filePath: path.join(directory, entry.filename),
        messages,
        suppressedMessages: [],
        errorCount: messages.filter((finding) => finding.severity === 2).length,
        fatalErrorCount: 0,
        warningCount: messages.filter((finding) => finding.severity === 1).length,
        fixableErrorCount: 0,
        fixableWarningCount: 0,
        ...(messages.length ? { source: entry.source } : {}),
        usedDeprecatedRules: [],
      },
    ];
    const official = {
      id: entry.id,
      sourceSha256: entry.sourceSha256,
      expected: expectedProvider,
      nativeAfterDiagnosticCount: entry.diagnostics.length,
      retainedProviderPolicyDifference: entry.diagnostics.length !== messages.length,
    };
    if (realFile) providerReceipt.realFile = { ...primary.realFile, observation: official };
    else providerReceipt.runs.push(official);
    assert.equal(fs.readFileSync(path.join(directory, entry.filename), "utf8"), entry.source);
    official.actual = JSON.parse(
      JSON.stringify(await eslint.lintText(entry.source, { filePath: entry.filename })),
    );
    compare(official, () => assert.deepEqual(official.actual, official.expected));
    fs.unlinkSync(path.join(directory, entry.filename));
    persist();
  }
  nativeReceipt.summary = {
    cases: nativeReceipt.runs.length,
    diagnosticCount: nativeReceipt.runs.reduce(
      (count, run) => count + (run.actual?.[0]?.messages.length ?? 0),
      0,
    ),
    matchedCases: nativeReceipt.runs.filter((run) => run.matched).length,
    realFileMatched: nativeReceipt.realFile.observation.matched,
    realFileDiagnosticCount: nativeReceipt.realFile.observation.actual[0].messages.length,
  };
  providerReceipt.summary = {
    cases: providerReceipt.runs.length,
    diagnosticCount: providerReceipt.runs.reduce(
      (count, run) => count + run.actual[0].messages.length,
      0,
    ),
    matchedCases: providerReceipt.runs.filter((run) => run.matched).length,
    realFileMatched: providerReceipt.realFile.observation.matched,
    realFileDiagnosticCount: providerReceipt.realFile.observation.actual[0].messages.length,
    retainedProviderPolicyDifferenceCases: providerReceipt.runs
      .filter((run) => run.retainedProviderPolicyDifference)
      .map((run) => run.id),
  };
  assert.equal(sha256(fs.readFileSync(path.join(fixture, "cases.json"))), custody.casesSha256);
  persist();
  console.log(
    JSON.stringify(
      {
        native: nativeReceipt.summary,
        provider: providerReceipt.summary,
        failures: failures.map(({ id, message }) => ({ id, message: message.slice(0, 400) })),
      },
      null,
      2,
    ),
  );
  if (failures.length) process.exitCode = 1;
} finally {
  persist();
  fs.rmSync(directory, { recursive: true, force: true });
}
