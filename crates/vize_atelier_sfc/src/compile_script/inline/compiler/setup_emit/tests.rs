use super::{has_blank_line_after_macro, has_semicolon_after_macro};

#[test]
fn detects_blank_line_after_macro_call() {
    let source = "const emit = defineEmits<{ change: [] }>();\n\nfunction call() {}";
    let macro_end = source.find(";\n").unwrap();

    assert!(has_blank_line_after_macro(source, macro_end));
}

#[test]
fn ignores_single_newline_after_macro_call() {
    let source = "const emit = defineEmits(['change'])\nfunction call() {}";
    let macro_end = source.find('\n').unwrap();

    assert!(!has_blank_line_after_macro(source, macro_end));
}

#[test]
fn ignores_non_whitespace_after_macro_call() {
    let source = "const emit = defineEmits(['change']); // comment\nfunction call() {}";
    let macro_end = source.find(';').unwrap();

    assert!(!has_blank_line_after_macro(source, macro_end));
}

#[test]
fn detects_semicolon_after_macro_call() {
    let source = "const props = defineProps<Props>();\n";
    let macro_end = source.find(';').unwrap();

    assert!(has_semicolon_after_macro(source, macro_end));
}
