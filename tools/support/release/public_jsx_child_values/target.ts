/** Explicit source-bound targets; no version-based reinterpretation of older public evidence. */
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
import { read, type Custody } from "./contract.ts";

export const vaporNine = "vapor-nine-v1";
export const nativeVdomTen = "native-vdom-ten-v1";
export const nativeVdomMerge = "f233823358c1242013b51e53680ce257253c84fc";
export type Target = typeof vaporNine | typeof nativeVdomTen;

export function qualificationTarget(value: string | undefined): Target {
  if (value === undefined) return vaporNine;
  assert(value === vaporNine || value === nativeVdomTen, "unknown public JSX qualification target");
  return value;
}

export function targetCustody(target: Target): Custody {
  const original = read<Custody>(fileURLToPath(new URL("./source-custody.json", import.meta.url)));
  if (target === vaporNine) return original;
  assert.equal(target, nativeVdomTen);
  const vdom = read<Custody>(
    fileURLToPath(new URL("./native-vdom-source-custody.json", import.meta.url)),
  );
  assert.equal(vdom.merge, nativeVdomMerge);
  assert.deepEqual(vdom.referenceDependencies, original.referenceDependencies);
  for (const [name, digest] of Object.entries(original.pins)) assert.equal(vdom.pins[name], digest);
  assert.deepEqual(
    Object.keys(vdom.pins).sort(),
    [
      ...Object.keys(original.pins),
      "tests/tooling/jsx-vapor-oracle/public-vdom-child-browser.ts",
    ].sort(),
  );
  return vdom;
}

type Original = { expectedText: string[]; expectedElementIdentities: number[][] };
type Trace = {
  phases: { text: string; elementIdentities: number[]; warnings: string[]; reads: number }[];
  readsAfterUnmount: number;
  completeUnmount: boolean;
};
function completeTrace(value: unknown, original: Original): Trace {
  assert(value && typeof value === "object");
  const trace = value as Trace;
  assert.equal(original.expectedText.length, 10);
  assert.equal(original.expectedElementIdentities.length, 10);
  assert.equal(trace.phases.length, 10);
  assert.equal(trace.readsAfterUnmount, 10);
  assert.equal(trace.completeUnmount, true);
  trace.phases.forEach((phase, index) => {
    assert.equal(phase.text, original.expectedText[index]);
    assert.deepEqual(phase.elementIdentities, original.expectedElementIdentities[index]);
    assert.deepEqual(phase.warnings, []);
    assert.equal(phase.reads, index + 1);
  });
  return trace;
}

export function qualifyVdomNode(value: unknown, original: Original): void {
  assert(value && typeof value === "object");
  const runtime = value as { routes: Record<string, unknown>; receipt: Record<string, unknown> };
  const reference = completeTrace(runtime.routes["public-babel"], original);
  const native = completeTrace(runtime.routes["source-vdom"], original);
  assert.deepEqual(native, reference, "complete installed native VDOM and original Babel traces");
  assert.deepEqual(runtime.receipt.vdomMatchesBabel, Array(10).fill(true));
  assert.equal(runtime.receipt.qualifiedVaporPhases, 9);
  assert.equal(runtime.receipt.referenceConflictPhases, 1);
  assert.equal(runtime.receipt.wholeCorpusQualified, false);
  assert.equal(runtime.receipt.issue6890Complete, false);
}

export function qualifyVdomBrowser(value: unknown, original: Original): void {
  assert(value && typeof value === "object");
  const browser = value as {
    routes: Record<string, unknown>;
    browserVersion: string;
    wholeCorpusQualified: boolean;
    issue6890Complete: boolean;
  };
  assert.equal(typeof browser.browserVersion, "string");
  assert(browser.browserVersion.length > 0);
  assert.deepEqual(Object.keys(browser.routes).sort(), [
    "native",
    "native-shadow",
    "reference",
    "reference-shadow",
  ]);
  for (const suffix of ["", "-shadow"]) {
    const reference = completeTrace(browser.routes["reference" + suffix], original);
    const native = completeTrace(browser.routes["native" + suffix], original);
    assert.deepEqual(native, reference, "complete genuine Chromium VDOM/Babel trace equality");
  }
  assert.equal(browser.wholeCorpusQualified, false);
  assert.equal(browser.issue6890Complete, false);
}
