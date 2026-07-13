use super::ScriptCompileContext;
use crate::types::BindingType;
use vize_carton::ToCompactString;

#[test]
fn test_context_analyze() {
    let content = r#"
const msg = ref('hello')
const count = ref(0)
let name = 'world'
const double = computed(() => count.value * 2)
function increment() { count.value++ }
"#;
    let mut ctx = ScriptCompileContext::new(content);
    ctx.analyze();

    assert_eq!(
        ctx.bindings.bindings.get("msg"),
        Some(&BindingType::SetupRef)
    );
    assert_eq!(
        ctx.bindings.bindings.get("count"),
        Some(&BindingType::SetupRef)
    );
    assert_eq!(
        ctx.bindings.bindings.get("name"),
        Some(&BindingType::SetupLet)
    );
    assert_eq!(
        ctx.bindings.bindings.get("increment"),
        Some(&BindingType::SetupConst)
    );
}

#[test]
fn test_extract_define_props_typed() {
    let content = r#"const props = defineProps<{ msg: string }>()"#;
    let mut ctx = ScriptCompileContext::new(content);
    ctx.analyze();

    assert!(ctx.has_define_props_call);
    assert!(ctx.macros.define_props.is_some());
    let props_call = ctx.macros.define_props.unwrap();
    assert_eq!(
        props_call.type_args,
        Some("{ msg: string }".to_compact_string())
    );
}

#[test]
fn test_extract_define_emits_typed() {
    let content = r#"const emit = defineEmits<{ (e: 'click'): void }>()"#;
    let mut ctx = ScriptCompileContext::new(content);
    ctx.analyze();

    assert!(ctx.has_define_emits_call);
    assert!(ctx.macros.define_emits.is_some());
}

#[test]
fn test_extract_with_defaults() {
    let content =
        r#"const props = withDefaults(defineProps<{ msg?: string }>(), { msg: 'hello' })"#;
    let mut ctx = ScriptCompileContext::new(content);
    ctx.analyze();

    assert!(ctx.has_define_props_call);
    assert!(ctx.macros.with_defaults.is_some());
}

#[test]
fn test_props_destructure() {
    let content = r#"const { foo, bar } = defineProps<{ foo: string, bar: number }>()"#;
    let mut ctx = ScriptCompileContext::new(content);
    ctx.analyze();

    assert!(ctx.macros.props_destructure.is_some());
    let destructure = ctx.macros.props_destructure.as_ref().unwrap();
    assert_eq!(destructure.bindings.len(), 2);
    assert!(destructure.bindings.contains_key("foo"));
    assert!(destructure.bindings.contains_key("bar"));
}

#[test]
fn test_props_destructure_with_alias() {
    let content =
        r#"const { foo: myFoo, bar = 123 } = defineProps<{ foo: string, bar?: number }>()"#;
    let mut ctx = ScriptCompileContext::new(content);
    ctx.analyze();

    assert!(ctx.macros.props_destructure.is_some());
    let destructure = ctx.macros.props_destructure.as_ref().unwrap();

    // Check that bindings use the key as the map key
    assert!(destructure.bindings.contains_key("foo"));
    assert!(destructure.bindings.contains_key("bar"));

    // Check local names
    assert_eq!(destructure.bindings.get("foo").unwrap().local, "myFoo");
    assert_eq!(destructure.bindings.get("bar").unwrap().local, "bar");

    // Check default value
    assert!(destructure.bindings.get("bar").unwrap().default.is_some());
}

#[test]
fn test_define_props_with_interface_reference() {
    let content = r#"
interface Props {
    msg: string
    count?: number
}
const props = defineProps<Props>()
"#;
    let mut ctx = ScriptCompileContext::new(content);
    ctx.analyze();

    // Check interface was captured
    assert!(ctx.interfaces.contains_key("Props"));

    // Check props were extracted from interface
    assert!(ctx.has_define_props_call);
    assert_eq!(ctx.bindings.bindings.get("msg"), Some(&BindingType::Props));
    assert_eq!(
        ctx.bindings.bindings.get("count"),
        Some(&BindingType::Props)
    );
}

#[test]
fn test_define_props_with_readonly_interface_prop() {
    let content = r#"
interface Props {
    readonly minScale?: number
}
const props = defineProps<Props>()
"#;
    let mut ctx = ScriptCompileContext::new(content);
    ctx.analyze();

    assert!(ctx.has_define_props_call);
    assert_eq!(
        ctx.bindings.bindings.get("minScale"),
        Some(&BindingType::Props)
    );
}

#[test]
fn test_define_props_with_type_alias_reference() {
    let content = r#"
type Props = {
    foo: string
    bar: number
}
const props = defineProps<Props>()
"#;
    let mut ctx = ScriptCompileContext::new(content);
    ctx.analyze();

    // Check type alias was captured
    assert!(ctx.type_aliases.contains_key("Props"));

    // Check props were extracted from type alias
    assert!(ctx.has_define_props_call);
    assert_eq!(ctx.bindings.bindings.get("foo"), Some(&BindingType::Props));
    assert_eq!(ctx.bindings.bindings.get("bar"), Some(&BindingType::Props));
}

#[test]
fn test_define_props_with_exported_type_alias() {
    let content = r#"
export type MenuItemProps = {
    id: string
    label: string
    routeName: string
    disabled?: boolean
}
const { label, disabled, routeName } = defineProps<MenuItemProps>()
"#;
    let mut ctx = ScriptCompileContext::new(content);
    ctx.analyze();

    // Check exported type alias was captured
    assert!(
        ctx.type_aliases.contains_key("MenuItemProps"),
        "export type alias should be collected"
    );

    // Check props were extracted from exported type alias
    assert!(ctx.has_define_props_call);
    assert_eq!(
        ctx.bindings.bindings.get("label"),
        Some(&BindingType::Props)
    );
    assert_eq!(
        ctx.bindings.bindings.get("disabled"),
        Some(&BindingType::Props)
    );
    assert_eq!(
        ctx.bindings.bindings.get("routeName"),
        Some(&BindingType::Props)
    );
}

#[test]
fn test_define_props_with_exported_interface() {
    let content = r#"
export interface Props {
    msg: string
    count?: number
}
const props = defineProps<Props>()
"#;
    let mut ctx = ScriptCompileContext::new(content);
    ctx.analyze();

    // Check exported interface was captured
    assert!(
        ctx.interfaces.contains_key("Props"),
        "export interface should be collected"
    );

    // Check props were extracted from exported interface
    assert!(ctx.has_define_props_call);
    assert_eq!(ctx.bindings.bindings.get("msg"), Some(&BindingType::Props));
    assert_eq!(
        ctx.bindings.bindings.get("count"),
        Some(&BindingType::Props)
    );
}

#[test]
fn test_with_defaults_with_interface() {
    let content = r#"
interface Props {
    msg?: string
    count?: number
}
const props = withDefaults(defineProps<Props>(), {
    msg: 'hello',
    count: 0
})
"#;
    let mut ctx = ScriptCompileContext::new(content);
    ctx.analyze();

    assert!(ctx.has_define_props_call);
    assert!(ctx.macros.with_defaults.is_some());
    assert_eq!(ctx.bindings.bindings.get("msg"), Some(&BindingType::Props));
    assert_eq!(
        ctx.bindings.bindings.get("count"),
        Some(&BindingType::Props)
    );
}
