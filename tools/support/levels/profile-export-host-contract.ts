export const core = "davinci/vize_l0/src/profiler.rs";
export const host = "crates/vize_carton/src/lib.rs";
export const exportFile = "crates/vize_carton/src/profile_export.rs";
export const exportTests = "crates/vize_carton/src/profile_export/tests.rs";
export const moves: [string, string][] = [
  ["davinci/vize_l0/src/profiler/export.rs", exportFile],
  ["davinci/vize_l0/src/profiler/tests/export.rs", exportTests],
];
export const removedExports = `pub use export::{
    PROFILE_EXPORT_SCHEMA_VERSION, ProfileExport, ProfileExportAllocCounts,
    ProfileExportAllocation, ProfileExportAttribution, ProfileExportBudget, ProfileExportCounter,
    ProfileExportOptions, ProfileExportSpan, ProfileExportSpanRange, ProfileExportTruncation,
    ProfileExportWallNs,
};
`;
export const hostDeclaration = "pub mod profile_export;\n";
export const hostImport =
  "use vize_carton::profile_export::{ProfileExportBudget, ProfileExportOptions, export_report};";
export const commonOldImport =
  "use vize_l0::profiler::{ProfileExportBudget, ProfileExportOptions, global_profiler};";
export const commonNewImport = hostImport + "\nuse vize_l0::profiler::global_profiler;";
export const oracleFiles = [
  "davinci/vize_l0/tests/pass_observer_timing.rs",
  "davinci/vize_l0/tests/dump_collector.rs",
  "davinci/vize_l1_to_l2/tests/provenance_profile_wire.rs",
];
export const callerChanges: [string, string, string][] = [
  ...["crates/vize/src/commands/profile_export.rs", ...oracleFiles].map(
    (file): [string, string, string] => [file, commonOldImport, commonNewImport],
  ),
  [
    "crates/vize_curator/src/inspector/stages/profile.rs",
    "use vize_l0::profiler::{ProfileExportBudget, ProfileExportOptions, Profiler, SpanAttribution};",
    hostImport + "\nuse vize_l0::profiler::{Profiler, SpanAttribution};",
  ],
  [
    "crates/vize_atelier_sfc/examples/davinci_production_perf/attribution.rs",
    `use vize_l0::profiler::{
    ProfileExportOptions, allocation_snapshot, global_profiler, reset_allocation_counters,
};`,
    `use vize_carton::profile_export::{ProfileExportOptions, export_report};
use vize_l0::profiler::{allocation_snapshot, global_profiler, reset_allocation_counters};`,
  ],
  [
    "crates/vize_carton/tests/davinci_profile_export.rs",
    `use vize_carton::profiler::{
    ProfileExportBudget, ProfileExportOptions, ProfilingAllocator, SpanAttribution,
    allocation_snapshot, global_profiler,
};`,
    hostImport +
      `
use vize_l0::profiler::{ProfilingAllocator, SpanAttribution, allocation_snapshot, global_profiler};`,
  ],
];
export const calls: [string, string, string][] = [
  [
    "crates/vize/src/commands/profile_export.rs",
    "global_profiler().export_report(",
    "export_report(global_profiler(), ",
  ],
  ...[
    ...oracleFiles,
    "crates/vize_curator/src/inspector/stages/profile.rs",
    "crates/vize_atelier_sfc/examples/davinci_production_perf/attribution.rs",
    "crates/vize_carton/tests/davinci_profile_export.rs",
    exportTests,
  ].map((file): [string, string, string] => [
    file,
    "profiler.export_report(",
    `export_report(${file === exportTests || file.includes("vize_curator") ? "&" : ""}profiler, `,
  ]),
];
export const edges: [string, string, "dependencies" | "dev-dependencies", string][] = [
  [
    "vize_l0",
    "davinci/vize_l0/Cargo.toml",
    "dev-dependencies",
    'vize_carton = { path = "../../crates/vize_carton" }',
  ],
  [
    "vize_l1_to_l2",
    "davinci/vize_l1_to_l2/Cargo.toml",
    "dev-dependencies",
    "vize_carton.workspace = true",
  ],
  [
    "vize_atelier_sfc",
    "crates/vize_atelier_sfc/Cargo.toml",
    "dev-dependencies",
    "vize_carton.workspace = true",
  ],
  [
    "vize_curator",
    "crates/vize_curator/Cargo.toml",
    "dependencies",
    "vize_carton.workspace = true",
  ],
];

export const gateChanges: [string, string, string][] = [
  [
    "tests/tooling/davinci/davinci-atelier-sfc-stage-alias.test.mjs",
    'import { test } from "node:test";',
    'import { test } from "node:test";\nimport { parse as parseToml } from "@iarna/toml";',
  ],
  [
    "tests/tooling/davinci/davinci-atelier-sfc-stage-alias.test.mjs",
    "function compiler() {",
    `function assertPreferredDependencies(cargoToml) {
  const manifest = parseToml(cargoToml);
  assert.deepEqual(manifest.dependencies.vize_l0, { workspace: true });
  assert.deepEqual(manifest["dev-dependencies"].vize_carton, { workspace: true });
  for (const scope of [manifest, ...Object.values(manifest.target ?? {})])
    for (const kind of ["dependencies", "build-dependencies"])
      for (const [key, value] of Object.entries(scope[kind] ?? {}))
        assert.ok(key !== "vize_carton" && value?.package !== "vize_carton");
}

function compiler() {`,
  ],
  [
    "tests/tooling/davinci/davinci-atelier-sfc-stage-alias.test.mjs",
    "  assert.match(cargoToml, /^vize_l0\\.workspace = true$/m);\n  assert.doesNotMatch(cargoToml, /^vize_carton\\.workspace = true$/m);",
    "  assertPreferredDependencies(cargoToml);",
  ],
  [
    "tests/tooling/support/davinci-host-imports.ts",
    'import { withoutI18nHostImports } from "./davinci-i18n-host-imports.ts";',
    'import { withoutProfileHostImports } from "./davinci-profile-host-imports.ts";\nimport { withoutI18nHostImports } from "./davinci-i18n-host-imports.ts";',
  ],
  [
    "tests/tooling/support/davinci-host-imports.ts",
    "return withoutI18nHostImports(storage, file);",
    "return withoutProfileHostImports(withoutI18nHostImports(storage, file), file);",
  ],
  [
    "tests/tooling/davinci-stage-dependencies.test.ts",
    '["vize", "vize_patina", "vize_maestro", "vize_relief", "vize_vitrine"]',
    '["vize", "vize_patina", "vize_maestro", "vize_relief", "vize_vitrine", "vize_curator"]',
  ],
];

// Exact reviewed bytes: pure source, precise replay output and Rustfmt output.
export const hashes = {
  sfcGate: [
    "f7126a7c534d77663eb1437b179f2d6e251b2e72c900a309dec8a737266c6aa9",
    [
      "77c6abd1f3216fcccd5bcc5b65ad0110fbe297957d153bdac58c4ca86a661bcc",
      "0039b68bc23fa3077a61407c0caf903a978e81027377c99563142120008829f1",
      "26f13cc77bf1f531f4aea165ebdf96d9b3b5629500fd09664e84ba077d2f4172",
    ],
  ],
  companion: "fbe7a328369e33946c0393c9e017551b33abbd84112d8ffe38950ecfae292e09",
  provider: "0c8f8755bf717dee0f3dbfd1ca61d40d64b14857ea62304bb9217bbfa69933d8",
  exporter: [
    "628bf5dac6d579a99e4ff76a14b04147df0028fdf921fd358a6660023115416a",
    [
      "338ff8f6ddd6fe54ff6610c47c7d97eec0c223642b5dbdedccca8b9f153f4ffb",
      "338ff8f6ddd6fe54ff6610c47c7d97eec0c223642b5dbdedccca8b9f153f4ffb",
    ],
  ],
  tests: [
    "a9d51cf548f238050ba4c8a59f107b00ffcc088b446152966a5381baa1c2ac25",
    [
      "35dcc60e6db0cc7f989379c84cd0f88cb1e1f42cd961fa6df6959ef8a03f291a",
      "3c82a8d4f46978f9e73d79b37bee5342f64466e09c0525a557106d609fa84b47",
    ],
  ],
};
