/**
 * Detect a `vue-tsc` baseline that never loaded the project it was asked to
 * check (#3513).
 *
 * `typecheck-baseline-project.mjs` materializes the baseline as `files: [...]`
 * over Vize's exact Vue corpus, extending the fixture's pinned config for its
 * compiler options. That fixed the "solution-style config checks nothing" half
 * of the ledger's blindness, and it moved the remaining half out of reach of
 * `typecheck-baseline-coverage.mjs`: `--listFiles` prints every entry of `files`
 * whether or not the extended config resolved, so file coverage now matches by
 * construction and can no longer tell a loaded project from a broken one.
 *
 * What survives that is a configuration failure. Against real vue-tsc 3.3.4 /
 * TypeScript 6.0.3 it takes two shapes, and the comparator dropped both:
 *
 *   error TS5083: Cannot read file '<root>/.nuxt/tsconfig.json'.
 *   tsconfig.json(2,18): error TS6053: File '<root>/.nuxt/tsconfig.app.json' not found.
 *
 * The first has no position, so it was counted into `baselineExcludedProjectCount`
 * and discarded. The second is reported against the config file, so it was
 * counted into `baselineExcludedNonVueCount` and became indistinguishable from an
 * ordinary `.ts` diagnostic. Either way `tsc` carries on with whatever options it
 * did manage to resolve — no `strict`, no `paths`, no `types` — checks the files
 * anyway, and the ledger scores that against Vize as a real measurement.
 *
 * So this is deliberately narrow: only a diagnostic with no file at all, or one
 * reported against a `tsconfig`/`jsconfig` JSON file, is a configuration
 * diagnostic. Misskey's 94 excluded non-Vue diagnostics are `.ts` source errors
 * and stay scoreable; a fixture whose baseline could not read its own config
 * does not.
 */

const diagnosticPattern = /^(.+)\((\d+),(\d+)\): (error|warning) TS(\d+): (.+)$/u;
const projectPattern = /^(error|warning) TS(\d+): (.+)$/u;
const configurationFilePattern = /[jt]sconfig[^/]*\.json$/u;

export function evaluateBaselineConfiguration(vueTscOutput) {
  if (typeof vueTscOutput !== "string") throw new Error("vue-tsc output must be a string");
  const diagnostics = [];
  for (const line of vueTscOutput.replaceAll("\r\n", "\n").split("\n")) {
    const positioned = diagnosticPattern.exec(line);
    if (positioned != null) {
      const file = positioned[1].replaceAll("\\", "/");
      if (!configurationFilePattern.test(file)) continue;
      diagnostics.push({
        code: Number(positioned[5]),
        column: Number(positioned[3]),
        file,
        line: Number(positioned[2]),
        message: normalize(positioned[6]),
        severity: positioned[4],
      });
      continue;
    }
    const project = projectPattern.exec(line);
    if (project == null) continue;
    diagnostics.push({
      code: Number(project[2]),
      column: null,
      file: null,
      line: null,
      message: normalize(project[3]),
      severity: project[1],
    });
  }
  const errors = diagnostics.filter((diagnostic) => diagnostic.severity === "error");
  // Warnings are recorded but do not gate: they did not change which files the
  // baseline checked, and a gate that fires on them is a gate nobody can keep green.
  const unusableReason =
    errors.length === 0
      ? null
      : `vue-tsc could not load the fixture project configuration (${errors.length} ` +
        `error${errors.length === 1 ? "" : "s"}): ${render(errors[0])}`;
  return {
    diagnostics,
    errorCount: errors.length,
    unusableReason,
    verdict: unusableReason == null ? "usable" : "unusable",
  };
}

function render(diagnostic) {
  const position =
    diagnostic.file == null ? "" : `${diagnostic.file}(${diagnostic.line},${diagnostic.column}): `;
  return `${position}${diagnostic.severity} TS${diagnostic.code}: ${diagnostic.message}`;
}

function normalize(message) {
  return message.replace(/\s+/gu, " ").trim();
}
