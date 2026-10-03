// Source-built SSR proof follows the actual selected-owner/collector/writer
// inputs. Prose and generated ledger replays never substitute for that proof.
export function nativeSsrCaptureRequired(paths) {
  return paths.some((path) =>
    /^(?:davinci\/vize_l0\/src\/|davinci\/vize_l1\/src\/(?:container|markup|embed|surface|event|parse|build|dialect\/vue|lib\.rs$)|davinci\/vize_l1_to_l2\/src\/native_file(?:\.rs|\/selected(?:\.rs|\/))|davinci\/vize_l2\/src\/(?:artifact|file|op|scope|resolution|lang\/js|lib\.rs$)|davinci\/vize_l3\/src\/decision\/(?:[^/]+\.rs$|build\/|policy\/|ssr(?:\.rs|\/))|davinci\/vize_l4\/src\/(?:write|runtime|module(?:\.rs$|\/(?:imports|setup)\.rs$)|targets\/ssr(?:\.rs|\/)|lib\.rs$)|davinci\/vize_l4\/tests\/(?:native_(?:ssr|selected_ssr)|fixtures\/native-(?:selected-)?ssr-)|crates\/vize_atelier_sfc\/src\/(?:native_ssr(?:\.rs|\/)|lib\.rs$)|crates\/vize_atelier_sfc\/tests\/(?:native_scriptless_ssr\.rs$|fixtures\/native-sfc-ssr-)|tests\/tooling\/(?:(?:l4-(?:native|selected)-ssr-reference|native-sfc-scriptless-ssr-reference|native-ssr-capture)\.test\.(?:ts|mjs)$|support\/native-sfc-ssr-reference\.ts$)|vendor\/oxc_parser\/src\/|\.github\/actions\/test-native-ssr\/|\.github\/workflows\/(?:check|pr-source-checks)\.yml$|tools\/support\/compat\/github\/(?:native-ssr-capture|plan-tooling-tests)\.mjs$|davinci\/vize_l[0-4](?:_to_l[0-4])?\/Cargo\.toml$|crates\/vize_atelier_sfc\/Cargo\.toml$|Cargo\.(?:toml|lock)$|rust-toolchain\.toml$|pnpm-(?:lock\.yaml|workspace\.yaml)$|npm\/ui\/package\.json$)/.test(
      path,
    ),
  );
}
