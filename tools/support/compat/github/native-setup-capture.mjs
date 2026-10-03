// Actual native setup inputs qualify a source-built affected-PR capture.
// Prose/ledger replays do not. Protected merge groups remain unconditional.
export function nativeSetupCaptureRequired(paths) {
  const selectedSfcInput =
    /^(?:crates\/vize_atelier_sfc\/src\/native_selected(?:\.rs$|\/)|crates\/vize_atelier_sfc\/tests\/fixtures\/native_selected_sfc_click_vue_3_5_35\.json$|tests\/tooling\/native-selected-sfc-dom-(?:reference|capture-workflow)\.test\.ts$|tests\/tooling\/support\/native-selected-sfc-dom-runtime\.ts$|\.github\/actions\/test-native-selected-sfc-dom\/action\.yml$)/;
  return paths.some(
    (path) =>
      selectedSfcInput.test(path) ||
      /^(?:davinci\/vize_l2\/src\/(?:file(?:\.rs|\/)|lang\/js\/file(?:\.rs|\/))|davinci\/vize_l3\/src\/decision\/dom\/vue(?:\.rs|\/)|davinci\/vize_l4\/src\/(?:module(?:\.rs|\/)|targets\/dom\/vue(?:\.rs|\/)|expr\.rs$)|crates\/vize_atelier_sfc\/src\/native(?:\.rs|\/)|crates\/vize_atelier_sfc\/tests\/fixtures\/native_sfc_(?:(?:js|const|ts[^/]*)_setup_|ordinary_empty_)|tests\/tooling\/native-sfc-(?:(?:js|primitive|strict|annotation)-setup|ordinary-empty-reference\.test\.ts$)|tests\/tooling\/support\/native-sfc-(?:setup|ordinary)-reference\.ts$|vendor\/oxc_parser\/src\/|\.github\/actions\/test-native-js-setup\/|tools\/support\/compat\/github\/native-setup-capture\.mjs$)/.test(
        path,
      ),
  );
}

export function toolingChecksRequired(plan, paths) {
  return plan.tests.length > 0 || nativeSetupCaptureRequired(paths);
}
