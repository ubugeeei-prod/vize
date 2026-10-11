import assert from "node:assert/strict";

/** Original 56 typed controls and 80 + 144 default/migration consumer vectors. */
export function scenarios({ typed, first, second, projection }) {
  const typedCases = typed.cases.flatMap((entry) => [
    ...[null, "recommended", "essential", "ecosystem"].map((preset) => ({
      entry,
      preset,
      config: null,
      findings: entry.diagnostics,
      suite: "typed",
    })),
    { entry, preset: "incremental", config: null, findings: [], suite: "typed" },
    {
      entry,
      preset: null,
      config: JSON.stringify({ linter: { rules: { [typed.rule]: "off" } } }) + "\n",
      findings: [],
      suite: "typed",
    },
    {
      entry,
      preset: null,
      config: JSON.stringify({ linter: projection.linter }) + "\n",
      findings: [],
      suite: "typed",
    },
  ]);
  const compatibility = first.cases
    .filter((entry) => entry.kind === "vue")
    .flatMap((entry) => [
      { entry, preset: null, config: null, findings: entry.diagnostics, suite: "first" },
      { entry, preset: "recommended", config: null, findings: entry.diagnostics, suite: "first" },
      { entry, preset: "ecosystem", config: null, findings: entry.diagnostics, suite: "first" },
      { entry, preset: "incremental", config: null, findings: [], suite: "first" },
    ]);
  for (const entry of first.cases.slice(0, 6)) {
    compatibility.push({
      entry,
      preset: null,
      findings: entry.baselineDiagnostics,
      suite: "first",
      config:
        JSON.stringify({
          linter: { rules: Object.fromEntries(first.promotedRules.map((name) => [name, "off"])) },
        }) + "\n",
    });
  }
  for (const version of first.configuredVueVersions) {
    for (const entry of first.cases.slice(0, 6))
      compatibility.push({
        entry,
        preset: null,
        suite: "first",
        config: JSON.stringify({ vue: { version } }) + "\n",
        findings: version === "3" ? entry.diagnostics : entry.baselineDiagnostics,
      });
  }
  assert.equal(compatibility.length, 80);
  for (const entry of second.cases) {
    compatibility.push(
      ...[null, "recommended", "essential", "ecosystem", "opinionated", "nuxt"].map((preset) => ({
        entry,
        preset,
        config: null,
        findings: preset === "essential" ? entry.essentialDiagnostics : entry.diagnostics,
        suite: "second",
      })),
      { entry, preset: "incremental", config: null, findings: [], suite: "second" },
      {
        entry,
        preset: null,
        suite: "second",
        findings: entry.baselineDiagnostics,
        config:
          JSON.stringify({
            linter: {
              rules: Object.fromEntries(second.promotedRules.map((name) => [name, "off"])),
            },
          }) + "\n",
      },
      ...second.configuredVueVersions.map((version) => ({
        entry,
        preset: null,
        suite: "second",
        config: JSON.stringify({ vue: { version } }) + "\n",
        findings: version === "3" ? entry.diagnostics : entry.baselineDiagnostics,
      })),
      {
        entry,
        preset: null,
        suite: "second",
        config: JSON.stringify({ linter: projection.linter }) + "\n",
        findings: entry.essentialDiagnostics,
      },
    );
  }
  assert.equal(typedCases.length, 56);
  assert.equal(compatibility.length, 224);
  return [...typedCases, ...compatibility];
}
