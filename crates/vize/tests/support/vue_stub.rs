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
        r#"export interface ComponentCustomProperties {}

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
