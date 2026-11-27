/** Restore only the reviewed runner allocation before older source custody.
 * This layer grants no compiled, native or performance execution credit.
 * Current runner policy is checked directly from raw workflows in its own test.
 */
import assert from "node:assert/strict";

export const sourceRunnerChanges = [
  {
    "workflow": ".github/workflows/pr-rust-differential.yml",
    "job": "differential",
    "before": "blacksmith-32vcpu-ubuntu-2404",
    "after": "blacksmith-16vcpu-ubuntu-2404",
  },
  {
    "workflow": ".github/workflows/pr-js-checks.yml",
    "job": "pr-js-package-tests",
    "before": "blacksmith-32vcpu-ubuntu-2404",
    "after": "blacksmith-16vcpu-ubuntu-2404",
  },
  {
    "workflow": ".github/workflows/pr-js-checks.yml",
    "job": "pr-oxlint-project-qualification",
    "before": "blacksmith-32vcpu-ubuntu-2404",
    "after": "blacksmith-16vcpu-ubuntu-2404",
  },
  {
    "workflow": ".github/workflows/pr-js-checks.yml",
    "job": "pr-js-package-conformance",
    "before": "blacksmith-32vcpu-ubuntu-2404",
    "after": "blacksmith-16vcpu-ubuntu-2404",
  },
  {
    "workflow": ".github/workflows/pr-rust-products.yml",
    "job": "pr-rust-shard",
    "before": "blacksmith-32vcpu-ubuntu-2404",
    "after": "blacksmith-16vcpu-ubuntu-2404",
  },
  {
    "workflow": ".github/workflows/pr-source-checks.yml",
    "job": "wasm-release-check",
    "before":
      "${{ contains(fromJSON(needs.pr-source-plan.outputs.rust-plan).packages, 'vize_vitrine') && 'blacksmith-32vcpu-ubuntu-2404' || 'ubuntu-24.04' }}",
    "after":
      "${{ contains(fromJSON(needs.pr-source-plan.outputs.rust-plan).packages, 'vize_vitrine') && 'blacksmith-16vcpu-ubuntu-2404' || 'ubuntu-24.04' }}",
  },
  {
    "workflow": ".github/workflows/pr-source-checks.yml",
    "job": "affected-tooling",
    "before": "blacksmith-32vcpu-ubuntu-2404",
    "after": "blacksmith-16vcpu-ubuntu-2404",
  },
  {
    "workflow": ".github/workflows/pr-source-checks.yml",
    "job": "ci-selection-tests",
    "before":
      "${{ needs.pr-source-plan.outputs.tooling == 'true' && 'blacksmith-32vcpu-ubuntu-2404' || 'ubuntu-24.04' }}",
    "after":
      "${{ needs.pr-source-plan.outputs.tooling == 'true' && 'blacksmith-16vcpu-ubuntu-2404' || 'ubuntu-24.04' }}",
  },
  {
    "workflow": ".github/workflows/pr-source-checks.yml",
    "job": "pr-native-runtime",
    "before": "${{ matrix.lane != 'none' && 'blacksmith-32vcpu-ubuntu-2404' || 'ubuntu-24.04' }}",
    "after": "${{ matrix.lane != 'none' && 'blacksmith-16vcpu-ubuntu-2404' || 'ubuntu-24.04' }}",
  },
  {
    "workflow": ".github/workflows/pr-source-checks.yml",
    "job": "pr-playground-test",
    "before":
      "${{ needs.pr-source-plan.outputs.playground == 'true' && 'blacksmith-32vcpu-ubuntu-2404' || 'ubuntu-24.04' }}",
    "after":
      "${{ needs.pr-source-plan.outputs.playground == 'true' && 'blacksmith-16vcpu-ubuntu-2404' || 'ubuntu-24.04' }}",
  },
  {
    "workflow": ".github/workflows/pr-rust-checks.yml",
    "job": "pr-rust-build",
    "before": "blacksmith-32vcpu-ubuntu-2404",
    "after": "blacksmith-16vcpu-ubuntu-2404",
  },
  {
    "workflow": ".github/workflows/check.yml",
    "job": "nix-flake",
    "before": "blacksmith-32vcpu-ubuntu-2404",
    "after": "blacksmith-16vcpu-ubuntu-2404",
  },
  {
    "workflow": ".github/workflows/check.yml",
    "job": "fmt-rust",
    "before": "blacksmith-32vcpu-ubuntu-2404",
    "after": "blacksmith-16vcpu-ubuntu-2404",
  },
  {
    "workflow": ".github/workflows/check.yml",
    "job": "check-js",
    "before": "blacksmith-32vcpu-ubuntu-2404",
    "after": "blacksmith-16vcpu-ubuntu-2404",
  },
  {
    "workflow": ".github/workflows/check.yml",
    "job": "semver-checks",
    "before": "blacksmith-32vcpu-ubuntu-2404",
    "after": "blacksmith-16vcpu-ubuntu-2404",
  },
  {
    "workflow": ".github/workflows/check.yml",
    "job": "security-audit",
    "before":
      "${{ needs.iteration-plan.outputs.dependencies == 'true' && 'blacksmith-32vcpu-ubuntu-2404' || 'ubuntu-24.04' }}",
    "after":
      "${{ needs.iteration-plan.outputs.dependencies == 'true' && 'blacksmith-16vcpu-ubuntu-2404' || 'ubuntu-24.04' }}",
  },
  {
    "workflow": ".github/workflows/check.yml",
    "job": "node-engine-compat",
    "before":
      "${{ needs.iteration-plan.outputs.engines == 'true' && 'blacksmith-32vcpu-ubuntu-2404' || 'ubuntu-24.04' }}",
    "after":
      "${{ needs.iteration-plan.outputs.engines == 'true' && 'blacksmith-16vcpu-ubuntu-2404' || 'ubuntu-24.04' }}",
  },
  {
    "workflow": ".github/workflows/check.yml",
    "job": "check-vize-apps",
    "before":
      "${{ needs.iteration-plan.outputs.apps == 'true' && 'blacksmith-32vcpu-ubuntu-2404' || 'ubuntu-24.04' }}",
    "after":
      "${{ needs.iteration-plan.outputs.apps == 'true' && 'blacksmith-16vcpu-ubuntu-2404' || 'ubuntu-24.04' }}",
  },
  {
    "workflow": ".github/workflows/check.yml",
    "job": "vue-parity",
    "before": "blacksmith-32vcpu-ubuntu-2404",
    "after": "blacksmith-16vcpu-ubuntu-2404",
  },
  {
    "workflow": ".github/workflows/check.yml",
    "job": "full-js-report",
    "before": "blacksmith-32vcpu-ubuntu-2404",
    "after": "blacksmith-16vcpu-ubuntu-2404",
  },
  {
    "workflow": ".github/workflows/check.yml",
    "job": "coverage",
    "before": "blacksmith-32vcpu-ubuntu-2404",
    "after": "blacksmith-16vcpu-ubuntu-2404",
  },
  {
    "workflow": ".github/workflows/check.yml",
    "job": "source-coverage",
    "before": "blacksmith-32vcpu-ubuntu-2404",
    "after": "blacksmith-16vcpu-ubuntu-2404",
  },
] as const;

/** Unowned workflows and every field other than these exact selectors pass through. */
export function sourceRunnersBefore(file: string, source: string): string {
  const rows = sourceRunnerChanges.filter((row) => row.workflow === file);
  if (rows.length === 0) return source;
  assert.equal([...source.matchAll(/^jobs:\n/gm)].length, 1, `one jobs map: ${file}`);
  for (const row of rows) {
    assert.equal(
      row.after,
      row.before.replaceAll("blacksmith-32vcpu-ubuntu-2404", "blacksmith-16vcpu-ubuntu-2404"),
    );
    const header = `  ${row.job}:\n`;
    assert.equal(source.split(header).length, 2, `one recorded job: ${file}/${row.job}`);
    const start = source.indexOf(header) + header.length;
    assert.ok(start > source.indexOf("jobs:\n"));
    const following = /^  [^ \t#][^\n]*:\n/m.exec(source.slice(start));
    const end = following ? start + following.index : source.length;
    const job = source.slice(start, end);
    const selectors = [...job.matchAll(/^    runs-on: (.+)$/gm)];
    assert.equal(selectors.length, 1, `one runner selector: ${file}/${row.job}`);
    const selector = selectors[0]!;
    assert.ok(
      selector[1] === row.before || selector[1] === row.after,
      `unowned runner: ${file}/${row.job}`,
    );
    const previous = job.replace(selector[0], `    runs-on: ${row.before}`);
    source = source.slice(0, start) + previous + source.slice(end);
  }
  return source;
}
