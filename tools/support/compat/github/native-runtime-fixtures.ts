// Executable discovery has no authored-language coordinates. These closed
// source identities and real Rust graph consumers stay outside batch cases.
import { createHash } from "node:crypto";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
type SourcePair = { path: string; before: string | null; after: string };
type Selector = { selector: string; source: string; consumesFixture: boolean };
export const runtimeFixturePath = "tests/_fixtures/runtime/native-runtime-10694/selection.json";
const runtimeFixtureSha256 = "20408c95b216c0b655200a5491e3ed0542a13e8db1adaf486274677b375f5463";

// Populated with the exact reviewed consumer source pairs, never a directory
// exemption. A later edit cannot reuse the Issue 10694 registration.
const registered: Record<string, { sources: SourcePair[]; tests: Selector[] }> = {
  typechecker: {
    sources: [
      {
        path: "crates/vize_carton/src/corsa_resolver.rs",
        before: "947bf22565f1948e3a2848dd90b7c1f3ec259bf293d6176941528291f0612717",
        after: "f195f877677277cd66910e67165fd2c6dd31738cf78113acecfa1fe06ccc381e",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/tests.rs",
        before: "6c15b056384611da328d4dc4bc01dc6f3cbe665bec18bbabffdb747233aa4ee2",
        after: "286aecabc6099ae6dc787bf51ba6f3e9015663167d6ca4bc0463e82fda7ad1cc",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/tests/fixture_cases.rs",
        before: null,
        after: "f869b0a0acb0207661e6f6a403b91850f3241cf6e1b14fc6da326b20affdb83a",
      },
      {
        path: "crates/vize_canon/src/lsp_client/bootstrap.rs",
        before: "b0f9fa8f1a3b9eb8df8ea12f5141059cd195a60d393da7e671daf5a9010a4305",
        after: "273948b9cc6c40af8e4eb0c793151caf678964b7ec2fec54c016d67431376aa6",
      },
      {
        path: "crates/vize_canon/src/batch/error.rs",
        before: "495e53261574cb81f49590911c552f3b6f614d9b700e5fd6c18e4186ad156d14",
        after: "895b884def8f152505e48742ab21e6fec23f1bda4c86af15064acbd5b7974f02",
      },
      {
        path: "crates/vize_canon/src/batch/snapshots/vize_canon__batch__error__tests__corsa_not_found_error_message.snap",
        before: "bef84760ee73db62358bf6b148fd029f37d5fc8dff61e5ac90c5a9ce09b7b0d1",
        after: "6c8c77c5d558f9e75c1e7950ae72bd6445fe810899d08a4c4c67c03a180584b3",
      },
      {
        path: "crates/vize_canon/src/batch/snapshots/vize_canon__batch__error__tests__corsa_not_found_no_pm.snap",
        before: "e40b76d36a5a64ecfcb4e49a6df5b753ff0d83ad013a57f0378b86a2356de5da",
        after: "de87619592c61e08e6406c25bfcdea334befdd33dfc0b62fa266aa0391a07ed2",
      },
    ],
    tests: [
      {
        selector:
          "corsa_resolver::tests::fixture_cases::runtime_selection_fixture_covers_declared_graphs_and_overrides",
        source: "crates/vize_carton/src/corsa_resolver/tests/fixture_cases.rs",
        consumesFixture: true,
      },
      {
        selector:
          "lsp_client::bootstrap::tests::runtime_selection_fixture_preserves_explicit_path_authority",
        source: "crates/vize_canon/src/lsp_client/bootstrap.rs",
        consumesFixture: true,
      },
      {
        selector:
          "batch::error::tests::explicit_missing_runtime_reports_the_full_path_without_install_advice",
        source: "crates/vize_canon/src/batch/error.rs",
        consumesFixture: false,
      },
      {
        selector:
          "batch::error::tests::every_package_manager_keeps_the_existing_typescript_dependency",
        source: "crates/vize_canon/src/batch/error.rs",
        consumesFixture: false,
      },
    ],
  },
  linter: {
    sources: [
      {
        path: "crates/vize_carton/src/corsa_resolver.rs",
        before: "947bf22565f1948e3a2848dd90b7c1f3ec259bf293d6176941528291f0612717",
        after: "f195f877677277cd66910e67165fd2c6dd31738cf78113acecfa1fe06ccc381e",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/tests.rs",
        before: "6c15b056384611da328d4dc4bc01dc6f3cbe665bec18bbabffdb747233aa4ee2",
        after: "286aecabc6099ae6dc787bf51ba6f3e9015663167d6ca4bc0463e82fda7ad1cc",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/tests/fixture_cases.rs",
        before: null,
        after: "f869b0a0acb0207661e6f6a403b91850f3241cf6e1b14fc6da326b20affdb83a",
      },
      {
        path: "crates/vize_patina/src/linter/corsa_session/paths.rs",
        before: "520d83c671bf4965de96d35272e97e9b10fe4d98f2e4a96ae389a5934da17e2e",
        after: "bc006e0ae6e331206ebf1c0e85cb6fd8d6ae4a298b36e023637636c83e383a00",
      },
      {
        path: "crates/vize_patina/src/linter/corsa_session/paths/store.rs",
        before: "f6b53e7c54bb702a1ba69dc8e906c7f414c1bdcde68c20fe01964f38d1897fd6",
        after: "5b5a3fc63a2bf6d7446220171f9b64917f417b784286f4b15641e811cfbff112",
      },
      {
        path: "crates/vize_patina/src/linter/corsa_session/paths/store/runtime_tests.rs",
        before: null,
        after: "9adfd1ceac1d790514b13915df0de205cf5b254a087e4bd662aff821d2a6c590",
      },
      {
        path: "crates/vize_patina/src/linter/corsa_session/session.rs",
        before: "bf2aff6b0ed2aa044a39c48cb3280903d76e173da2af931f9416a454edbaeb24",
        after: "5cf2c7cbbf2fc36109cb8f4e02e7e02db51902b00d32ce5b3ec04152dd872ee5",
      },
    ],
    tests: [
      {
        selector:
          "corsa_resolver::tests::fixture_cases::runtime_selection_fixture_covers_declared_graphs_and_overrides",
        source: "crates/vize_carton/src/corsa_resolver/tests/fixture_cases.rs",
        consumesFixture: true,
      },
      {
        selector:
          "linter::corsa_session::paths::store::runtime_tests::runtime_selection_fixture_uses_selected_config_roots",
        source: "crates/vize_patina/src/linter/corsa_session/paths/store/runtime_tests.rs",
        consumesFixture: true,
      },
    ],
  },
};

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
