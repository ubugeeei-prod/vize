use super::super::{
    create_project_case_without_node_modules, resolve_test_tsgo_binary,
    snapshot_project_diagnostics,
};

#[test]
fn nuxt_reference_graph_preserves_plugin_injection_import_types() {
    let project_root = create_project_case_without_node_modules(
        "nuxt-plugin-injection-import-types",
        &[
            (
                ".nuxt/nuxt.d.ts",
                "/// <reference path=\"types/plugins.d.ts\" />\n",
            ),
            (
                "node_modules/nuxt/package.json",
                r#"{"name":"nuxt","version":"4.5.2","type":"module"}"#,
            ),
            (
                "node_modules/nuxt/dist/app/index.d.ts",
                "export * from './nuxt.js'\n",
            ),
            (
                "node_modules/nuxt/dist/app/nuxt.d.ts",
                r#"export interface NuxtApp {
  [key: string]: unknown
}

export interface Plugin<Injections extends Record<string, unknown> = Record<string, unknown>> {
  (nuxtApp: NuxtApp): void | { provide: Injections }
}

export declare function defineNuxtPlugin<T extends Record<string, unknown>>(
  plugin: (nuxtApp: NuxtApp) => void | { provide: T }
): Plugin<T>

export declare function useNuxtApp(): NuxtApp
"#,
            ),
            (
                "node_modules/nuxt/dist/app/nuxt.js",
                "export function useNuxtApp() {}\nexport function defineNuxtPlugin(plugin) { return plugin }\n",
            ),
            (
                "node_modules/nuxt/dist/pages/runtime/plugins/router.d.ts",
                r#"import type { Plugin } from '#app/nuxt'

interface Router {
  currentRoute: { value: { path: string } }
}

declare const plugin: Plugin<{ router: Router }>
export default plugin
"#,
            ),
            (
                "node_modules/nuxt/dist/pages/runtime/plugins/router.js",
                "export default {}\n",
            ),
            (
                ".nuxt/types/plugins.d.ts",
                r#"import type { Plugin } from '#app'

type Decorate<T extends Record<string, unknown>> = {
  [K in keyof T as K extends string ? `$${K}` : never]: T[K]
}

type InjectionType<A extends Plugin> = A extends { default: Plugin<infer T> }
  ? Decorate<T>
  : unknown

type NuxtAppInjections =
  InjectionType<typeof import("../../node_modules/nuxt/dist/pages/runtime/plugins/router.js")> &
  InjectionType<typeof import("../../app/plugins/track.client")>

declare module '#app' {
  interface NuxtApp extends NuxtAppInjections {}
}
"#,
            ),
            (
                "app/plugins/track.client.ts",
                r#"export default defineNuxtPlugin(nuxtApp => {
  const path: string = nuxtApp.$router.currentRoute.value.path
})
"#,
            ),
            (
                "app/route-path.ts",
                "export const path: string = useNuxtApp().$router.currentRoute.value.path\n",
            ),
        ],
    );
    std::fs::write(
        project_root.join("globals.d.ts"),
        format!(
            r#"export {{}}

declare global {{
  const defineNuxtPlugin: typeof import("{root}/node_modules/nuxt/dist/app/nuxt").defineNuxtPlugin
  const useNuxtApp: typeof import("{root}/node_modules/nuxt/dist/app/nuxt").useNuxtApp
}}
"#,
            root = project_root.display()
        ),
    )
    .unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r##"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "types": [],
    "baseUrl": ".nuxt",
    "paths": {
      "#app": ["../node_modules/nuxt/dist/app"],
      "#app/*": ["../node_modules/nuxt/dist/app/*"]
    }
  },
  "include": [
    ".nuxt/nuxt.d.ts",
    "globals.d.ts",
    "app/**/*",
    "node_modules/nuxt/dist/**/*.js"
  ]
}"##,
    )
    .unwrap();

    if resolve_test_tsgo_binary().is_some() {
        assert_eq!(
            snapshot_project_diagnostics(&project_root),
            Some(Vec::new())
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}
