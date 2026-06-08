use super::super::{Drawer, DrawerOptions};
use crate::croquis::{InvalidExportKind, TypeExportKind};

#[test]
fn test_drawer_script_bindings() {
    let mut drawer = Drawer::for_lint();
    drawer.draw_script(
        r#"
            const count = ref(0)
            const name = 'hello'
            let flag = true
            function handleClick() {}
        "#,
    );

    let summary = drawer.finish();
    assert!(summary.reactivity.is_reactive("count"));
    assert!(summary.reactivity.needs_value_access("count"));
    insta::assert_debug_snapshot!(summary);
}

#[test]
fn test_drawer_define_props() {
    let mut drawer = Drawer::for_lint();
    drawer.draw_script(
        r#"
            const props = defineProps<{
                msg: string
                count?: number
            }>()
        "#,
    );

    let summary = drawer.finish();
    assert_eq!(summary.macros.props().len(), 2);

    let prop_names: Vec<_> = summary
        .macros
        .props()
        .iter()
        .map(|p| p.name.as_str())
        .collect();
    assert!(prop_names.contains(&"msg"));
    assert!(prop_names.contains(&"count"));
}

#[test]
fn test_type_exports() {
    let mut drawer = Drawer::for_lint();
    drawer.draw_script(
        r#"
export type Props = {
    msg: string
}
export interface Emits {
    (e: 'update', value: string): void
}
const count = ref(0)
        "#,
    );

    let summary = drawer.finish();
    assert_eq!(summary.type_exports.len(), 2);

    let type_export = &summary.type_exports[0];
    assert_eq!(type_export.name.as_str(), "Props");
    assert_eq!(type_export.kind, TypeExportKind::Type);
    assert!(type_export.hoisted);

    let interface_export = &summary.type_exports[1];
    assert_eq!(interface_export.name.as_str(), "Emits");
    assert_eq!(interface_export.kind, TypeExportKind::Interface);
    assert!(interface_export.hoisted);
}

#[test]
fn test_invalid_exports() {
    let mut drawer = Drawer::for_lint();
    drawer.draw_script(
        r#"
export const foo = 'bar'
export let count = 0
export function hello() {}
export class MyClass {}
export default { foo: 'bar' }
const valid = ref(0)
        "#,
    );

    let summary = drawer.finish();
    assert_eq!(summary.invalid_exports.len(), 5);

    let kinds: Vec<_> = summary.invalid_exports.iter().map(|e| e.kind).collect();
    assert!(kinds.contains(&InvalidExportKind::Const));
    assert!(kinds.contains(&InvalidExportKind::Let));
    assert!(kinds.contains(&InvalidExportKind::Function));
    assert!(kinds.contains(&InvalidExportKind::Class));
    assert!(kinds.contains(&InvalidExportKind::Default));

    let names: Vec<_> = summary
        .invalid_exports
        .iter()
        .map(|e| e.name.as_str())
        .collect();
    assert!(names.contains(&"foo"));
    assert!(names.contains(&"count"));
    assert!(names.contains(&"hello"));
    assert!(names.contains(&"MyClass"));
}

#[test]
fn test_mixed_exports() {
    let mut drawer = Drawer::for_lint();
    drawer.draw_script(
        r#"
export type MyType = string
export const invalid = 123
export interface MyInterface { name: string }
        "#,
    );

    let summary = drawer.finish();
    assert_eq!(summary.type_exports.len(), 2);
    assert_eq!(summary.invalid_exports.len(), 1);
    assert_eq!(summary.invalid_exports[0].name.as_str(), "invalid");
}

#[test]
fn test_inject_detection_in_script_setup() {
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script_setup(
        r#"import { inject } from 'vue'

const theme = inject('theme')
const { name } = inject('user') as { name: string; id: number }"#,
    );

    let summary = drawer.finish();
    let injects = summary.provide_inject.injects();

    assert_eq!(injects.len(), 2, "Should detect 2 inject calls");

    assert_eq!(
        injects[0].key,
        crate::provide::ProvideKey::String(vize_carton::CompactString::new("theme"))
    );

    assert_eq!(
        injects[1].key,
        crate::provide::ProvideKey::String(vize_carton::CompactString::new("user"))
    );
    assert!(
        matches!(
            &injects[1].pattern,
            crate::provide::InjectPattern::ObjectDestructure(_)
        ),
        "Should detect object destructure pattern"
    );
}
