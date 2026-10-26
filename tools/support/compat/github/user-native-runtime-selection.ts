/** Reuse the ordinary source CLI and NAPI builds for runtime-boundary changes. */
export function userNativeRuntimeSelectionRequired(paths: readonly string[]): boolean {
  return (
    paths.length === 0 ||
    paths.some((path) =>
      /^(?:crates\/vize_carton\/src\/corsa_resolver(?:\.rs$|\/)|crates\/vize_canon\/src\/(?:lsp_client\/bootstrap\.rs$|batch\/error\.rs$)|crates\/vize_maestro\/src\/ide\/diagnostics(?:\.rs$|\/typecheck_unavailable_tests\.rs$)|crates\/vize_patina\/src\/linter\/(?:corsa_session\/(?:paths(?:\.rs$|\/)|session\.rs$)|native_type_aware\.rs$)|npm\/cli\/(?:package\.json$|src\/cli\.ts$)|editors\/vscode\/(?:package\.json$|src\/(?:corsa-runtime|server-options|extension)\.ts$)|tests\/_fixtures\/runtime\/native-runtime-10694\/|tests\/tooling\/(?:native-runtime-(?:cli-selection|napi-selection|selection-lsp-strace)\.test\.ts$|support\/native-runtime-selection\/)|tests\/differential\/build-receipt\.ts$|npm\/native\/scripts\/formatter-history-build\.mts$|tools\/support\/compat\/github\/(?:user-native-runtime-selection|plan-pr-checks)\.ts$|\.github\/(?:actions\/(?:test-native-runtime-selection|setup-frozen-native-test-runtime)\/|workflows\/(?:pr-source-checks|pr-js-checks)\.yml$))/.test(
        path,
      ),
    )
  );
}
