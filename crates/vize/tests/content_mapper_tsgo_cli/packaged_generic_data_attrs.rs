//! A generic SFC imported by path from a package, mapped through its shipped
//! vue-tsc declaration (the shape of `@nuxt/ui/components/NavigationMenu.vue`).
//! With embedded helpers nothing declares `__VLS_PROPS_FALLBACK`, so the props
//! are closed; loose mode must still let `data-*` and `aria-*` fall through.

use std::{fs, path::Path, path::PathBuf};

use super::{
    CompactString, TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};

const PACKAGE: &str = r#"{"name":"ui-kit","type":"module","exports":{"./components/*":"./dist/runtime/components/*"}}"#;

const COMPILED: &str = r#"<script setup>
defineOptions({ inheritAttrs: false });
defineProps({ items: { type: null, required: false }, orientation: { type: null, required: false } });
defineEmits(["update:modelValue"]);
</script>
<template><nav v-bind="$attrs" /></template>
"#;

const DECLARATION: &str = r#"export interface NavigationMenuItem {
    label?: string;
}
type Orientation = 'horizontal' | 'vertical';
export interface NavigationMenuProps<T extends NavigationMenuItem[] | NavigationMenuItem[][] = NavigationMenuItem[], O extends Orientation = Orientation> {
    items?: T;
    orientation?: O;
    modelValue?: string;
}
declare const _default: typeof __VLS_export;
export default _default;
declare const __VLS_export: <T extends NavigationMenuItem[] | NavigationMenuItem[][], O extends Orientation = Orientation>(__VLS_props: NonNullable<Awaited<typeof __VLS_setup>>["props"], __VLS_ctx?: __VLS_PrettifyLocal<Pick<NonNullable<Awaited<typeof __VLS_setup>>, "attrs" | "emit" | "slots">>, __VLS_exposed?: NonNullable<Awaited<typeof __VLS_setup>>["expose"], __VLS_setup?: Promise<{
    props: import("vue").PublicProps & __VLS_PrettifyLocal<NavigationMenuProps<T, O> & {
        "onUpdate:modelValue"?: ((value: string | undefined) => any) | undefined;
    }> & (typeof globalThis extends {
        __VLS_PROPS_FALLBACK: infer P;
    } ? P : {});
    expose: (exposed: {}) => void;
    attrs: any;
    slots: {};
    emit: (evt: "update:modelValue", value: string | undefined) => void;
}>) => import("vue").VNode & {
    __ctx?: NonNullable<Awaited<typeof __VLS_setup>>;
};
type __VLS_PrettifyLocal<T> = (T extends any ? {
    [K in keyof T]: T[K];
} : {
    [K in keyof T as K]: T[K];
}) & {};
"#;

const APP: &str = r#"<script setup lang="ts">
import UNavigationMenu from 'ui-kit/components/NavigationMenu.vue'
</script>

<template>
  <UNavigationMenu :items="[{ label: 'Home' }]" data-testid="nav" />
  <UNavigationMenu :items="[{ label: 'Home' }]" :data-index="0" aria-label="Main" />
</template>
"#;

const WRONG: &str = r#"<script setup lang="ts">
import UNavigationMenu from 'ui-kit/components/NavigationMenu.vue'
</script>

<template>
  <UNavigationMenu :items="[{ label: 1 }]" data-testid="nav" />
</template>
"#;

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn tsconfig(strict: bool) -> String {
    let mut config = serde_json::json!({
        "compilerOptions": {
            "module": "ESNext",
            "moduleResolution": "Bundler",
            "noEmit": true,
            "skipLibCheck": true,
            "strict": true,
            "target": "ESNext",
            "types": [],
        },
        "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
        "include": ["src/**/*.vue"],
    });
    if strict {
        config["vueCompilerOptions"] = serde_json::json!({ "checkUnknownProps": true });
    }
    serde_json::to_string_pretty(&config).unwrap()
}

/// `file(line,column) TSxxxx` heads the host printed, in order.
fn mapped_heads(prefix: &str, strict: bool) -> Option<Vec<CompactString>> {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return None;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(cases_root)
        .unwrap();
    let root = project.path();
    install_mapper_manifest(root);
    install_vue_package(root);
    let components = "node_modules/ui-kit/dist/runtime/components";
    write(root, "node_modules/ui-kit/package.json", PACKAGE);
    write(root, &cstr!("{components}/NavigationMenu.vue"), COMPILED);
    write(
        root,
        &cstr!("{components}/NavigationMenu.d.vue.ts"),
        DECLARATION,
    );
    write(root, "src/App.vue", APP);
    write(root, "src/Wrong.vue", WRONG);
    write(root, "tsconfig.json", &tsconfig(strict));
    let output = check_project(&tsgo, root, "tsconfig.json");
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    // An untransformed file prints TS18069, which the exact head list rejects.
    eprintln!("{}", output_text(&output));
    Some(
        stdout
            .lines()
            .filter_map(|line| line.split_once(": error TS"))
            .map(|(head, rest)| cstr!("{head} TS{}", rest.get(..4).unwrap_or(rest)))
            .collect(),
    )
}

#[test]
fn standard_tsgo_lets_data_and_aria_attrs_fall_through_packaged_generics() {
    let Some(heads) = mapped_heads("packaged-generic-data-attrs-loose-", false) else {
        return;
    };
    // `data-testid`, `:data-index` and `aria-label` are fallthrough attrs;
    // the declared `items` value is still checked through the inferred `T`.
    assert_eq!(heads, ["src/Wrong.vue(6,31) TS2322"]);
}

#[test]
fn standard_tsgo_keeps_strict_data_attrs_closed_on_packaged_generics() {
    let Some(heads) = mapped_heads("packaged-generic-data-attrs-strict-", true) else {
        return;
    };
    assert_eq!(
        heads,
        [
            "src/App.vue(6,49) TS2353",
            "src/App.vue(7,50) TS2353",
            "src/Wrong.vue(6,31) TS2322",
        ],
    );
}
