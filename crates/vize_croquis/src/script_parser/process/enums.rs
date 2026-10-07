//! TypeScript enum declaration processing.

use oxc_ast::ast::TSEnumDeclaration;
use vize_carton::CompactString;
use vize_relief::BindingType;

use super::super::ScriptParseResult;

pub(super) fn process_enum_declaration(
    result: &mut ScriptParseResult,
    enumeration: &TSEnumDeclaration<'_>,
) {
    // Const enums still have runtime values in the isolated TypeScript
    // transform used for SFCs. Only ambient declarations have no value.
    if enumeration.declare {
        return;
    }
    // This existing processor records a declaration but never visits initializers.
    result.refuse_occurrences();

    let name = enumeration.id.name.as_str();
    result.bindings.add(name, BindingType::SetupConst);
    result.binding_spans.insert(
        CompactString::new(name),
        (enumeration.id.span.start, enumeration.id.span.end),
    );
}
