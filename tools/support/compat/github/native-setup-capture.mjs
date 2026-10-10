// Actual native setup inputs qualify a source-built affected-PR capture.
// Prose/ledger replays do not. Protected merge groups remain unconditional.
export function nativeSetupCaptureRequired(paths) {
  const staticClassInput =
    /^(?:davinci\/vize_l4\/(?:src\/targets(?:\.rs$|\/static_class\.rs$)|tests\/(?:native_dom_static_class\.rs$|fixtures\/native-dom-static-class-))|tests\/tooling\/native-dom-static-class-reference\.test\.ts$)/;
  const selectedSfcInput =
    /^(?:crates\/vize_atelier_sfc\/src\/native_selected(?:\.rs$|\/)|crates\/vize_atelier_sfc\/tests\/fixtures\/native_selected_sfc_click_vue_3_5_35\.json$|tests\/tooling\/native-selected-sfc-dom-(?:reference|capture-workflow)\.test\.ts$|tests\/tooling\/support\/native-selected-sfc-dom-runtime\.ts$|\.github\/actions\/test-native-selected-sfc-dom\/action\.yml$)/;
  const localHandlerInput =
    /^(?:davinci\/vize_l3\/src\/decision\/dom\/build\/handler(?:\.rs$|\/)|crates\/vize_atelier_sfc\/tests\/fixtures\/native_handler_local_sfc_click_vue_3_5_35\.json$|tests\/tooling\/native-handler-local-sfc-dom-(?:reference|capture-workflow)\.test\.ts$|tests\/tooling\/support\/native-handler-local-sfc-dom-runtime\.ts$|\.github\/actions\/test-native-handler-local-sfc-dom\/action\.yml$)/;
  const selectedSetupInput =
    /^(?:crates\/vize_atelier_sfc\/src\/native_selected_setup(?:\.rs$|\/)|crates\/vize_atelier_sfc\/tests\/fixtures\/(?:native_selected_setup_sfc|native_original_for(?:_(?:value|constant(?:_inherited)?))?_sfc)_vue_3_5_35\.json$|tests\/tooling\/native-(?:selected-setup|original-for(?:-(?:value|constant))?)-sfc-dom-(?:reference|capture-workflow)\.test\.ts$|tests\/tooling\/support\/(?:native-selected-setup-sfc-dom-runtime|native-original-for-constant-sfc-dom-(?:runtime|processes|observation))\.ts$|davinci\/vize_l3\/src\/decision\/native\/setup(?:\.rs$|\/))/;
  const nativeDomInput =
    /^davinci\/(?:vize_l2\/src\/resolution(?:\.rs$|\/)|vize_l3\/src\/decision\/dom(?:\.rs$|\/)|vize_l4\/src\/(?:targets\/dom(?:\.rs$|\/)|expr(?:\.rs$|\/)))/;
  return paths.some(
    (path) =>
      selectedSfcInput.test(path) ||
      staticClassInput.test(path) ||
      selectedSetupInput.test(path) ||
      nativeDomInput.test(path) ||
      localHandlerInput.test(path) ||
      /^(?:davinci\/vize_l2\/src\/(?:file(?:\.rs|\/)|lang\/js\/file(?:\.rs|\/))|davinci\/vize_l3\/src\/decision\/dom\/(?:vue(?:\.rs|\/)|build(?:\.rs|\/)|dependencies\.rs$)|davinci\/vize_l4\/src\/(?:module(?:\.rs|\/)|targets\/dom(?:\.rs$|\/(?:vue(?:\.rs|\/)|(?:for_head|expression|write)\.rs$))|expr\.rs$)|crates\/vize_atelier_sfc\/src\/native(?:\.rs|\/)|crates\/vize_atelier_sfc\/tests\/fixtures\/native_sfc_(?:(?:js|const|ts[^/]*)_setup_|ordinary_empty_)|tests\/tooling\/native-sfc-(?:(?:js|primitive|strict|annotation)-setup|ordinary-empty-reference\.test\.ts$)|tests\/tooling\/support\/native-sfc-(?:setup|ordinary)-reference\.ts$|vendor\/oxc_parser\/src\/|\.github\/actions\/test-native-js-setup\/|tools\/support\/compat\/github\/native-setup-capture\.mjs$)/.test(
        path,
      ),
  );
}

export function toolingChecksRequired(plan, paths) {
  return plan.tests.length > 0 || nativeSetupCaptureRequired(paths);
}
