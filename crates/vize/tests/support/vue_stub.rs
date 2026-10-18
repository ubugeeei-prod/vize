#![cfg(test)]
use std::path::Path;

pub fn install_vue_jsx_type_stub(project_root: &Path) {
    let node_modules = project_root.join("node_modules");
    if let Ok(metadata) = std::fs::symlink_metadata(&node_modules) {
        if metadata.file_type().is_symlink() || metadata.is_file() {
            std::fs::remove_file(&node_modules).unwrap();
        } else {
            std::fs::remove_dir_all(&node_modules).unwrap();
        }
    }
    let vue_dir = node_modules.join("vue");
    std::fs::create_dir_all(&vue_dir).unwrap();
    std::fs::write(
        vue_dir.join("package.json"),
        r#"{
  "name": "vue",
  "types": "index.d.ts"
}"#,
    )
    .unwrap();
    std::fs::write(
        vue_dir.join("index.d.ts"),
        r#"export type Slot = (...args: any[]) => unknown[];
export type Slots = Readonly<Record<string, Slot | undefined>>;
export interface ComponentCustomProperties {}

export interface VNode {
  readonly __v_isVNode: true;
}
// Public input contracts mirror Vue 3.5.20; VNode remains opaque in these fixtures.
type VNodeMountHook = (vnode: VNode) => void;
type VNodeUpdateHook = (vnode: VNode, oldVNode: VNode) => void;
type VNodeRef = string | Ref | ((ref: Element | ComponentPublicInstance | null, refs: Record<string, unknown>) => void);
export type VNodeProps = {
  key?: PropertyKey;
  ref?: VNodeRef;
  ref_for?: boolean;
  ref_key?: string;
  onVnodeBeforeMount?: VNodeMountHook | VNodeMountHook[];
  onVnodeMounted?: VNodeMountHook | VNodeMountHook[];
  onVnodeBeforeUpdate?: VNodeUpdateHook | VNodeUpdateHook[];
  onVnodeUpdated?: VNodeUpdateHook | VNodeUpdateHook[];
  onVnodeBeforeUnmount?: VNodeMountHook | VNodeMountHook[];
  onVnodeUnmounted?: VNodeMountHook | VNodeMountHook[];
};
export interface AllowedComponentProps {
  class?: unknown;
  style?: unknown;
}
export interface ComponentCustomProps {}
export type PublicProps = VNodeProps & AllowedComponentProps & ComponentCustomProps;

export interface ComponentPublicInstance<Props = {}> extends ComponentCustomProperties {
  $props: Props;
  $attrs: Record<string, unknown>;
  $slots: Record<string, unknown>;
  $refs: Record<string, unknown>;
  $emit: (...args: any[]) => void;
}

export type DefineComponent<Props = {}> = {
  new (): ComponentPublicInstance<Props>;
};

export type ComponentOptions<Props = {}> = {
  props?: Props;
  setup?: any;
  render?: Function;
};

export interface FunctionalComponent<P = {}> {
  (props: P, ctx: any): any;
}

export type ConcreteComponent<Props = {}> =
  | ComponentOptions<Props>
  | FunctionalComponent<Props>;

declare const RefSymbol: unique symbol;

export interface Ref<T = unknown, _Raw = T> {
  value: T;
  [RefSymbol]: true;
}

export interface ShallowRef<T = unknown, _Raw = T> extends Ref<T, _Raw> {
  readonly __v_isShallow?: true;
}

export type PropType<T> = { new (...args: any[]): T & {} } | { (): T } | null;

export declare const Transition: DefineComponent;
export declare function defineComponent(options: any): DefineComponent;
"#,
    )
    .unwrap();
    std::fs::write(
        vue_dir.join("jsx.d.ts"),
        r#"declare global {
  namespace JSX {
    interface IntrinsicElements {
      [name: string]: unknown;
    }
  }
}

export {};
"#,
    )
    .unwrap();
}
