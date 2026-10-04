// Additional genuine scoped SSR inputs. The existing unstyled SSR predicate
// remains unchanged; the planner joins both into its existing SSR proof flag.
export function nativeScopedCaptureRequired(paths) {
  return paths.some((path) =>
    /^(?:davinci\/vize_l1\/src\/css(?:\.rs$|\/)|davinci\/vize_l1_to_l2\/src\/native_file\/selected_scoped(?:\.rs$|\/)|davinci\/vize_l4\/src\/module\/scope\.rs$|crates\/vize_atelier_sfc\/src\/(?:native_scoped_ssr(?:\.rs$|\/)|native(?:\.rs$|\/(?:scope|styles|scoped)(?:\.rs$|\/)))|crates\/vize_atelier_sfc\/tests\/(?:native_scoped_ssr\.rs$|fixtures\/native-scoped-ssr-vue-[^/]+\.json$)|tests\/tooling\/(?:native-sfc-scoped-ssr-reference\.test\.ts$|native-scoped-capture\.test\.mjs$|support\/native-scoped-ssr-[^/]+\.ts$)|\.github\/actions\/test-native-scoped-css\/|tools\/support\/compat\/github\/native-scoped-capture\.mjs$)/.test(
      path,
    ),
  );
}
