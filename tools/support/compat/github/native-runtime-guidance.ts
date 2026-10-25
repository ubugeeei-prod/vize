// Closed executable-discovery guidance ownership; no LSP dialect/request credit.
export const nativeRuntimeGuidanceRegistration = {
  sources: [
    {
      path: "crates/vize_carton/src/corsa_resolver.rs",
      before: "947bf22565f1948e3a2848dd90b7c1f3ec259bf293d6176941528291f0612717",
      after: "850beef2582ea0c0e8f11afd579633e367b82bae4d3d165d1750c60b9a210231",
    },
    {
      path: "crates/vize_carton/src/corsa_resolver/tests.rs",
      before: "6c15b056384611da328d4dc4bc01dc6f3cbe665bec18bbabffdb747233aa4ee2",
      after: "16d7a3b29dfe44c129ef4005b0c62e406b4955062470a1083a6c4f6b29b67ccc",
    },
    {
      path: "crates/vize_carton/src/corsa_resolver/tests/fixture_cases.rs",
      before: null,
      after: "f869b0a0acb0207661e6f6a403b91850f3241cf6e1b14fc6da326b20affdb83a",
    },
    {
      path: "crates/vize_maestro/src/ide/diagnostics.rs",
      before: "28d3364c7c22748bc3e8c475e317c19c7d9d2e34741e823e1251881dd87456a3",
      after: "cea77bf85301aa18b7ad9d5dd305e4a6ab2641d418d2df366680af4f137e4dcb",
    },
    {
      path: "crates/vize_maestro/src/ide/diagnostics/typecheck_unavailable_tests.rs",
      before: "08d381606e630b9a03dc8b505e0c2699d1285b0b1cb654307f5e1a65c7329de3",
      after: "e413af3ac80a9706b89146e1aad38dbdc4b5df39ee702cd33c28e4462997304d",
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
        "ide::diagnostics::typecheck_unavailable_tests::unavailable_typecheck_diagnostic_points_to_typescript_7",
      source: "crates/vize_maestro/src/ide/diagnostics/typecheck_unavailable_tests.rs",
      consumesFixture: false,
    },
    {
      selector:
        "ide::diagnostics::typecheck_unavailable_tests::unavailable_typecheck_notice_points_to_typescript_7",
      source: "crates/vize_maestro/src/ide/diagnostics/typecheck_unavailable_tests.rs",
      consumesFixture: false,
    },
  ],
};

/** Keep all diagnostic behavior and all existing unit assertions outside guidance exact. */
export function unchangedNativeRuntimeGuidance(
  path: string,
  before: string,
  after: string,
): boolean {
  if (path === "crates/vize_maestro/src/ide/diagnostics.rs") {
    const strip = (source: string): string | null => {
      let result = source;
      for (const name of [
        "TYPECHECK_UNAVAILABLE_HINT_MESSAGE",
        "TYPECHECK_UNAVAILABLE_NOTICE_MESSAGE",
      ]) {
        const pattern = new RegExp(`(?<=const ${name}: &str = )"(?:[^"\\\\]|\\\\[\\s\\S])*"`, "g");
        if ([...result.matchAll(pattern)].length !== 1) return null;
        result = result.replace(pattern, '"PINNED_RUNTIME_GUIDANCE"');
      }
      return result;
    };
    const retained = strip(before);
    return retained !== null && retained === strip(after);
  }
  if (path === "crates/vize_maestro/src/ide/diagnostics/typecheck_unavailable_tests.rs") {
    const oldHint = "    assert_ts7_guidance(&hint.message);";
    const newHint =
      '    assert_eq!(\n        hint.message,\n        "Type checking is unavailable in this workspace. Make sure `tsconfig.json` exists. \\\n         Provide a native TypeScript compiler, for example `typescript-native@npm:typescript@7.1.0-beta`, \\\n         or configure `typeChecker.corsaPath`, `CORSA_PATH`, or `TSGO_PATH`; \\\n         see https://vizejs.dev/guide/static-analysis."\n    );';
    const oldNotice = "    assert_ts7_guidance(TYPECHECK_UNAVAILABLE_NOTICE_MESSAGE);";
    const newNotice =
      '    assert_eq!(\n        TYPECHECK_UNAVAILABLE_NOTICE_MESSAGE,\n        "Vize: type checking is unavailable in this workspace. Make sure tsconfig.json exists. \\\n         Provide a native TypeScript compiler, for example `typescript-native@npm:typescript@7.1.0-beta`, \\\n         or configure `typeChecker.corsaPath`, `CORSA_PATH`, or `TSGO_PATH`."\n    );';
    const oldHelper =
      '\nfn assert_ts7_guidance(message: &str) {\n    assert!(message.contains("typescript@^7"));\n    assert!(message.contains("typeChecker.corsaPath"));\n    assert!(!message.contains("@typescript/native-preview"));\n}\n';
    return (
      before.split(oldHint).length === 2 &&
      before.split(oldNotice).length === 2 &&
      before.endsWith(oldHelper) &&
      before.replace(oldHint, newHint).replace(oldNotice, newNotice).slice(0, -oldHelper.length) ===
        after
    );
  }

  return false;
}
