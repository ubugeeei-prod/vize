use super::parse_script_setup;

#[test]
fn static_emits_retain_their_written_name_ranges() {
    for (source, name, declaration) in [
        (
            "defineEmits<{ (event: \"submit\", accepted: boolean): void }>()",
            "submit",
            "\"submit\"",
        ),
        ("defineEmits<{ save: [value: number] }>()", "save", "save"),
        (
            "defineEmits({ cancel: (reason: string) => true })",
            "cancel",
            "cancel",
        ),
    ] {
        let result = parse_script_setup(source);
        let (start, end) = result
            .macros
            .emit_declaration(name)
            .expect("static event declaration range");
        assert_eq!(&source[start as usize..end as usize], declaration);
    }
}

#[test]
fn shifting_macros_keeps_event_declarations_in_the_script_coordinate_space() {
    let source = "defineEmits(['save'])";
    let mut result = parse_script_setup(source);
    let declaration = result
        .macros
        .emit_declaration("save")
        .expect("event declaration");

    result.macros.shift_offsets(17);

    assert_eq!(
        result.macros.emit_declaration("save"),
        Some((declaration.0 + 17, declaration.1 + 17))
    );
}
