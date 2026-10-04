// Only actual original Vapor source/capture inputs qualify affected PR capture.
// Protected merge groups remain unconditional and prose stays outside this hook.
export function nativeVaporCaptureRequired(paths) {
  return paths.some((path) =>
    /^(?:davinci\/vize_l(?:0|1|2)\/src\/|davinci\/vize_l1_to_l2\/src\/native_file(?:\.rs|\/)|davinci\/vize_l3\/src\/decision(?:\.rs|\/)|davinci\/vize_l4\/src\/(?:write(?:\.rs|\/)|runtime(?:\.rs|\/)|module(?:\.rs|\/)|targets\/vapor(?:\.rs|\/))|davinci\/vize_l4\/tests\/(?:native_vapor\.rs$|fixtures\/native-vapor-)|crates\/vize_atelier_sfc\/src\/(?:lib\.rs$|native_vapor(?:_setup)?(?:\.rs|\/))|crates\/vize_atelier_sfc\/tests\/(?:native_vapor(?:_setup)?_sfc\.rs$|fixtures\/native_vapor(?:_setup)?_sfc_)|tests\/tooling\/(?:native-vapor(?:-(?:sfc|setup-sfc))?-reference\.test\.mjs$|support\/(?:native-vapor-|generate-native-vapor-|vue-vapor-release\.mjs$))|\.github\/(?:actions\/test-native-vapor\/|workflows\/(?:check|pr-source-checks)\.yml$)|tools\/support\/compat\/github\/native-vapor-capture\.mjs$)/.test(
      path,
    ),
  );
}
