pub(super) const VUE_FACADE_PACKAGE_JSON: &str = r#"{
  "name": "vue",
  "types": "index.d.ts"
}
"#;

pub(super) const VUE_FACADE_TYPES: &str = r#"export * from "@vue/runtime-dom";
"#;

/// `vue/jsx-runtime` for projects that resolve no real `vue` package.
///
/// A `.tsx` consumer compiled with `jsxImportSource: "vue"` resolves its JSX
/// namespace through this subpath. Without it the whole file falls back to the
/// ambient `JSX` namespace, which drops `ElementAttributesProperty` and makes
/// every component element report spurious prop errors instead of checking
/// against the component's `$props`.
pub(crate) const VUE_FACADE_JSX_RUNTIME_TYPES: &str = r#"import type { AllowedComponentProps, NativeElements, VNodeProps } from "./index";

export declare function jsx(type: any, props: any, key?: any): any;
export declare function jsxs(type: any, props: any, key?: any): any;
export declare function jsxDEV(type: any, props: any, key?: any, isStatic?: boolean, source?: any, self?: any): any;
export declare const Fragment: any;

export namespace JSX {
  interface Element {}
  interface ElementClass {
    $props: {};
  }
  interface ElementAttributesProperty {
    $props: {};
  }
  interface IntrinsicElements extends NativeElements {
    [name: string]: any;
  }
  interface IntrinsicAttributes extends VNodeProps, AllowedComponentProps {}
}
"#;

/// `vue/jsx` for projects that resolve no real `vue` package.
///
/// The virtual project references this type package whenever the tsconfig
/// selects Vue's JSX (`jsxImportSource: "vue"` or an explicit `vue/jsx` type
/// entry), so a missing file is a hard `TS2688` on the shared helpers file.
pub(crate) const VUE_FACADE_JSX_GLOBAL_TYPES: &str = r#"import type { AllowedComponentProps, NativeElements, VNodeProps } from "./index";

declare global {
  namespace JSX {
    interface Element {}
    interface ElementClass {
      $props: {};
    }
    interface ElementAttributesProperty {
      $props: {};
    }
    interface IntrinsicElements extends NativeElements {
      [name: string]: any;
    }
    interface IntrinsicAttributes extends VNodeProps, AllowedComponentProps {}
  }
}

export {};
"#;

pub(super) const VUE_RUNTIME_DOM_STUB_PACKAGE_JSON: &str = r#"{
  "name": "@vue/runtime-dom",
  "types": "index.d.ts"
}
"#;

pub(super) const VUE_RUNTIME_CORE_STUB_PACKAGE_JSON: &str = r#"{
  "name": "@vue/runtime-core",
  "types": "index.d.ts"
}
"#;

pub(crate) const VUE_RUNTIME_CORE_STUB_TYPES: &str = r#"export interface ComponentCustomProps {}
"#;

pub(crate) const VUE_RUNTIME_DOM_STUB_TYPES: &str = r#"import type { ComponentCustomProps as RuntimeCoreComponentCustomProps } from "@vue/runtime-core";

export interface ComponentCustomProperties {}

export interface ComponentPublicInstance<Props = {}> extends ComponentCustomProperties {
  $props: Props;
  $attrs: { [key: string]: unknown };
  $slots: { [key: string]: unknown };
  $refs: { [key: string]: unknown };
  $emit: (...args: any[]) => void;
}

export interface VNodeProps {
  key?: PropertyKey;
  ref?: unknown;
  ref_for?: boolean;
  ref_key?: string;
  onVnodeBeforeMount?: unknown;
  onVnodeMounted?: unknown;
  onVnodeBeforeUpdate?: unknown;
  onVnodeUpdated?: unknown;
  onVnodeBeforeUnmount?: unknown;
  onVnodeUnmounted?: unknown;
}

export interface AllowedComponentProps {
  class?: unknown;
  style?: unknown;
}

export interface ComponentCustomProps extends RuntimeCoreComponentCustomProps {}

export type PublicProps = VNodeProps & AllowedComponentProps & ComponentCustomProps;

export type NativeElements = Record<string, Record<string, unknown>>;

export type DefineComponent<
  Props = {},
  RawBindings = {},
  D = {},
  C = {},
  M = {},
  Mixin = {},
  Extends = {},
  E = {},
  EE = string,
  PP = Props,
  PropsDefaults = {},
  MakeDefaultsOptional = true,
  Options = {},
  S = {}
> = {
  new (): ComponentPublicInstance<Props>;
} & ComponentOptions<Props, RawBindings, D, C, M>;

export type ComponentOptions<
  Props = {},
  RawBindings = any,
  D = any,
  C = any,
  M = any
> = {
  name?: string;
  __name?: string;
  __file?: string;
  __vccOpts?: any;
  props?: any;
  emits?: any;
  slots?: any;
  setup?: any;
  render?: Function;
  components?: any;
  directives?: any;
  inheritAttrs?: boolean;
  compatConfig?: any;
  call?: (this: unknown, ...args: unknown[]) => never;
  __isFragment?: never;
  __isTeleport?: never;
  __isSuspense?: never;
  __defaults?: any;
  __vapor?: boolean;
  __multiRoot?: boolean;
  __isKeepAlive?: boolean;
  __isBuiltIn?: boolean;
};

export interface FunctionalComponent<
  P = {},
  E = {},
  S = any
> {
  (props: P, ctx: any): any;
  props?: any;
  emits?: any;
  slots?: any;
  inheritAttrs?: boolean;
  displayName?: string;
  compatConfig?: any;
}

export type ConcreteComponent<
  Props = {},
  RawBindings = any,
  D = any,
  C = any,
  M = any,
  E = {},
  S = any
> = ComponentOptions<Props, RawBindings, D, C, M> | FunctionalComponent<Props, E, S>;

declare const RefSymbol: unique symbol;

export interface Ref<T = unknown, _Raw = T> {
  value: T;
  [RefSymbol]: true;
}

export interface ComputedRef<T = unknown> extends Ref<T> {
  readonly value: T;
}

export interface WritableComputedRef<T = unknown> extends Ref<T> {
  value: T;
}

export interface ShallowRef<T = unknown, _Raw = T> extends Ref<T, _Raw> {
  readonly __v_isShallow?: true;
}

export type InjectionKey<T> = symbol & { readonly __v_vlsInjection?: T };
export type PropType<T> = { new (...args: any[]): T & {} } | { (): T } | null;

export interface DirectiveBinding<Value = any, Modifiers extends string = string, Arg = any> {
  instance: ComponentPublicInstance | Record<string, any> | null;
  value: Value;
  oldValue: Value | null;
  arg?: Arg;
  modifiers: Partial<Record<Modifiers, boolean>>;
  dir: ObjectDirective<any, Value, Modifiers, Arg>;
}
export type DirectiveHook<HostElement = any, Prev = any, Value = any, Modifiers extends string = string, Arg = any> = (el: HostElement, binding: DirectiveBinding<Value, Modifiers, Arg>, vnode: any, prevVNode: Prev) => void;
export interface ObjectDirective<HostElement = any, Value = any, Modifiers extends string = string, Arg = any> {
  created?: DirectiveHook<HostElement, null, Value, Modifiers, Arg>;
  beforeMount?: DirectiveHook<HostElement, null, Value, Modifiers, Arg>;
  mounted?: DirectiveHook<HostElement, null, Value, Modifiers, Arg>;
  beforeUpdate?: DirectiveHook<HostElement, any, Value, Modifiers, Arg>;
  updated?: DirectiveHook<HostElement, any, Value, Modifiers, Arg>;
  beforeUnmount?: DirectiveHook<HostElement, null, Value, Modifiers, Arg>;
  unmounted?: DirectiveHook<HostElement, null, Value, Modifiers, Arg>;
  deep?: boolean;
}
export type FunctionDirective<HostElement = any, V = any, Modifiers extends string = string, Arg = any> = DirectiveHook<HostElement, any, V, Modifiers, Arg>;
export type Directive<HostElement = any, Value = any, Modifiers extends string = string, Arg = any> = ObjectDirective<HostElement, Value, Modifiers, Arg> | FunctionDirective<HostElement, Value, Modifiers, Arg>;
export type DirectiveModifiers<K extends string = string> = Partial<Record<K, boolean>>;

export declare const Transition: DefineComponent;
export declare function defineComponent(options: any): DefineComponent;
export declare function defineAsyncComponent(source: any): DefineComponent;
export declare function defineProps<T = {}>(): T;
export declare function computed<T>(getter: () => T): ComputedRef<T>;
export declare function computed<T>(options: { get: () => T; set: (value: T) => void }): WritableComputedRef<T>;
export declare function ref<T>(value: T): Ref<T>;
export declare function reactive<T extends object>(target: T): T;
export declare function shallowRef<T>(value: T): ShallowRef<T>;
export declare function toRef<T extends object, K extends keyof T>(object: T, key: K): Ref<T[K]>;
export declare function useTemplateRef<T = unknown>(key: string): ShallowRef<T | null>;
export declare function useCssModule(name?: string): Record<string, string>;
export declare function useId(): string;
export declare function watch<T>(source: T, callback: (...args: any[]) => void, options?: any): void;
export declare function watchEffect(effect: (onCleanup: (cleanupFn: () => void) => void) => void): void;
export declare function onMounted(callback: () => void): void;
export declare function customRef<T>(factory: any): Ref<T>;
export declare function provide<T>(key: InjectionKey<T> | string | symbol, value: T): void;
export declare function inject<T>(key: InjectionKey<T> | string | symbol): T | undefined;
export declare function inject<T>(key: InjectionKey<T> | string | symbol, defaultValue: T): T;
export declare function markRaw<T extends object>(value: T): T;
export declare function createApp(root: any): {
  config: { globalProperties: { [key: string]: any }; };
  mount(container: string | Element): ComponentPublicInstance; unmount(): void; use(plugin: any, ...options: any[]): any;
};
"#;

pub(super) const VITE_STUB_PACKAGE_JSON: &str = r#"{
  "name": "vite",
  "types": "client.d.ts"
}
"#;

pub(super) const VITE_CLIENT_STUB: &str = r#"interface ImportMetaEnv {
  readonly [key: string]: string | boolean | undefined;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

export {};
"#;
