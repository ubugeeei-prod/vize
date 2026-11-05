// Generated from the canonical emit-inference helpers. Do not edit.
import type {
  IsAny as $IsAny,
  $ConstructorEmitPropsFactory,
  $EmitOwnedKeys,
  __EmitProps,
} from "./virtual-ts.js" with { "resolution-mode": "import" };
type $LooseEmitProps = { readonly __vizeLooseListener: [] };
type $DeferredEmitProps<E, P = __EmitProps<E>> = `on${Capitalize<string>}` extends keyof P
  ? $EmitIsWide<new () => { $emit: E }> extends true
    ? $LooseEmitProps
    : P
  : P;
type $LooseEmitPropsFactory<C> = C extends (props: infer P, ...args: any[]) => any
  ? $IsAny<P> extends true
    ? (component: C) => (props: {}) => $LooseEmitProps
    : (component: C) => (props: P) => {}
  : (component: C) => (props: {}) => $LooseEmitProps;
type $EmitIsWide<C> = C extends new (...args: any[]) => { $emit: infer E }
  ? $IsAny<E> extends true
    ? true
    : E extends (...args: infer A) => any
      ? string extends A[0]
        ? true
        : [A] extends [any[]]
          ? any[] extends A
            ? true
            : false
          : false
      : false
  : false;
type $NativeEmitPropsFactory<C> = C extends new (...args: any[]) => { $emit: any }
  ? C extends { readonly __vizeComponentMarker: true }
    ? $ConstructorEmitPropsFactory<
        C extends { readonly __vizeFallthroughProps?: infer F } ? F : {},
        $EmitOwnedKeys<C>
      >
    : <P, E>(
        component: new (props: P, ...args: any[]) => { $emit: E },
      ) => (props: P) => $DeferredEmitProps<E>
  : $LooseEmitPropsFactory<C>;
type $EmitPropsFactory<C> =
  $IsAny<C> extends true
    ? (component: C) => (props: {}) => $LooseEmitProps
    : C extends { __vizeResolveEmitProps?: infer F }
      ? F extends (...args: any[]) => any
        ? (component: C) => F
        : $NativeEmitPropsFactory<C>
      : C extends { __vizeResolveProps?: infer F }
        ? F extends (...args: any[]) => any
          ? (component: C) => F
          : $NativeEmitPropsFactory<C>
        : $NativeEmitPropsFactory<C>;
export type { $NativeEmitPropsFactory, $EmitPropsFactory };
