//! Runtime enum names include const enums, while ambient enums remain type-only.

#![expect(clippy::string_slice, reason = "tests assert identifier source spans")]

use vize_croquis::script_parser::parse_script;
use vize_relief::BindingType;

#[test]
fn enum_names_and_identifier_spans_survive_const_and_export_modifiers() {
    let source = r#"enum Local { A }
const enum LocalConst { A }
export enum Shared { A }
export const enum SharedConst { A }
declare enum Ambient { A }
declare const enum AmbientConst { A }
"#;
    let result = parse_script(source);
    for name in ["Local", "LocalConst", "Shared", "SharedConst"] {
        assert_eq!(
            result.bindings.get(name),
            Some(BindingType::SetupConst),
            "{name}"
        );
        let (start, end) = result
            .binding_spans
            .get(name)
            .expect("enum identifier span");
        assert_eq!(&source[*start as usize..*end as usize], name);
    }
    for name in ["Ambient", "AmbientConst"] {
        assert_eq!(result.bindings.get(name), None, "{name} is type-only");
        assert!(!result.binding_spans.contains_key(name));
    }
}
