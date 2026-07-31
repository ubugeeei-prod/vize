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
