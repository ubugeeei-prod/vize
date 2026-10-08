#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use super::parse_script_setup;

#[test]
fn type_props_retain_their_written_declaration_ranges() {
    let source = r#"
            const props = defineProps<{
                msg: string
                count?: number
            }>()
        "#;
    let result = parse_script_setup(source);

    assert_eq!(result.macros.all_calls().len(), 1);
    assert_eq!(result.macros.props().len(), 2);
    for (name, declaration) in [("msg", "msg: string"), ("count", "count?: number")] {
        let (start, end) = result
            .macros
            .prop_declaration(name)
            .expect("inline prop declaration range");
        assert_eq!(&source[start as usize..end as usize], declaration);
    }
}

#[test]
fn runtime_array_props_retain_their_literal_ranges() {
    let source = r#"
            const props = defineProps(['foo', 'bar'])
        "#;
    let result = parse_script_setup(source);

    assert_eq!(result.macros.props().len(), 2);
    for (name, declaration) in [("foo", "'foo'"), ("bar", "'bar'")] {
        let (start, end) = result
            .macros
            .prop_declaration(name)
            .expect("runtime prop declaration range");
        assert_eq!(&source[start as usize..end as usize], declaration);
    }
}

#[test]
fn shifting_macros_keeps_calls_and_prop_declarations_in_one_coordinate_space() {
    let source = "defineProps<{ msg: string }>()";
    let mut result = parse_script_setup(source);
    let call_before = result
        .macros
        .define_props()
        .expect("defineProps call")
        .start;
    let declaration_before = result
        .macros
        .prop_declaration("msg")
        .expect("prop declaration");

    result.macros.shift_offsets(17);

    assert_eq!(
        result.macros.define_props().expect("shifted call").start,
        call_before + 17
    );
    assert_eq!(
        result.macros.prop_declaration("msg"),
        Some((declaration_before.0 + 17, declaration_before.1 + 17))
    );
}

#[test]
fn local_alias_and_interface_props_retain_exact_owning_members() {
    for declaration in [
        "type Props = { hidden?: boolean; tone?: 'dark' | 'light' };",
        "interface Props { hidden?: boolean; tone?: 'dark' | 'light' }",
    ] {
        for invocation in [
            "defineProps<Props>();",
            "withDefaults(defineProps<Props>(), { tone: 'light' });",
            "const { tone = 'light' } = defineProps<Props>();",
        ] {
            let source = vize_carton::cstr!("{declaration}\n{invocation}");
            let result = parse_script_setup(&source);
            for (name, expected) in [
                ("hidden", "hidden?: boolean;"),
                ("tone", "tone?: 'dark' | 'light'"),
            ] {
                let (start, end) = result.macros.prop_declaration(name).unwrap();
                assert_eq!(source.get(start as usize..end as usize), Some(expected));
            }
        }
    }
}

#[test]
fn repeated_members_do_not_claim_one_arbitrary_authored_owner() {
    let source = "type Props = { shared: string } & { shared: number }; defineProps<Props>();";
    let result = parse_script_setup(source);
    assert!(result.macros.prop_declaration("shared").is_none());
}

#[test]
fn inline_defaults_keys_retain_exact_owner_ranges_and_shift_with_the_script() {
    let source = "type Props = { tone?: string }; const unrelated = { tone: 'dark' }; withDefaults(defineProps<Props>(), { tone: 'light' });";
    let mut result = parse_script_setup(source);
    let ranges = result.macros.with_defaults_key_ranges("tone");
    assert_eq!(ranges.len(), 1);
    let (start, end) = ranges[0];
    assert_eq!(source.get(start as usize..end as usize), Some("tone"));
    assert_eq!(start as usize, source.rfind("tone:").unwrap());
    result.macros.shift_offsets(17);
    assert_eq!(
        result.macros.with_defaults_key_ranges("tone"),
        &[(start + 17, end + 17)]
    );
}

#[test]
fn default_key_spelling_does_not_bless_distinct_or_ambiguous_owners() {
    for defaults in [
        "defaults",
        "{ tone }",
        "{ 'tone': 'light' }",
        "{ [tone]: 'light' }",
        "{ ...defaults, tone: 'light' }",
        "{ get tone() { return 'light' } }",
        "{ tone() { return 'light' } }",
    ] {
        let source = vize_carton::cstr!(
            "type Props = {{ tone?: string }}; const tone = 'dark'; const defaults = {{ tone: 'light' }}; withDefaults(defineProps<Props>(), {defaults});"
        );
        assert!(
            parse_script_setup(&source)
                .macros
                .with_defaults_key_ranges("tone")
                .is_empty()
        );
    }
    for source in [
        "type Props = { tone: string } & { tone: number }; withDefaults(defineProps<Props>(), { tone: 'light' });",
        "type Props = { tone?: string }; defineProps<Props>(); withDefaults(other(), { tone: 'light' });",
        "type A = { tone?: string }; type B = { tone?: string }; withDefaults(defineProps<A>(), { tone: 'a' }); defineProps<B>();",
        "import type { Imported } from './props'; type A = { tone?: string }; defineProps<A>(); withDefaults(defineProps<Imported>(), { tone: 'a' });",
        "type A = { tone?: string }; type B = { tone: string } & { tone: number }; defineProps<A>(); withDefaults(defineProps<B>(), { tone: 'a' });",
    ] {
        assert!(
            parse_script_setup(source)
                .macros
                .with_defaults_key_ranges("tone")
                .is_empty()
        );
    }
}
