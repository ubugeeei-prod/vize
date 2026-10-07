use crate::script_parser::parse_script_setup;

#[test]
fn retained_static_slot_keys_use_their_whole_written_ast_ranges() {
    let source = "defineSlots<{ header(props: { title: string }): unknown; \"item-row\": (props: { title: number }) => unknown; [dynamic](props: {}): unknown }>();";
    let mut result = parse_script_setup(source);
    assert_eq!(result.macros.slot_type_argument_range(), Some((12, 139)));
    assert_eq!(
        result.macros.static_slot_declarations().collect::<Vec<_>>(),
        [(14, 20), (57, 67)]
    );
    assert_eq!(source.get(57..67), Some("\"item-row\""));
    result.macros.shift_offsets(23);
    assert_eq!(result.macros.slot_type_argument_range(), Some((35, 162)));
    assert_eq!(
        result.macros.static_slot_declarations().collect::<Vec<_>>(),
        [(37, 43), (80, 90)]
    );
}

#[test]
fn repeated_intersection_keys_retain_each_written_declaration() {
    let result = parse_script_setup("defineSlots<{ header(): void } & { header(): void }>();");
    assert_eq!(
        result.macros.static_slot_declarations().collect::<Vec<_>>(),
        [(14, 20), (35, 41)]
    );
}
