import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export const cssNamingGate = "tests/tooling/davinci/davinci-atelier-sfc-stage-alias.test.mjs";
export const currentCSSNamingGateSHA =
  "429f0747c4f3c0d9867c929b1f5b01165cd380b21714411c56b11093f40b566c";
export const historicalCSSNamingGateSHA =
  "2d743d27f4328b01a811d7ba0f962fcd4541e528f09e8193d346d23ead9d8921";
const digest = (source: string) => createHash("sha256").update(source).digest("hex");
const originalRow =
  '  ["crates/vize_atelier_sfc/tests/css_engine_panic_boundary.rs", "test", 5],\n';
const currentRows =
  '  ["crates/vize_atelier_sfc/tests/css_engine_panic_boundary.rs", "test", 2],\n' +
  '  ["crates/vize_atelier_sfc/tests/support/css_engine_successors.rs", "test", 1],\n';
const preservedAuthorityLaw =
  "\n  // The finite whole-result law replaces three partial Vec<String> assertions.\n" +
  "  // Keep all five original preferred aliases in the complete archived authority.\n" +
  "  const preservedPath =\n" +
  '    "tests/_fixtures/differential/css-engine-vendor-exit/finite-before/css_engine_panic_boundary.rs.txt";\n' +
  '  const preserved = fs.readFileSync(path.join(repoRoot, preservedPath), "utf8");\n' +
  "  assertPreferredSource(preserved, preservedPath);\n" +
  "  assert.equal(preserved.match(/\\bvize_l0\\b/gu)?.length, 5, preservedPath);\n";

/** Checks-only historical replay; no physical gate or current assertion is rewritten. */
export function cssNamingGateReplayText(file: string, source: string): string;
export function cssNamingGateReplayText(
  file: string,
  source: string | undefined,
): string | undefined;
export function cssNamingGateReplayText(
  file: string,
  source: string | undefined,
): string | undefined {
  if (file !== cssNamingGate || source === undefined || digest(source) !== currentCSSNamingGateSHA)
    return source;
  assert.equal(source.split(currentRows).length, 2, "exact whole current CSS naming rows");
  assert.equal(
    source.split(preservedAuthorityLaw).length,
    2,
    "exact whole archived CSS naming law",
  );
  const historical = source.replace(currentRows, originalRow).replace(preservedAuthorityLaw, "");
  assert.equal(
    digest(historical),
    historicalCSSNamingGateSHA,
    "complete historical SFC naming gate bytes",
  );
  return historical;
}
