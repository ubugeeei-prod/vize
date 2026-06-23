use vize_carton::config::VueVersion;
use vize_croquis::Croquis;

use super::super::helpers::VUE_TYPE_HELPERS;
use super::super::types::{VirtualTsGenerationOptions, VirtualTsOptions, VirtualTsOutput};
use super::generate_virtual_ts_with_offsets_and_checks;
use super::spans::DEFINE_COMPONENT_HELPER;

const LEGACY_VUE_TYPE_HELPERS: &str = r#"type __EmitShape<T> = T extends (...args: any[]) => any ? T : T extends Record<string, any> ? { [K in keyof T]: T[K] extends (...args: infer A) => any ? A : T[K] extends any[] ? T[K] : any[]; } : Record<string, any[]>;
type __EmitArgs<T, K extends keyof T> = T[K] extends any[] ? T[K] : any[];
type __EmitFn<T> = __EmitShape<T> extends (...args: any[]) => any ? __EmitShape<T> : (<K extends keyof __EmitShape<T>>(event: K, ...args: __EmitArgs<__EmitShape<T>, K>) => void);
type __RuntimePropValue<T> = T extends { new (...args: any[]): infer V } ? V : T extends { (): infer V } ? V : never;
type __RuntimePropCtorInner<T> = T extends null | undefined ? never : T extends readonly (infer U)[] ? __RuntimePropCtorInner<U> : T extends { type: infer U } ? __RuntimePropCtorInner<U> : T extends StringConstructor ? string : T extends NumberConstructor ? number : T extends BooleanConstructor ? boolean : T extends ArrayConstructor ? unknown[] : T extends ObjectConstructor ? Record<string, unknown> : T extends DateConstructor ? Date : T extends FunctionConstructor ? (...args: any[]) => any : __RuntimePropValue<T>;
type __RuntimePropCtor<T> = [__RuntimePropCtorInner<T>] extends [never] ? unknown : __RuntimePropCtorInner<T>;
type __RuntimePropResolved<T> = T extends { required: true } ? true : T extends { default: any } ? true : false;
type __RuntimePropShape<T extends Record<string, any>> = { [K in keyof T]: __RuntimePropResolved<T[K]> extends true ? __RuntimePropCtor<T[K]> : __RuntimePropCtor<T[K]> | undefined; };
type __DefaultFactory<T> = (props: any) => T;
type __WithDefaultValue<T> = T | __DefaultFactory<T>;
type __WithDefaultsArgs<T> = { [K in keyof T]?: __WithDefaultValue<T[K]> };
type __WithDefaultsResult<T, D extends __WithDefaultsArgs<T>> = Omit<T, keyof D> & Required<Pick<T, keyof D & keyof T>>;
type __Ref<T> = { value: T };
type __ShallowRef<T> = __Ref<T> & { readonly __v_isShallow?: true };
type __VizeKebabCase<S extends string> = S extends `${infer Head}${infer Tail}` ? Head extends Lowercase<Head> ? `${Head}${__VizeKebabCase<Tail>}` : `-${Lowercase<Head>}${__VizeKebabCase<Tail>}` : S;
type __VizeKebabProps<T> = { [K in keyof T & string as __VizeKebabCase<K>]: T[K] };
type __VizeComponentProps<T> = T extends unknown ? T & Partial<__VizeKebabProps<T>> : never;
type __VizeIsAny<T> = 0 extends (1 & T) ? true : false;
type __VForEntry<T> = __VizeIsAny<T> extends true ? [item: any, key: number, index: number] : T extends readonly (infer U)[] ? [item: U, key: number, index: number] : T extends number ? [item: number, key: number, index: number] : T extends string ? [item: string, key: number, index: number] : T extends Iterable<infer U> ? [item: U, key: number, index: number] : T extends object ? [item: T[keyof T], key: keyof T, index: number] : [item: any, key: number, index: number];
declare function __vForList<T>(source: T | undefined | null): readonly __VForEntry<NonNullable<T>>[];"#;
const LEGACY_REF_UNWRAP_HELPER: &str =
    "    type __U<T> = T extends { value: infer __V } ? __V : T;\n";
const MODERN_REF_UNWRAP_HELPER: &str = r#"    type __VizeIsUnion<T, __U = T> = T extends unknown ? ([__U] extends [T] ? false : true) : false;
    type __VizeWidenTemplateRef<T> = __VizeIsUnion<T> extends true ? T : T extends string ? string : T extends number ? number : T extends boolean ? boolean : T;
    type __U<T> = T extends import('vue').Ref ? __VizeWidenTemplateRef<T['value']> : T;
"#;
const LEGACY_DEFINE_COMPONENT_HELPER: &str =
    "declare function __vizeDefineComponent<T>(options: T): T;\n";
pub(super) const LEGACY_COMPONENT_INSTANCE_HELPER: &str = r#"type __VizeVue2ComponentInstance = {
  $el: Element;
  $refs: Record<string, any>;
  $attrs: Record<string, unknown>;
  $listeners: Record<string, unknown>;
  $children: any[];
  $scopedSlots: Record<string, unknown>;
  $parent: any;
  $root: any;
  $options: Record<string, any>;
  $data: Record<string, any>;
  $on: (...args: any[]) => any;
  $off: (...args: any[]) => any;
  $once: (...args: any[]) => any;
  $set: (...args: any[]) => any;
  $delete: (...args: any[]) => any;
  $watch: (...args: any[]) => any;
  $nextTick: (...args: any[]) => any;
  $forceUpdate: () => void;
  $destroy: () => void;
  $createElement: (...args: any[]) => any;
  _c: (...args: any[]) => any;
};
"#;

pub(super) fn needs_legacy_vue2_helpers(legacy_vue2: bool, dialect: VueVersion) -> bool {
    legacy_vue2 || matches!(dialect, VueVersion::V2 | VueVersion::V2_7)
}

pub(super) fn vue_type_helpers(legacy_vue2: bool, dialect: VueVersion) -> &'static str {
    if needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        LEGACY_VUE_TYPE_HELPERS
    } else {
        VUE_TYPE_HELPERS
    }
}

pub(super) fn ref_unwrap_helper(legacy_vue2: bool, dialect: VueVersion) -> &'static str {
    if needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        LEGACY_REF_UNWRAP_HELPER
    } else {
        MODERN_REF_UNWRAP_HELPER
    }
}

pub(super) fn define_component_helper(legacy_vue2: bool, dialect: VueVersion) -> &'static str {
    if needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        LEGACY_DEFINE_COMPONENT_HELPER
    } else {
        DEFINE_COMPONENT_HELPER
    }
}

pub(super) fn instance_helper(legacy_vue2: bool, dialect: VueVersion) -> &'static str {
    if needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        LEGACY_COMPONENT_INSTANCE_HELPER
    } else {
        ""
    }
}

pub(super) fn instance_suffix(
    legacy_vue2: bool,
    dialect: VueVersion,
    has_exposed_type: bool,
) -> &'static str {
    match (
        needs_legacy_vue2_helpers(legacy_vue2, dialect),
        has_exposed_type,
    ) {
        (true, true) => "} & __VizeVue2ComponentInstance & Exposed;\n",
        (true, false) => "} & __VizeVue2ComponentInstance;\n",
        (false, true) => "} & Exposed;\n",
        (false, false) => "};\n",
    }
}

/// Generate virtual TypeScript with Vue 2.7 / Nuxt 2 compatibility enabled.
pub fn generate_virtual_ts_with_offsets_legacy_vue2(
    summary: &Croquis,
    script_content: Option<&str>,
    template_ast: Option<&vize_relief::RootNode<'_>>,
    script_offset: u32,
    template_offset: u32,
    options: &VirtualTsOptions,
) -> VirtualTsOutput {
    generate_virtual_ts_with_offsets_and_checks(
        summary,
        script_content,
        template_ast,
        script_offset,
        template_offset,
        options,
        VirtualTsGenerationOptions {
            legacy_vue2: true,
            ..Default::default()
        },
    )
}

#[cfg(test)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        process::Command,
    };

    use vize_carton::{String, append, config::VueVersion, cstr};

    use super::vue_type_helpers;

    #[test]
    fn v_for_helper_preserves_member_array_aliases_without_losing_object_keys() {
        let Some(tsgo) = resolve_test_tsgo_binary() else {
            return;
        };
        let case_dir = std::env::temp_dir()
            .join(cstr!("vize-legacy-vfor-helper-{}", std::process::id()).as_str());
        let _ = std::fs::remove_dir_all(&case_dir);
        std::fs::create_dir_all(&case_dir).unwrap();
        let test_path = case_dir.join("vfor-helper.ts");
        let mut source = String::new("");
        append!(source, "{}\n\n", vue_type_helpers(true, VueVersion::V2));
        source.push_str(
            r#"interface QuestionField {
  questionFieldInputType: string;
  question: string;
  type: string;
}
interface QuestionFormat {
  id: string;
  questionField: QuestionField[];
}

declare const anySource: any;
for (const [answer, i] of __vForList(anySource.questionField)) {
  const keyText = `${i}question`;
  answer.questionFieldInputType;
  void keyText;
}

const obj: { foo: number; bar: number } = { foo: 1, bar: 2 };
for (const [value, key] of __vForList(obj)) {
  const valueCheck: number = value;
  const keyCheck: "foo" | "bar" = key;
  void valueCheck;
  void keyCheck;
}

declare const questionFormat: QuestionFormat;
for (const [answer, i] of __vForList(questionFormat.questionField)) {
  const inputType: string = answer.questionFieldInputType;
  const index: number = i;
  void inputType;
  void index;
}
"#,
        );
        std::fs::write(&test_path, source.as_bytes()).unwrap();

        let output = Command::new(tsgo)
            .args([
                "--ignoreConfig",
                "--noEmit",
                "--strict",
                "--target",
                "ES2022",
            ])
            .arg(&test_path)
            .output()
            .unwrap();
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        assert!(
            output.status.success(),
            "legacy v-for helper should keep any/member-array aliases off the object overload while preserving object key types\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );

        let _ = std::fs::remove_dir_all(&case_dir);
    }

    fn resolve_test_tsgo_binary() -> Option<PathBuf> {
        if let Ok(path) = std::env::var("CORSA_PATH")
            && Path::new(&path).exists()
        {
            return Some(PathBuf::from(path));
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)?;
        [
            root.join("node_modules/.bin/tsgo"),
            root.join("examples/vite-musea/node_modules/.bin/tsgo"),
        ]
        .into_iter()
        .find(|candidate| candidate.exists())
    }
}
