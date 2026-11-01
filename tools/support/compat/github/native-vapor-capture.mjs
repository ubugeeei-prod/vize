// Only actual original Vapor source/capture inputs qualify affected PR capture.
// Protected merge groups remain unconditional and prose stays outside this hook.
export function nativeVaporCaptureRequired(paths) {
  const inputModel =
    /^(?:davinci\/vize_l3\/tests\/native_model_vapor\.rs$|crates\/vize_atelier_sfc\/tests\/fixtures\/native_original_input_model_(?:(?:inputs|ssr_contexts)_vue_3_5_35|vapor_sfc_vue_3_6_rc9)\.json$|tests\/tooling\/native-original-input-model-vapor-reference\.test\.ts$|tests\/tooling\/support\/native-original-input-model-vapor-(?:boundary|primary|runtime|observation)\.ts$)/;
  const trimModel =
    /^(?:davinci\/vize_l3\/tests\/native_model_trim\.rs$|crates\/vize_atelier_sfc\/tests\/native_input_model_trim\/(?:main|custody|output|refusal)\.rs$|crates\/vize_atelier_sfc\/tests\/fixtures\/native_original_input_model_trim_(?:inputs|all_targets)\.json$|tests\/tooling\/native-input-model-trim-(?:reference|capture-workflow)\.test\.ts$|tests\/tooling\/support\/native-input-model-trim-(?:boundary|dom-runtime|vapor-runtime|vapor-module)\.ts$)/;
  return paths.some(
    (path) =>
      inputModel.test(path) ||
      trimModel.test(path) ||
      /^(?:davinci\/vize_l(?:0|1|2)\/src\/|davinci\/vize_l1_to_l2\/src\/native_file(?:\.rs|\/)|davinci\/vize_l3\/src\/decision(?:\.rs|\/)|davinci\/vize_l4\/src\/(?:expr(?:\.rs$|\/)|write(?:\.rs|\/)|runtime(?:\.rs|\/)|module(?:\.rs|\/)|targets\/vapor(?:\.rs|\/))|davinci\/vize_l4\/tests\/(?:native_vapor\.rs$|fixtures\/native-vapor-)|crates\/vize_atelier_(?:jsx|vapor)\/src\/|crates\/vize_atelier_jsx\/tests\/v_slots_forwarding\.rs$|crates\/vize_atelier_sfc\/src\/(?:lib\.rs$|native_vapor(?:_setup)?(?:\.rs|\/))|crates\/vize_atelier_sfc\/tests\/(?:vapor_setup_refusals\/|native_vapor(?:_setup)?_sfc\.rs$|fixtures\/native_vapor(?:_setup)?_sfc_)|tests\/tooling\/(?:jsx-runtime-slots-vapor\.test\.mjs$|native-vapor(?:-(?:sfc|setup-sfc))?-reference\.test\.mjs$|support\/(?:davinci-runtime-trace\.mjs$|native-vapor-|generate-native-vapor-|vue-vapor-release\.mjs$))|\.github\/(?:actions\/test-native-vapor\/|workflows\/(?:check|pr-source-checks)\.yml$)|tools\/support\/compat\/github\/native-vapor-capture\.mjs$)/.test(
        path,
      ),
  );
}
