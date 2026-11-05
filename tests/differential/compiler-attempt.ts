import assert from "node:assert/strict";
import { compareBytes } from "./compare.mjs";
import { sha256 } from "./harness.mjs";

interface CompilerResult {
  status: number | null;
  signal: NodeJS.Signals | null;
  stdout?: Buffer | null;
  stderr?: Buffer | null;
  error?: Error | null;
}

interface CompilerOutput {
  filename: unknown;
  code: unknown;
  css: unknown;
  errors: unknown;
  warnings: unknown;
  script_lang: unknown;
  macro_artifacts: unknown;
}

interface CompilerObservation {
  exitStatus: number | null;
  signal: NodeJS.Signals | null;
  stdoutBase64: string;
  stderrBase64: string;
  processError: string | null;
  jsonBase64: string | null;
  jsonSha256: string | null;
  codeBase64: string | null;
  codeSha256: string | null;
  cssBase64: string | null;
  cssSha256: string | null;
  referenceComparison: ReturnType<typeof compareBytes> | null;
  cssReferenceComparison: ReturnType<typeof compareBytes> | null;
  facetError: string | null;
}

export function compilerAttempt(
  result: CompilerResult,
  output: Buffer | null,
  expected: Buffer,
  expectedScriptLang = "js",
  expectedCss: Buffer | null = null,
): CompilerObservation {
  const observation: CompilerObservation = {
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
      const parsed = JSON.parse(output.toString("utf8")) as CompilerOutput;
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
      observation.facetError = error instanceof Error ? error.message : String(error);
    }
  }
  return observation;
}
