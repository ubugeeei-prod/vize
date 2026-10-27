import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { expectedBuildIdentity, validateBuildReceipt } from "./build-receipt.ts";
import { compareBytes } from "./compare.mjs";
import { compilerCssApiReport, loadCompilerCssApiCases } from "./compiler-css-api.ts";
import { closedSetupReference } from "../tooling/support/script-setup-default-expose.ts";
import {
  loadProductManifest,
  readPinnedArtifact,
  runPlannedCases,
  sha256,
  validateResultEnvelope,
} from "./harness.mjs";

export const COMPILER_ARGV = [
  "build",
  "Layout.vue",
  "--format",
  "json",
  "--output",
  "out",
  "--ssr",
  "--no-config",
];
const DIALECT_INVENTORY_REF = "tests/_fixtures/differential/dialect-inputs/inventory.json";
const DIALECT_TEMPLATE_TRANSPORT = "sfc-template-wrapper-v1";
const DIALECT_TEMPLATE_PREFIX = Buffer.from("<template>\n");
const DIALECT_TEMPLATE_SUFFIX = Buffer.from("</template>\n");
const DIALECT_VITE_CONFIG_PATH = "vite.config.mjs";
const OUTPUT_FILE = "out/Layout.json";
const NOT_COMPARED = "whole-product native compiler path unavailable";

export function compilerArgv(fixture) {
  if (!fixture.dialectInput) return [...COMPILER_ARGV];
  const argv = [
    ...COMPILER_ARGV.slice(0, -1),
    "--template-syntax",
    fixture.dialectInput.selector.templateSyntax,
  ];
  return fixture.dialectInput.selector.vueVersion === "3"
    ? [...argv, "--no-config"]
    : [...argv, "--config", DIALECT_VITE_CONFIG_PATH];
}

function dialectViteConfig(registered) {
  if (registered.selector.vueVersion === "3") return null;
  return Buffer.from(`export default { vue: { version: "${registered.selector.vueVersion}" } };\n`);
}

function dialectResultInput(fixture) {
  return {
    id: fixture.dialectInputId,
    sha256: fixture.dialectInputSha256,
    transport: fixture.dialectTransport,
    ...(fixture.dialectConfig
      ? {
          config: {
            path: fixture.dialectConfigPath,
            sha256: fixture.dialectConfigSha256,
          },
        }
      : {}),
  };
}

function loadDialectInput(root, fixture) {
  const registryPath = path.resolve(root, "../dialect-inputs/inventory.json");
  const registry = JSON.parse(fs.readFileSync(registryPath, "utf8"));
  assert.equal(registry.schema, "vize.dialectInputInventory");
  assert.equal(registry.version, 1);
  const registered = registry.cases.find((entry) => entry.id === fixture.inputs.dialectInput.id);
  assert(registered, `unknown dialect input: ${fixture.inputs.dialectInput.id}`);
  assert.deepEqual(fixture.inputs.dialectInput, {
    id: registered.id,
    sha256: registered.input.sha256,
    selector: registered.selector,
    expectedParserMode: registered.expectedParserMode,
    transport: DIALECT_TEMPLATE_TRANSPORT,
  });
  assert.equal(registered.state, "input-only");
  const raw = readPinnedArtifact(path.dirname(registryPath), registered.input);
  const input = Buffer.concat([DIALECT_TEMPLATE_PREFIX, raw, DIALECT_TEMPLATE_SUFFIX]);
  const config = dialectViteConfig(registered);
  const selector = config
    ? `compiler-cli-ssr-template-v1 Vue ${registered.selector.vueVersion} via generated ${DIALECT_VITE_CONFIG_PATH}; --template-syntax ${registered.selector.templateSyntax}; parser ${registered.expectedParserMode}`
    : `compiler-cli-ssr-template-v1 default Vue ${registered.selector.vueVersion}; --template-syntax ${registered.selector.templateSyntax}; parser ${registered.expectedParserMode}`;
  assert.deepEqual(fixture.dialectCoverage, {
    state: "verified",
    registryRef: DIALECT_INVENTORY_REF,
    evidence: [
      {
        kind: "adapter-execution",
        dialects: registered.dialects,
        selector,
      },
    ],
  });
  return { raw, input, config, registered };
}

export function loadCompilerManifest(manifestPath) {
  const loaded = loadProductManifest(manifestPath, "compiler");
  const root = path.dirname(manifestPath);
  const cssApiCases = loadCompilerCssApiCases(loaded.manifest.cssApiCases, (file) => {
    const relative = file.slice("tests/_fixtures/differential/compiler/".length);
    const physical = path.join(root, relative);
    return fs.existsSync(physical) ? fs.readFileSync(physical) : null;
  });
  return {
    ...loaded,
    cssApiCases,
    cases: loaded.cases.map((fixture) => {
      assert.deepEqual(fixture.targets, ["ssr"]);
      assert.equal(fixture.state, "active");
      assert.equal(fixture.adapters.native, null);
      assert.equal(typeof fixture.adapters.reasons.native, "string");
      const isDialectInput = fixture.inputs.dialectInput !== undefined;
      const dialect = isDialectInput ? loadDialectInput(root, fixture) : null;
      if (dialect) {
        assert.equal(fixture.adapters.legacy, "compiler-cli-ssr-template-v1");
        assert.equal(fixture.id, `compiler/${dialect.registered.id}`);
      } else {
        assert.equal(fixture.adapters.legacy, "compiler-cli-ssr-v1");
        assert.match(fixture.inputs.root, /^[a-z0-9-]+$/);
        assert.equal(fixture.id, `compiler/sfc/${fixture.inputs.root}`);
        assert.equal(fixture.inputs.files.length, 1);
        const entry = fixture.inputs.files[0];
        assert.equal(entry.path, "Layout.vue.txt");
        assert.equal(entry.role, "entry");
      }
      const referenceRoot = dialect ? path.basename(dialect.registered.id) : fixture.inputs.root;
      assert.equal(fixture.reference.path, `${referenceRoot}/module.expected.txt`);
      assert.equal(typeof fixture.reference.source, "string");
      assert(fixture.reference.source.length > 0, "compiler reference source is required");
      if (fixture.reference.cssPath !== undefined || fixture.reference.cssSha256 !== undefined) {
        assert.equal(fixture.reference.cssPath, `${fixture.inputs.root}/style.expected.css`);
        assert.match(fixture.reference.cssSha256, /^[a-f0-9]{64}$/);
      }
      assert.equal(fixture.comparison.contract, "compiler-ssr-module-bytes-v1");
      assert(["js", "ts"].includes(fixture.comparison.scriptLang ?? "js"));
      if (!dialect && fixture.dialectCoverage.state !== "unknown") {
        assert.deepEqual(fixture.dialectCoverage, {
          state: "source-inspected",
          registryRef: null,
          evidence: [
            {
              kind: "source-inspection",
              dialects: [
                "vue3-sfc",
                "html-template",
                ...(fixture.comparison.scriptLang === "ts" ? ["ts"] : []),
              ],
              selector: "compiler-cli-ssr-v1 default Vue 3 dialect",
            },
          ],
        });
      }
      assert.deepEqual(fixture.comparison.requiredFacets, [
        "code",
        "css",
        "errors",
        "warnings",
        "macro_artifacts",
      ]);
      const input = dialect
        ? dialect.input
        : readPinnedArtifact(path.join(root, fixture.inputs.root), fixture.inputs.files[0]);
      const archivedExpected = readPinnedArtifact(root, fixture.reference);
      // Authenticate the original reference before applying the reviewed
      // closed-by-default setup contract. Candidate bytes remain untouched.
      const expected = Buffer.from(
        closedSetupReference({ code: archivedExpected.toString("utf8") }, input.toString("utf8"))
          .code,
      );
      const expectedCss = fixture.reference.cssPath
        ? readPinnedArtifact(root, {
            path: fixture.reference.cssPath,
            sha256: fixture.reference.cssSha256,
          })
        : null;
      return {
        ...fixture,
        input,
        expected,
        expectedCss,
        argv: compilerArgv(dialect ? { dialectInput: dialect.registered } : fixture),
        ...(dialect
          ? {
              dialectInput: dialect.raw,
              dialectInputId: dialect.registered.id,
              dialectInputSha256: dialect.registered.input.sha256,
              dialectTransport: DIALECT_TEMPLATE_TRANSPORT,
              dialectConfig: dialect.config,
              dialectConfigPath: dialect.config ? DIALECT_VITE_CONFIG_PATH : null,
              dialectConfigSha256: dialect.config ? sha256(dialect.config) : null,
            }
          : {}),
      };
    }),
  };
}

export function compilerAttempt(
  result,
  output,
  expected,
  expectedScriptLang = "js",
  expectedCss = null,
) {
  const observation = {
    exitStatus: result.status,
    signal: result.signal,
    stdoutBase64: result.stdout?.toString("base64") ?? "",
    stderrBase64: result.stderr?.toString("base64") ?? "",
    processError: result.error?.message ?? null,
    jsonBase64: output?.toString("base64") ?? null,
    jsonSha256: output ? sha256(output) : null,
    codeBase64: null,
    codeSha256: null,
    cssBase64: null,
    cssSha256: null,
    referenceComparison: null,
    cssReferenceComparison: null,
    facetError: null,
  };
  if (output) {
    try {
      const parsed = JSON.parse(output.toString("utf8"));
      assert.deepEqual(Object.keys(parsed), [
        "filename",
        "code",
        "css",
        "errors",
        "warnings",
        "script_lang",
        "macro_artifacts",
      ]);
      assert.equal(parsed.filename, "Layout.vue");
      assert.equal(parsed.script_lang, expectedScriptLang);
      assert.deepEqual(parsed.errors, []);
      assert.deepEqual(parsed.warnings, []);
      assert.deepEqual(parsed.macro_artifacts, []);
      assert.equal(typeof parsed.code, "string", "compiler output code must be a string");
      const code = Buffer.from(parsed.code, "utf8");
      observation.codeBase64 = code.toString("base64");
      observation.codeSha256 = sha256(code);
      observation.referenceComparison = compareBytes(expected, code);
      if (expectedCss === null) {
        assert.equal(parsed.css, null);
      } else {
        assert.equal(typeof parsed.css, "string", "compiler output css must be a string");
        const css = Buffer.from(parsed.css, "utf8");
        observation.cssBase64 = css.toString("base64");
        observation.cssSha256 = sha256(css);
        observation.cssReferenceComparison = compareBytes(expectedCss, css);
      }
    } catch (error) {
      observation.facetError = error.message;
    }
  }
  return observation;
}

export function validateCompilerReport(loaded, report, expectedBuild) {
  validateResultEnvelope(loaded, report, expectedBuild.sourceRevision);
  assert.deepEqual(report.cssApi, compilerCssApiReport(loaded.cssApiCases));
  assert.deepEqual(report.argv, COMPILER_ARGV);
  if (report.buildReceipt) validateBuildReceipt(report.buildReceipt, expectedBuild);
  if (report.binary) {
    assert(report.buildReceipt, "executable observation requires a build receipt");
    assert.equal(report.binary.path, expectedBuild.binaryPath);
    assert.equal(report.binary.sha256, expectedBuild.binarySha256);
    assert.equal(report.binary.version, expectedBuild.cliVersion);
  }
  const summary = {
    plannedCases: loaded.cases.length,
    legacyMatches: 0,
    legacyFailures: 0,
    baselineDrift: 0,
    nativeUnsupported: loaded.cases.length,
    pairedComparisons: 0,
    nativeHandled: 0,
    nativeEquivalent: 0,
  };
  for (const [index, fixture] of loaded.cases.entries()) {
    const row = report.rows[index];
    assert.equal(row.id, fixture.id);
    assert.equal(row.target, "ssr");
    assert.deepEqual(row.native, { state: "unsupported", reason: fixture.adapters.reasons.native });
    assert.deepEqual(row.comparison, { state: "not-compared", reason: NOT_COMPARED });
    assert.deepEqual(row.legacy.argv, fixture.argv);
    assert.equal(row.legacy.inputSha256, sha256(fixture.input));
    assert.deepEqual(
      row.legacy.dialectInput,
      fixture.dialectInput ? dialectResultInput(fixture) : undefined,
    );
    assert(["completed", "failed"].includes(row.legacy.state));
    const attempt = row.legacy.observation;
    if (attempt) {
      assert.equal(typeof attempt.stdoutBase64, "string");
      assert.equal(typeof attempt.stderrBase64, "string");
      const json = attempt.jsonBase64 === null ? null : Buffer.from(attempt.jsonBase64, "base64");
      assert.equal(json ? sha256(json) : null, attempt.jsonSha256);
      const replay = compilerAttempt(
        {
          status: attempt.exitStatus,
          signal: attempt.signal,
          stdout: Buffer.from(attempt.stdoutBase64, "base64"),
          stderr: Buffer.from(attempt.stderrBase64, "base64"),
          error: attempt.processError ? new Error(attempt.processError) : null,
        },
        json,
        fixture.expected,
        fixture.comparison.scriptLang,
        fixture.expectedCss,
      );
      assert.deepEqual(attempt, replay, "compiler observation or byte comparison was forged");
    }
    if (row.legacy.state === "completed") {
      assert(report.binary, "completed observation requires executable identity");
      assert(attempt, "completed observation is required");
      assert.equal(attempt.exitStatus, 0);
      assert.equal(attempt.signal, null);
      assert.equal(attempt.processError, null);
      assert.equal(attempt.facetError, null, "invalid compiler facets cannot complete");
      assert.equal(
        row.legacy.verdict,
        attempt.referenceComparison.state === "equal" &&
          (attempt.cssReferenceComparison === null ||
            attempt.cssReferenceComparison.state === "equal")
          ? "matched-reference"
          : "baseline-drift",
      );
      summary[row.legacy.verdict === "matched-reference" ? "legacyMatches" : "baselineDrift"] += 1;
    } else {
      assert.equal(row.legacy.verdict, "failed");
      assert.equal(typeof row.legacy.error, "string");
      if (attempt?.facetError) assert.equal(row.legacy.error, attempt.facetError);
      summary.legacyFailures += 1;
    }
  }
  assert.deepEqual(report.summary, summary);
  return summary;
}

function runCompilerCase(fixture, binaryPath, binaryFailure) {
  const row = {
    id: fixture.id,
    target: "ssr",
    legacy: {
      state: "failed",
      verdict: "failed",
      argv: fixture.argv,
      inputSha256: sha256(fixture.input),
      ...(fixture.dialectInput
        ? {
            dialectInput: dialectResultInput(fixture),
          }
        : {}),
      observation: null,
    },
    native: { state: "unsupported", reason: fixture.adapters.reasons.native },
    comparison: { state: "not-compared", reason: NOT_COMPARED },
  };
  let workspace;
  try {
    if (binaryFailure) throw new Error(binaryFailure);
    workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-differential-compiler-"));
    fs.writeFileSync(path.join(workspace, "Layout.vue"), fixture.input);
    if (fixture.dialectConfig) {
      fs.writeFileSync(path.join(workspace, fixture.dialectConfigPath), fixture.dialectConfig);
    }
    const result = spawnSync(binaryPath, fixture.argv, {
      cwd: workspace,
      timeout: 30_000,
      maxBuffer: 4 * 1024 * 1024,
    });
    const output = fs.existsSync(path.join(workspace, OUTPUT_FILE))
      ? fs.readFileSync(path.join(workspace, OUTPUT_FILE))
      : null;
    row.legacy.observation = compilerAttempt(
      result,
      output,
      fixture.expected,
      fixture.comparison.scriptLang,
      fixture.expectedCss,
    );
    if (row.legacy.observation.facetError) throw new Error(row.legacy.observation.facetError);
    if (result.error) throw result.error;
    assert.equal(result.status, 0, result.stderr?.toString());
    assert(output, "compiler JSON output is missing");
    row.legacy.state = "completed";
    row.legacy.verdict =
      row.legacy.observation.referenceComparison.state === "equal" &&
      (row.legacy.observation.cssReferenceComparison === null ||
        row.legacy.observation.cssReferenceComparison.state === "equal")
        ? "matched-reference"
        : "baseline-drift";
  } catch (error) {
    row.legacy.error = error.message;
  } finally {
    if (workspace) fs.rmSync(workspace, { recursive: true, force: true });
  }
  return row;
}

export function runCompilerPack({ manifestPath, binaryPath, sourceRevision, repoRoot }) {
  const loaded = loadCompilerManifest(manifestPath);
  assert.match(sourceRevision, /^[a-f0-9]{40}$/);
  const report = {
    schema: "vize.differential.result",
    version: 1,
    product: "compiler",
    manifestSha256: loaded.manifestSha256,
    sourceRevision,
    argv: COMPILER_ARGV,
    buildReceipt: null,
    binary: null,
    rows: [],
    cssApi: compilerCssApiReport(loaded.cssApiCases),
    summary: null,
  };
  let build = { sourceRevision };
  let binaryFailure;
  try {
    assert.equal(typeof binaryPath, "string", "explicit source-built executable is required");
    assert(path.isAbsolute(binaryPath), "executable path must be absolute");
    assert(fs.statSync(binaryPath).isFile());
    fs.accessSync(binaryPath, fs.constants.X_OK);
    build = expectedBuildIdentity(repoRoot);
    assert.equal(
      build.sourceRevision,
      sourceRevision,
      "checkout revision changed before execution",
    );
    assert.equal(
      fs.realpathSync(binaryPath),
      fs.realpathSync(path.join(repoRoot, build.binaryPath)),
    );
    const receipt = JSON.parse(fs.readFileSync(`${binaryPath}.differential-build.json`, "utf8"));
    validateBuildReceipt(receipt, build);
    const version = spawnSync(binaryPath, ["--version"], { timeout: 30_000 });
    assert.equal(version.status, 0, version.stderr?.toString());
    assert.equal(version.stdout.toString().trim(), build.cliVersion);
    report.buildReceipt = receipt;
    report.binary = {
      path: build.binaryPath,
      sha256: build.binarySha256,
      version: build.cliVersion,
    };
  } catch (error) {
    binaryFailure = error.message;
  }
  report.rows = runPlannedCases(loaded, {
    runCase: (fixture) => [runCompilerCase(fixture, binaryPath, binaryFailure)],
  });
  report.summary = {
    plannedCases: loaded.cases.length,
    legacyMatches: report.rows.filter((row) => row.legacy.verdict === "matched-reference").length,
    legacyFailures: report.rows.filter((row) => row.legacy.verdict === "failed").length,
    baselineDrift: report.rows.filter((row) => row.legacy.verdict === "baseline-drift").length,
    nativeUnsupported: report.rows.length,
    pairedComparisons: 0,
    nativeHandled: 0,
    nativeEquivalent: 0,
  };
  validateCompilerReport(loaded, report, build);
  return report;
}
