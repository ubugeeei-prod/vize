use super::{
    CompactString, PlainScriptExport, PlainScriptExportKind, collect_named_value_exports,
    emit_setup_invocation_and_exports,
};

fn export(name: &str, kind: PlainScriptExportKind) -> PlainScriptExport {
    PlainScriptExport {
        name: CompactString::new(name),
        kind,
        source_range: 0..name.len(),
        bridged_value: true,
    }
}

fn assert_exports(
    source: &str,
    exports: &[PlainScriptExport],
    expected: &[(&str, PlainScriptExportKind)],
) {
    assert_eq!(exports.len(), expected.len());
    for (export, &(name, kind)) in exports.iter().zip(expected) {
        assert_eq!(export.name, name);
        assert_eq!(export.kind, kind);
        assert!(export.bridged_value);
        assert_eq!(&source[export.source_range.clone()], name);
    }
}

#[test]
fn collect_named_value_exports_includes_ts_enums() {
    let source = "export enum DiffDisplayMode { Hidden = 'hidden' }\nexport type Props = {}\n";
    let exports = collect_named_value_exports(source);

    assert_exports(
        source,
        &exports,
        &[("DiffDisplayMode", PlainScriptExportKind::Enum)],
    );
}

#[test]
fn declaration_kinds_are_classified_by_declaration_space() {
    let source = "export const plain = 1;\nexport function helper() {}\nexport class Widget {}\nexport enum Mode { A = 'a' }\nexport const enum ConstMode { B = 'b' }\n";
    let exports = collect_named_value_exports(source);

    assert_exports(
        source,
        &exports,
        &[
            ("plain", PlainScriptExportKind::Value),
            ("helper", PlainScriptExportKind::Value),
            ("Widget", PlainScriptExportKind::Class),
            ("Mode", PlainScriptExportKind::Enum),
            ("ConstMode", PlainScriptExportKind::Enum),
        ],
    );
}

#[test]
fn value_and_type_declarations_are_bridged_in_both_declaration_spaces() {
    let mut ts = vize_s0::String::default();
    emit_setup_invocation_and_exports(
        &mut ts,
        &[
            export("Mode", PlainScriptExportKind::Enum),
            export("Widget", PlainScriptExportKind::Class),
        ],
    );

    assert!(ts.contains("export const Mode = __vize_plain_script_exports.Mode;\n"));
    assert!(ts.contains("export type Mode = (typeof Mode)[keyof typeof Mode];\n"));
    assert!(ts.contains("export const Widget = __vize_plain_script_exports.Widget;\n"));
    assert!(ts.contains("export type Widget = InstanceType<typeof Widget>;\n"));
}

#[test]
fn value_only_declarations_never_gain_a_type_export() {
    let mut ts = vize_s0::String::default();
    emit_setup_invocation_and_exports(
        &mut ts,
        &[
            export("plain", PlainScriptExportKind::Value),
            export("helper", PlainScriptExportKind::Value),
        ],
    );

    assert!(ts.contains("export const plain = __vize_plain_script_exports.plain;\n"));
    assert!(ts.contains("export const helper = __vize_plain_script_exports.helper;\n"));
    assert!(
        !ts.contains("export type "),
        "value-only exports must not invent a type meaning:\n{ts}"
    );
}

#[test]
fn merged_enum_declarations_are_bridged_once() {
    let source = "export enum Mode { A = 'a' }\nexport enum Mode { B = 'b' }\n";
    let exports = collect_named_value_exports(source);
    assert_exports(source, &exports, &[("Mode", PlainScriptExportKind::Enum)]);

    let mut ts = vize_s0::String::default();
    emit_setup_invocation_and_exports(&mut ts, &exports);
    assert_eq!(ts.matches("export const Mode =").count(), 1);
    assert_eq!(ts.matches("export type Mode =").count(), 1);
}

#[test]
fn no_exports_keeps_the_bare_setup_invocation() {
    let mut ts = vize_s0::String::default();
    emit_setup_invocation_and_exports(&mut ts, &[]);
    assert_eq!(ts.as_str(), "__setup();\n\n");
}
