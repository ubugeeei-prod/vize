// Executable discovery has no authored-language coordinates. These closed
// source identities and real Rust graph consumers stay outside batch cases.
import { createHash } from "node:crypto";
import { nativeRuntimeRegistry as registered } from "./native-runtime-registry.ts";
import { unchangedNativeRuntimeGuidance } from "./native-runtime-guidance.ts";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
export const runtimeFixturePath = "tests/_fixtures/runtime/native-runtime-10694/selection.json";
const runtimeFixtureSha256 = "20408c95b216c0b655200a5491e3ed0542a13e8db1adaf486274677b375f5463";

// Populated with the exact reviewed consumer source pairs, never a directory
// exemption. A later edit cannot reuse the Issue 10694 registration.

function require(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`invalid native runtime case: ${message}`);
}

function object(value: unknown, keys: string[]): Record<string, unknown> {
  require(value && typeof value === "object" && !Array.isArray(value), "object required");
  require(JSON.stringify(Object.keys(value).sort()) ===
    JSON.stringify([...keys].sort()), "closed schema fields differ");
  return value as Record<string, unknown>;
}

function digest(bytes: Buffer): string {
  return createHash("sha256").update(bytes).digest("hex");
}

function pinned(
  readFile: ReadFile,
  side: "base" | "head",
  path: string,
  expected: unknown,
): Buffer | null {
  const bytes = readFile(side, path);
  if (expected === null) {
    require(bytes === null, `unexpected prior source: ${path}`);
    return null;
  }
  require(typeof expected === "string" && /^[a-f0-9]{64}$/.test(expected), `invalid hash: ${path}`);
  require(bytes !== null &&
    digest(bytes) === expected, `${side} source/input hash mismatch: ${path}`);
  return bytes;
}

/** Independently retain every non-runtime byte of the mixed error module. */
export function unchangedNonRuntimeErrors(before: string, after: string): boolean {
  const strip = (source: string) => {
    const begin = source.indexOf(
      "    /// Generate CLI error message with installation instructions.\n",
    );
    const end = source.indexOf("\n}\n\nimpl std::fmt::Display for CorsaNotFoundError", begin);
    if (begin < 0 || end < begin) return null;
    let retained = source.slice(0, begin) + source.slice(end);
    const addedTest =
      "\n    #[test]\n    fn explicit_missing_runtime_reports_the_full_path_without_install_advice(";
    const testBegin = retained.indexOf(addedTest);
    if (testBegin !== -1) {
      const testEnd = retained.lastIndexOf("\n}");
      const added = retained.slice(testBegin, testEnd);
      if (
        (added.match(/\bfn /g) ?? []).length !== 2 ||
        !added.includes("fn every_package_manager_keeps_the_existing_typescript_dependency(")
      )
        return null;
      retained = retained.slice(0, testBegin) + retained.slice(testEnd);
    }
    return retained;
  };
  const retained = strip(before);
  return retained !== null && retained === strip(after);
}

/** Only the approved runtime-help literal may change in these mixed modules. */
export function unchangedNativeRuntimeHelp(before: string, after: string): boolean {
  const oldText =
    "Type-aware lint rules were skipped because the Corsa runtime could not be started. Configure `typeChecker.corsaPath` or install `typescript@^7`.";
  const newText =
    "Type-aware lint rules were skipped because the native compiler could not be started. Install a native TypeScript runtime in this project or configure `typeChecker.corsaPath`, `CORSA_PATH`, or `TSGO_PATH` to its executable.";
  return before.split(oldText).length === 2 && before.replace(oldText, newText) === after;
}

/** Only the existing fake runtime's explicit selection may change. */
export function unchangedGlobalDiagnosticMock(before: string, after: string): boolean {
  const marker = "fn cli_global_diagnostics_do_not_trigger_session_fallback() {";
  const start = before.indexOf(marker);
  if (start < 0 || before.indexOf(marker, start + 1) !== -1) return false;
  const end = before.indexOf("\n}\n", start);
  const previous = "    let executor = CorsaExecutor::new(&case_dir).unwrap();";
  const current =
    "    let executor = CorsaExecutor::with_corsa_path(&case_dir, Some(&tsgo)).unwrap();";
  const selected = before.indexOf(previous, start);
  return (
    selected >= start &&
    end > selected &&
    before.slice(0, selected) + current + before.slice(selected + previous.length) === after
  );
}

/** Preserve the complete incremental fallback/counter control around its selected mock. */
export function unchangedIncrementalRuntimeMock(before: string, after: string): boolean {
  const previous = "    let executor = CorsaExecutor::new(&case_dir).unwrap();";
  const current =
    "    let executor = CorsaExecutor::with_corsa_path(&case_dir, Some(&tsgo)).unwrap();";
  return before.split(previous).length === 2 && before.replace(previous, current) === after;
}

export function newNativeRuntimeCases(
  product: string,
  previous: unknown,
  current: unknown,
  readFile: ReadFile,
): { ids: string[]; sources: Set<string> } {
  const result = { ids: [] as string[], sources: new Set<string>() };
  if (current === undefined) return result;
  require(Array.isArray(current), "runtimeCases must be an array");
  require(previous === undefined || Array.isArray(previous), "prior runtimeCases must be an array");
  const oldIds = new Set((previous ?? []).map((entry: { id?: unknown }) => entry?.id));
  const seen = new Set<string>();
  for (const value of current) {
    const row = object(value, ["id", "state", "adapter", "targets", "input", "sources", "tests"]);
    require(typeof row.id === "string" && !seen.has(row.id), "missing or duplicate ID");
    seen.add(row.id);
    if (oldIds.has(row.id)) continue;
    require(row.id === `${product}/runtime/project-native-runtime-10694`, "unregistered ID");
    const contract = registered[product];
    require(contract !== undefined, "unregistered product");
    require(row.state === "active" &&
      row.adapter === "native-runtime-selection-unit-v1", "unregistered state or adapter");
    require(JSON.stringify(row.targets) === '["runtime-selection"]', "unregistered target");
    const input = object(row.input, ["path", "sha256"]);
    require(input.path === runtimeFixturePath &&
      input.sha256 === runtimeFixtureSha256, "unregistered fixture path or graph identity");
    const fixture = pinned(readFile, "head", runtimeFixturePath, input.sha256);
    require(fixture !== null, "missing graph fixture");
    const graph = object(JSON.parse(fixture.toString("utf8")), [
      "schema",
      "version",
      "graphCases",
      "overrideCases",
      "routingCases",
    ]);
    require(graph.schema === "vize.native-runtime-selection" &&
      graph.version === 1, "invalid graph fixture envelope");
    for (const [collection, ids, fields] of [
      [
        "graphCases",
        [
          "canonical-beta",
          "typescript-six-beta-alias",
          "stale-bundle-refused",
          "unlinked-store-refused",
          "platform-mismatch-refused",
        ],
        ["id", "manifests", "files", "expected"],
      ],
      [
        "overrideCases",
        ["explicit-valid", "explicit-missing", "environment-valid", "environment-missing"],
        ["id", "source", "requested", "present"],
      ],
      [
        "routingCases",
        ["selected-config-root", "closest-package-before-ancestor-vue", "no-config-package-root"],
        ["id", "kind", "selectedConfig", "expectedRoot"],
      ],
    ] as const) {
      const rows = graph[collection];
      require(Array.isArray(rows) &&
        rows.length === ids.length, `all registered ${collection} required`);
      for (const [index, value] of rows.entries()) {
        const entry = object(value, [...fields]);
        require(entry.id === ids[index], `unregistered ${collection} identity`);
      }
    }
    require(Array.isArray(row.sources) &&
      row.sources.length === contract.sources.length, "all exact source pairs required");
    for (const [index, expected] of contract.sources.entries()) {
      const pair = object(row.sources[index], ["path", "before", "after"]);
      require(pair.path === expected.path &&
        pair.before === expected.before &&
        pair.after === expected.after, "unregistered source pair");
      const before = pinned(readFile, "base", expected.path, expected.before);
      const after = pinned(readFile, "head", expected.path, expected.after);
      require(after !== null, "missing current source");
      if (expected.path === "crates/vize_canon/src/batch/error.rs")
        require(before !== null &&
          unchangedNonRuntimeErrors(
            before.toString("utf8"),
            after.toString("utf8"),
          ), "unrelated mixed error module edit");
      if (expected.path === "crates/vize_canon/src/batch/executor/tests.rs")
        require(before !== null &&
          unchangedGlobalDiagnosticMock(
            before.toString("utf8"),
            after.toString("utf8"),
          ), "unrelated executor test or oracle edit");
      if (expected.path === "crates/vize_canon/src/batch/executor/tests/incremental_fallback.rs")
        require(before !== null &&
          unchangedIncrementalRuntimeMock(
            before.toString("utf8"),
            after.toString("utf8"),
          ), "unrelated incremental counter or fallback oracle edit");
      if (expected.path.startsWith("crates/vize_maestro/"))
        require(before !== null &&
          unchangedNativeRuntimeGuidance(
            expected.path,
            before.toString("utf8"),
            after.toString("utf8"),
          ), "unrelated Maestro diagnostic or oracle edit");
      if (
        expected.path === "crates/vize_patina/src/linter/native_type_aware.rs" ||
        expected.path ===
          "crates/vize_patina/src/linter/native_type_aware/plain_module_tests/packets.rs"
      )
        require(before !== null &&
          unchangedNativeRuntimeHelp(
            before.toString("utf8"),
            after.toString("utf8"),
          ), "unrelated mixed type-aware module edit");
      result.sources.add(expected.path);
    }
    require(Array.isArray(row.tests) &&
      row.tests.length === contract.tests.length, "all actual Rust selectors required");
    for (const [index, expected] of contract.tests.entries()) {
      const test = object(row.tests[index], ["selector", "source", "sha256"]);
      require(test.selector === expected.selector &&
        test.source === expected.source, "unregistered selector or source");
      const source = pinned(readFile, "head", expected.source, test.sha256)?.toString("utf8");
      const name = expected.selector.split("::").at(-1);
      require(source !== undefined &&
        new RegExp(`#\\[test\\]\\s+fn ${name}\\(`).test(
          source,
        ), "registered Rust test body missing");
      if (expected.consumesFixture)
        require(source.includes(
          runtimeFixturePath,
        ), "graph fixture not consumed by registered Rust source");
    }
    for (const [path, declaration] of [
      ["crates/vize_carton/src/corsa_resolver.rs", /#\[cfg\(test\)\]\s+mod tests;/],
      ["crates/vize_carton/src/corsa_resolver/tests.rs", /\bmod fixture_cases;/],
      ["crates/vize_carton/src/corsa_resolver/tests.rs", /\bmod provider_authority;/],
      ["crates/vize_canon/src/lsp_client/bootstrap.rs", /#\[cfg\(test\)\]\s+mod tests\s*\{/],
      [
        "crates/vize_patina/src/linter/corsa_session/paths/store.rs",
        /#\[cfg\(test\)\]\s+mod runtime_tests;/,
      ],
    ] as const) {
      const source = readFile("head", path)?.toString("utf8");
      require(source !== undefined &&
        declaration.test(source), `missing ordinary Rust module: ${path}`);
    }
    result.ids.push(row.id);
  }
  return result;
}
