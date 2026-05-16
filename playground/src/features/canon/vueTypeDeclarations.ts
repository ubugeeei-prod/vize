/**
 * Vue type declarations for Monaco TypeScript language service.
 * Provides type information for Vue APIs, compiler macros, and template globals.
 */
export const VUE_GLOBALS_DECLARATIONS = `
// Vue module declaration
declare module 'vue' {
  // Reactivity: Core
  export function ref<T>(value: T): Ref<T>;
  export function ref<T = any>(): Ref<T | undefined>;
  export function reactive<T extends object>(target: T): T;
  export function readonly<T extends object>(target: T): Readonly<T>;
  export function computed<T>(getter: () => T): ComputedRef<T>;
  export function computed<T>(options: { get: () => T; set: (value: T) => void }): WritableComputedRef<T>;

  // Reactivity: Utilities
  export function unref<T>(ref: T | Ref<T>): T;
  export function toRef<T extends object, K extends keyof T>(object: T, key: K): Ref<T[K]>;
  export function toRefs<T extends object>(object: T): { [K in keyof T]: Ref<T[K]> };
  export function isRef<T>(value: Ref<T> | unknown): value is Ref<T>;
  export function isReactive(value: unknown): boolean;
  export function isReadonly(value: unknown): boolean;
  export function isProxy(value: unknown): boolean;

  // Reactivity: Advanced
  export function shallowRef<T>(value: T): ShallowRef<T>;
  export function triggerRef(ref: ShallowRef): void;
  export function customRef<T>(factory: (track: () => void, trigger: () => void) => { get: () => T; set: (value: T) => void }): Ref<T>;
  export function toRaw<T>(observed: T): T;
  export function markRaw<T extends object>(value: T): T;

  // Lifecycle Hooks
  export function onMounted(callback: () => void): void;
  export function onUnmounted(callback: () => void): void;
  export function onBeforeMount(callback: () => void): void;
  export function onBeforeUnmount(callback: () => void): void;
  export function onUpdated(callback: () => void): void;
  export function onBeforeUpdate(callback: () => void): void;
  export function onActivated(callback: () => void): void;
  export function onDeactivated(callback: () => void): void;
  export function onErrorCaptured(callback: (err: unknown, instance: any, info: string) => boolean | void): void;

  // Watch
  export function watch<T>(source: () => T, callback: (newValue: T, oldValue: T) => void, options?: WatchOptions): () => void;
  export function watch<T>(source: Ref<T>, callback: (newValue: T, oldValue: T) => void, options?: WatchOptions): () => void;
  export function watchEffect(effect: () => void, options?: WatchOptions): () => void;

  // Dependency Injection
  export type InjectionKey<T> = symbol & { readonly __vize_injection?: T };
  export function provide<T>(key: InjectionKey<T> | string | symbol, value: T): void;
  export function inject<T>(key: InjectionKey<T> | string | symbol): T | undefined;
  export function inject<T>(key: InjectionKey<T> | string | symbol, defaultValue: T): T;

  // Misc
  export function nextTick(callback?: () => void): Promise<void>;
  export function getCurrentInstance(): any;

  // Types
  export interface Ref<T = any> {
    value: T;
  }
  export interface ComputedRef<T = any> extends Ref<T> {
    readonly value: T;
  }
  export interface WritableComputedRef<T> extends Ref<T> {}
  export interface ShallowRef<T = any> extends Ref<T> {}
  export type UnwrapRef<T> = T extends Ref<infer V> ? V : T;
  export type Reactive<T> = T;
  export type MaybeRef<T> = T | Ref<T>;

  export interface WatchOptions {
    immediate?: boolean;
    deep?: boolean;
    flush?: 'pre' | 'post' | 'sync';
  }
}

// Vue Compiler Macros (available in <script setup>)
type __EmitShape<T> = T extends (...args: any[]) => any ? T : T extends Record<string, any> ? { [K in keyof T]: T[K] extends (...args: infer A) => any ? A : T[K] extends any[] ? T[K] : any[]; } : Record<string, any[]>;
type __EmitArgs<T, K extends keyof T> = T[K] extends any[] ? T[K] : any[];
type __EmitFn<T> = __EmitShape<T> extends (...args: any[]) => any ? __EmitShape<T> : (<K extends keyof __EmitShape<T>>(event: K, ...args: __EmitArgs<__EmitShape<T>, K>) => void);
type __RuntimePropCtor<T> = T extends readonly (infer U)[] ? __RuntimePropCtor<U> : T extends { type: infer U } ? __RuntimePropCtor<U> : T extends StringConstructor ? string : T extends NumberConstructor ? number : T extends BooleanConstructor ? boolean : T extends ArrayConstructor ? unknown[] : T extends ObjectConstructor ? Record<string, unknown> : T extends DateConstructor ? Date : T extends FunctionConstructor ? (...args: any[]) => any : unknown;
type __RuntimePropResolved<T> = T extends { required: true } ? true : T extends { default: any } ? true : false;
type __RuntimePropShape<T extends Record<string, any>> = { [K in keyof T]: __RuntimePropResolved<T[K]> extends true ? __RuntimePropCtor<T[K]> : __RuntimePropCtor<T[K]> | undefined; };
type __DefaultFactory<T> = (props: any) => T;
type __WithDefaultValue<T> = T | __DefaultFactory<T>;
type __WithDefaultsArgs<T> = { [K in keyof T]?: __WithDefaultValue<T[K]> };
type __WithDefaultsResult<T, D extends __WithDefaultsArgs<T>> = Omit<T, keyof D> & { [K in keyof D & keyof T]-?: T[K] };
declare function defineProps<T>(): Readonly<T>;
declare function defineProps<const T extends readonly string[]>(props: T): Readonly<{ [K in T[number]]?: any }>;
declare function defineProps<const T extends Record<string, any>>(props: T): Readonly<__RuntimePropShape<T>>;
declare function defineEmits<T>(): __EmitFn<T>;
declare function defineEmits<const T extends readonly string[]>(events: T): (event: T[number], ...args: any[]) => void;
declare function defineEmits<const T extends Record<string, any>>(events: T): __EmitFn<T>;
declare function defineExpose<T>(exposed?: T): void;
declare function defineOptions<T>(options: T): void;
declare function defineSlots<T>(): T;
declare function defineModel<T>(name?: string, options?: { required?: boolean; default?: T }): import('vue').Ref<T>;
declare function withDefaults<T, D extends __WithDefaultsArgs<T>>(props: T, defaults: D): __WithDefaultsResult<T, D>;

// Vue Global Instance Properties (available in templates)
declare const $attrs: Record<string, unknown>;
declare const $slots: Record<string, (...args: any[]) => any>;
declare const $refs: Record<string, any>;
declare const $el: HTMLElement | undefined;
declare const $parent: any;
declare const $root: any;
declare const $emit: (...args: any[]) => void;
declare const $forceUpdate: () => void;
declare const $nextTick: (callback?: () => void) => Promise<void>;

// Event handler context
declare const $event: Event;
`;
