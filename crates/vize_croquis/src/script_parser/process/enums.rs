//! TypeScript enum declaration processing.

use oxc_ast::ast::TSEnumDeclaration;
use vize_carton::CompactString;
use vize_relief::BindingType;

use super::super::ScriptParseResult;

pub(super) fn process_enum_declaration(
    result: &mut ScriptParseResult,
    enumeration: &TSEnumDeclaration<'_>,
) {
    if enumeration.r#const || enumeration.declare {
        return;
    }

    let name = enumeration.id.name.as_str();
    result.bindings.add(name, BindingType::SetupConst);
    result.binding_spans.insert(
        CompactString::new(name),
        (enumeration.id.span.start, enumeration.id.span.end),
    );
}
