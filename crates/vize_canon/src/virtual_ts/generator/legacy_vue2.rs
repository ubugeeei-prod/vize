use vize_carton::config::VueVersion;
use vize_croquis::Croquis;

use super::super::helpers::VUE_TYPE_HELPERS;
use super::super::types::{VirtualTsGenerationOptions, VirtualTsOptions, VirtualTsOutput};
use super::generate_virtual_ts_with_offsets_and_checks;
use super::spans::DEFINE_COMPONENT_HELPER;

const LEGACY_VUE_TYPE_HELPERS: &str = r#"type __EmitShape<T> = T extends (...args: any[]) => any ? T : T extends Record<string, any> ? { [K in keyof T]: T[K] extends (...args: infer A) => any ? A : T[K] extends any[] ? T[K] : any[]; } : Record<string, any[]>;
type __EmitArgs<T, K extends keyof T> = T[K] extends any[] ? T[K] : any[];
type __EmitFn<T, __S = __EmitShape<T>, __K extends keyof __S & string = keyof __S & string, __U = { [K in __K]: (event: K, ...args: __EmitArgs<__S, K>) => void }[__K]> = __S extends (...args: any[]) => any ? __S : [__K] extends [never] ? (event: never, ...args: any[]) => void : (__U extends unknown ? (fn: __U) => void : never) extends (fn: infer __I) => void ? __I : never;
type __RuntimePropValue<T> = T extends abstract new (...args: any[]) => infer V ? V : T extends (...args: any[]) => infer V ? V : never;
type __RuntimePropCtorInner<T> = T extends null | undefined ? never : T extends readonly (infer U)[] ? __RuntimePropCtorInner<U> : T extends { type: infer U } ? __RuntimePropCtorInner<U> : T extends StringConstructor ? string : T extends NumberConstructor ? number : T extends BooleanConstructor ? boolean : T extends ArrayConstructor ? unknown[] : T extends ObjectConstructor ? Record<string, any> : T extends DateConstructor ? Date : T extends FunctionConstructor ? (...args: any[]) => any : __RuntimePropValue<T>;
type __RuntimePropCtor<T> = [__RuntimePropCtorInner<T>] extends [never] ? unknown : __RuntimePropCtorInner<T>;
type __RuntimePropHasBoolean<T> = T extends BooleanConstructor ? true : T extends readonly (infer U)[] ? __RuntimePropHasBoolean<U> : T extends { type: infer U } ? __RuntimePropHasBoolean<U> : false;
type __RuntimePropResolved<T> = T extends { required: true } ? true : T extends { default: any } ? true : __RuntimePropHasBoolean<T>;
type __RuntimePropShape<T extends Record<string, any>> = { [K in keyof T]: __RuntimePropResolved<T[K]> extends true ? __RuntimePropCtor<T[K]> : __RuntimePropCtor<T[K]> | undefined; };
type __VizeIsAny<T> = 0 extends (1 & T) ? true : false;
type __DefaultFactory<T> = (props: any) => T; type __WithDefaultValue<T> = T | __DefaultFactory<T>;
type __LooseRequired<T> = { [P in keyof (T & Required<T>)]: T[P] };
type __VizeBooleanKey<T, K extends keyof T = keyof T> = K extends any ? [Exclude<T[K], undefined>] extends [never] ? never : [Exclude<T[K], undefined>] extends [boolean] ? K : never : never;
type __DefineProps<T, __BKeys extends keyof T = never> = __LooseRequired<T>;
type __VizePrettify<T> = (T extends any ? { [K in keyof T]: T[K] } : { [K in keyof T as K]: T[K] }) & {};
type __WithDefaultsArgs<T> = { [K in keyof T]?: __WithDefaultValue<T[K]> };
type __WithDefaultsResult<T, D, __BKeys extends keyof T = never, __Props = __LooseRequired<T>> = T extends unknown ? Omit<__Props, keyof D> & { [K in keyof D & keyof T]-?: [D[K]] extends [undefined] ? __LooseRequired<T>[K] : Exclude<__Props[K & keyof __Props], undefined> } : never;
type __Ref<T> = { value: T };
type __ShallowRef<T> = __Ref<T> & { readonly __v_isShallow?: true };
type __VizeKebabCase<S extends string> = S extends `${infer Head}${infer Tail}` ? Head extends Lowercase<Head> ? `${Head}${__VizeKebabCase<Tail>}` : `-${Lowercase<Head>}${__VizeKebabCase<Tail>}` : S;
type __VizeKebabProps<T> = { [K in keyof T & string as __VizeKebabCase<K>]: T[K] };
type __VizeComponentProps<T> = T extends unknown ? T & Partial<__VizeKebabProps<T>> : never;
type __VizeVue2LooseEventArg<T> = __VizeIsAny<T> extends true ? any : [T] extends [Object] ? ([Object] extends [T] ? any : T) : T;
declare type __VizeVue2LooseEmitArgs<A extends readonly unknown[]> = { [K in keyof A]: __VizeVue2LooseEventArg<A[K]> };
type __VForEntry<T> = T extends number ? [item: number, key: number, index: number] : T extends string ? [item: string, key: number, index: number] : T extends readonly (infer U)[] ? [item: U, key: number, index: number] : T extends Iterable<infer U> ? [item: U, key: number, index: number] : [item: T[keyof T], key: keyof T extends string ? keyof T : `${keyof T & (string | number)}`, index: number];
declare function __vForList<const T>(source: T | undefined | null): readonly __VForEntry<NonNullable<T>>[];"#;
const LEGACY_REF_UNWRAP_HELPER: &str =
    "    type __U<T> = T extends { value: infer __V } ? __V : T;\n";
const LEGACY_EXPOSED_UNWRAP_HELPER: &str = "type __VizeShallowUnwrapRef<T> = { [K in keyof T]: T[K] extends { value: infer __V } ? __V : T[K] };\n";
/// Template-scope ref unwrapping for the modern (Vue 3) dialect when the shared
/// preamble is *not* hoisted (check server, content mapper).
///
/// The widening conditional types stay here, inside `__template()`, instead of
/// joining the module-scope preamble: `__U` is their only reference. Module-scope
/// declarations are module-local, so an unused one surfaces to the user as a
/// `TS6196` hint on their own file.
/// Check for a `value` property before referring to `vue.Ref`: when Vue types
/// are temporarily unavailable, that import can resolve to `any` and would
/// otherwise erase ordinary imported function signatures in template scope.
const MODERN_REF_UNWRAP_HELPER: &str = r#"    type __VizeIsUnion<T, __U = T> = T extends unknown ? ([__U] extends [T] ? false : true) : false;
    type __VizeWidenTemplateRef<T> = __VizeIsAny<T> extends true ? T : __VizeIsUnion<T> extends true ? T : T extends string ? string extends T ? string : T : T extends number ? number extends T ? number : T : T extends boolean ? boolean extends T ? boolean : T : T;
    type __U<T> = T extends { value: unknown } ? T extends import('vue').Ref ? __VizeWidenTemplateRef<T['value']> : T : T;
"#;
/// [`MODERN_REF_UNWRAP_HELPER`] for the hoisted path: the ambient helpers file
/// (`SHARED_PREAMBLE_DTS`) is a `.d.ts` global script, so it declares the two
/// widening types once per program and never reports them unused.
///
/// `__U` itself stays per file — it is dialect-dependent. Hoisting the two
/// aliases saves 369 bytes in every generated `.vue.ts` and stops TypeScript
/// instantiating a distinct declaration per file instead of caching one
/// (#3443, #3460).
const MODERN_HOISTED_REF_UNWRAP_HELPER: &str = "    type __U<T> = T extends { value: unknown } ? T extends import('vue').Ref ? __VizeWidenTemplateRef<T['value']> : T : T;\n";
const MODERN_GENERIC_REF_UNWRAP_HELPER: &str = "    type __U<T> = T extends { value: unknown } ? T extends import('vue').Ref<any> ? T['value'] : T : T;\n";
const MODERN_EXPOSED_UNWRAP_HELPER: &str = "type __VizeShallowUnwrapRef<T> = { [K in keyof T]: T[K] extends import('vue').Ref<infer __V> ? __V : T[K] };\n";
const LEGACY_DEFINE_COMPONENT_HELPER: &str = r#"type __VizeNuxt2Context = {
  app: any;
  route: any;
  params: Record<string, string>;
  query: Record<string, string | string[] | undefined>;
  store: any;
  error: (...args: any[]) => any;
  redirect: (...args: any[]) => any;
  req?: any;
  res?: any;
  env?: Record<string, unknown>;
  isDev?: boolean;
  isHMR?: boolean;
} & Record<string, any>;
type __VizeNuxt2PageOptions = ThisType<any> & {
  validate?: (context: __VizeNuxt2Context) => unknown;
  asyncData?: (context: __VizeNuxt2Context) => any;
  fetch?: (context: __VizeNuxt2Context) => any;
  head?: any;
  layout?: any;
  layoutTransition?: any;
  loading?: any;
  middleware?: any;
  scrollToTop?: any;
  transition?: any;
};
declare function __vizeDefineComponent<T>(options: T & __VizeNuxt2PageOptions): T;
"#;
pub(super) const LEGACY_COMPONENT_INSTANCE_HELPER: &str = r#"type __VizeVue2ComponentInstance = {
  $el: Element;
  $refs: Record<string, any>;
  $attrs: Record<string, any>;
  $listeners: Record<string, any>;
  $children: any[];
  $scopedSlots: Record<string, any>;
  $parent: any;
  $root: any;
  $isServer: boolean; $vnode: any; $ssrContext: any;
  $options: Record<string, any>;
  $data: Record<string, any>;
  $mount: (...args: any[]) => any;
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

pub(super) fn ref_unwrap_helper(
    legacy_vue2: bool,
    dialect: VueVersion,
    hoist_shared_preamble: bool,
) -> &'static str {
    if needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        LEGACY_REF_UNWRAP_HELPER
    } else if hoist_shared_preamble {
        MODERN_HOISTED_REF_UNWRAP_HELPER
    } else {
        MODERN_REF_UNWRAP_HELPER
    }
}

pub(super) fn ref_unwrap_helper_for_template(
    legacy_vue2: bool,
    dialect: VueVersion,
    has_generic_param: bool,
    hoist_shared_preamble: bool,
) -> &'static str {
    if has_generic_param && !needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        MODERN_GENERIC_REF_UNWRAP_HELPER
    } else {
        ref_unwrap_helper(legacy_vue2, dialect, hoist_shared_preamble)
    }
}

pub(super) fn exposed_unwrap_helper(legacy_vue2: bool, dialect: VueVersion) -> &'static str {
    if needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        LEGACY_EXPOSED_UNWRAP_HELPER
    } else {
        MODERN_EXPOSED_UNWRAP_HELPER
    }
}

pub(super) fn define_component_helper(legacy_vue2: bool, dialect: VueVersion) -> &'static str {
    if needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        LEGACY_DEFINE_COMPONENT_HELPER
    } else {
        DEFINE_COMPONENT_HELPER
    }
}

mod instance;
pub(super) use instance::{generic_instance_suffix, instance_helper, instance_suffix};

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
mod tests;
