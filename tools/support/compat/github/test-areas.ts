/**
 * Product areas of test cases, decided by the repository path that owns each case:
 * a crate or package directory, or a test file's directory. Listed in report order.
 */
export const TEST_AREAS: ReadonlyArray<{
  readonly area: string;
  readonly paths: readonly string[];
}> = [
  {
    area: "Compiler",
    paths: [
      "crates/vize_armature",
      "crates/vize_relief",
      "crates/vize_carton",
      "crates/vize_croquis",
      "crates/vize_atelier_core",
      "crates/vize_atelier_dom",
      "crates/vize_atelier_jsx",
      "crates/vize_atelier_sfc",
      "crates/vize_incremental",
      "davinci",
      "upstream/forks/oxc_parser",
      "upstream/forks/lightningcss",
      "upstream/forks/parcel_sourcemap",
    ],
  },
  { area: "Vapor", paths: ["crates/vize_atelier_vapor"] },
  { area: "SSR", paths: ["crates/vize_atelier_ssr"] },
  { area: "Type checker", paths: ["crates/vize_canon", "crates/vize_croquis_cf"] },
  {
    area: "Linter",
    paths: ["crates/vize_patina", "upstream/forks/oxlint_html_renderer186", "npm/linter/oxlint"],
  },
  { area: "Formatter", paths: ["crates/vize_glyph", "upstream/forks/oxc_formatter"] },
  { area: "LSP", paths: ["crates/vize_maestro", "editors"] },
  { area: "CLI", paths: ["crates/vize", "crates/vize_doctor", "crates/vize_curator", "npm/cli"] },
  {
    area: "Builders and plugins",
    paths: ["crates/vize_vitrine", "npm/builder", "npm/framework", "npm/native", "npm/wasm"],
  },
  { area: "Musea", paths: ["crates/vize_musea", "npm/mcp-musea"] },
  { area: "Fresco", paths: ["crates/vize_fresco", "npm/fresco", "npm/fresco-native"] },
  { area: "Marquette", paths: ["crates/vize_marquette", "npm/marquette"] },
  { area: "UI", paths: ["npm/ui", "npm/compose"] },
  { area: "Playground", paths: ["playground"] },
  { area: "Examples", paths: ["examples"] },
  { area: "Docs", paths: ["docs"] },
  {
    area: "Tooling",
    paths: ["tests/tooling", "tests/davinci_test_support", "tests/vize_test_runner", "tools"],
  },
];

// The CLI links its integration tests as one binary; each module keeps the
// prefix of the product it exercises (see rust-test-cohorts.ts).
const CLI_INTEGRATION: ReadonlyArray<readonly [string, RegExp]> = [
  ["Type checker", /^vize::integration (?:check|typecheck)_/],
  ["LSP", /^vize::integration lsp_/],
  ["Linter", /^vize::integration lint_/],
  ["Formatter", /^vize::integration (?:fmt|format)_/],
];

function owns(owner: string, path: string): boolean {
  return path === owner || path.startsWith(`${owner}/`);
}

/** The area of one case; a path no area owns is reported under its own name. */
export function areaFor(path: string, name: string): string {
  if (path === "crates/vize") {
    const feature = CLI_INTEGRATION.find(([, pattern]) => pattern.test(name));
    if (feature) return feature[0];
  }
  const matches = TEST_AREAS.filter(({ paths }) => paths.some((owner) => owns(owner, path)));
  // Owners never nest across areas, so a path has at most one.
  if (matches.length > 1) throw new Error(`${path} belongs to ${matches.length} test areas`);
  return matches[0]?.area ?? `Other (${path.split("/").slice(0, 2).join("/")})`;
}

export function areaRank(area: string): number {
  const index = TEST_AREAS.findIndex((entry) => entry.area === area);
  return index === -1 ? TEST_AREAS.length : index;
}
