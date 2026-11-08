use super::collect_options_descriptor;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, cstr};

/// Parse `source` and return `(member_name, group_label)` pairs in order.
/// Labels are compared instead of the enum to keep names borrowed (the
/// workspace disallows `ToString::to_string`).
fn members(allocator: &Allocator, source: &str) -> Vec<(CompactString, &'static str)> {
    let source_type = SourceType::from_path("script.ts").unwrap();
    let ret = Parser::new(allocator, source, source_type).parse();
    let descriptor = collect_options_descriptor(&ret.program).expect("descriptor");
    descriptor
        .members
        .iter()
        .map(|member| (member.name.clone(), member.group.label()))
        .collect()
}

#[test]
fn collects_members_across_groups_in_source_order() {
    let source = r#"
export default {
  props: ['foo'],
  data() { return { bar: 1 } },
  computed: { baz() { return 2 } },
  methods: { qux() {} },
  inject: { wiz: { from: 'w' } },
  setup() { return { zap: 0 } },
}
"#;
    let allocator = Allocator::default();
    assert_eq!(
        members(&allocator, source),
        vec![
            (cstr!("foo"), "props"),
            (cstr!("bar"), "data"),
            (cstr!("baz"), "computed"),
            (cstr!("qux"), "methods"),
            (cstr!("wiz"), "inject"),
            (cstr!("zap"), "setup"),
        ]
    );
}

#[test]
fn records_array_prop_span_including_quotes_and_all_option_keys() {
    let allocator = Allocator::default();
    let source = "export default { props: ['foo'], name: 'C' }";
    let source_type = SourceType::from_path("script.ts").unwrap();
    let ret = Parser::new(&allocator, source, source_type).parse();
    let descriptor = collect_options_descriptor(&ret.program).expect("descriptor");

    // Array-form prop member span covers the full string literal incl. quotes.
    let prop = &descriptor.members[0];
    assert_eq!(&source[prop.start as usize..prop.end as usize], "'foo'");
    // Every top-level option key is recorded (props + name), verbatim.
    let keys: Vec<&str> = descriptor
        .option_keys
        .iter()
        .map(|k| k.name.as_str())
        .collect();
    assert_eq!(keys, vec!["props", "name"]);
}

#[test]
fn resolves_define_component_and_identifier_export() {
    let allocator = Allocator::default();
    let from_define = members(
        &allocator,
        "export default defineComponent({ methods: { a() {} } })",
    );
    assert_eq!(from_define, vec![(cstr!("a"), "methods")]);

    let from_ident = members(
        &allocator,
        "const c = { computed: { b() { return 1 } } }\nexport default c",
    );
    assert_eq!(from_ident, vec![(cstr!("b"), "computed")]);
}

#[test]
fn no_options_for_script_setup_style() {
    let allocator = Allocator::default();
    let source = "import { ref } from 'vue'\nconst x = ref(0)";
    let source_type = SourceType::from_path("script.ts").unwrap();
    let ret = Parser::new(&allocator, source, source_type).parse();
    assert!(collect_options_descriptor(&ret.program).is_none());
}
