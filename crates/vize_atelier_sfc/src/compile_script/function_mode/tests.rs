use super::compile_script_setup;

#[test]
fn preserves_define_model_setup_runtime_accessors() {
    let content = r#"
const minimum = computed(() => 1)
const [model, modifiers] = defineModel<number>({
  get(value) { return value ?? minimum.value },
  set(value) { return modifiers.number ? Math.max(minimum.value, Number(value)) : value },
})
"#;
    let result = compile_script_setup(content, "Test", false, true, None).unwrap();

    assert!(
        result.code.contains("_useModel(__props, \"modelValue\", {")
            && result.code.contains("return value ?? minimum.value"),
        "{}",
        result.code
    );
    assert!(result.code.contains("modifiers.number"), "{}", result.code);
}

#[test]
fn emits_static_enums_before_the_component() {
    let content = r#"
enum Step { Name, General = 1 + 1 }
const current = Step.Name
"#;
    let result = compile_script_setup(content, "ProfileForm", false, true, None).unwrap();

    let step = result
        .code
        .find("enum Step")
        .expect("enum should be emitted");
    let component = result
        .code
        .find("const __sfc__")
        .expect("component wrapper");
    assert!(step < component, "{}", result.code);
    assert!(
        result.code.contains("const current = Step.Name"),
        "{}",
        result.code
    );
}

#[test]
fn runtime_references_skip_typescript_only_syntax() {
    use super::helpers::runtime_identifier_references;
    let refs = runtime_identifier_references(
        r#"
const a: TypeA = valueA as TypeB
const b = useB<TypeC>(valueC!) satisfies TypeD
type Alias = TypeE
interface Shape extends BaseShape { field: TypeF }
declare const ambient: TypeG
declare function ambientFn(arg: TypeH): void
function run(this: ThisType, arg: TypeI): TypeJ { return callI(arg) }
class Box implements Contract { declare tag: TypeK; value = initL }
enum Kind { A = external.value }
"#,
    );
    // Exact oracle: every TypeA..TypeK, BaseShape, ThisType and Contract name
    // appears only in type positions and must be absent.
    let expected: vize_carton::FxHashSet<vize_carton::String> = [
        "valueA", "useB", "valueC", "callI", "arg", "initL", "external",
    ]
    .into_iter()
    .map(Into::into)
    .collect();
    assert_eq!(refs, expected);
}

#[test]
fn runtime_references_match_the_stripped_javascript() {
    use super::helpers::runtime_identifier_references;
    use crate::compile_script::typescript::transform_typescript_to_js;
    use oxc_ast_visit::Visit;

    struct Names(vize_carton::FxHashSet<vize_carton::String>);
    impl<'a> Visit<'a> for Names {
        fn visit_identifier_reference(&mut self, it: &oxc_ast::ast::IdentifierReference<'a>) {
            self.0.insert(it.name.as_str().into());
        }
    }
    for code in [
        "const count = ref<number>(props.start as number)\nwatch(count, (v: number) => emit('change', v!))",
        "const items = computed((): Item[] => list.value.filter((i): i is Item => !!i))",
        "async function load(id: Id): Promise<void> { data.value = await fetchData<Data>(id) }",
        "const handler = <T,>(x: T) => format(x)\nlet el: HTMLElement | null = null",
        "const store = useStore()\nconst { a, b } = storeToRefs(store) as Refs",
        "const plain = 1\nconsole.log(plain, window.location)",
    ] {
        let allocator = oxc_allocator::Allocator::default();
        let js = transform_typescript_to_js(code);
        let parsed = oxc_parser::Parser::new(&allocator, &js, oxc_span::SourceType::mjs()).parse();
        assert!(parsed.diagnostics.is_empty(), "{js}");
        let mut names = Names(Default::default());
        names.visit_program(&parsed.program);
        assert_eq!(runtime_identifier_references(code), names.0, "{code}");
    }
}
