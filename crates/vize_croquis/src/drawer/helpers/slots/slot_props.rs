use oxc_allocator::Allocator;
use oxc_ast::ast::BindingPattern;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, SmallVec, String, profile};

/// Extract prop names from v-slot expression pattern
#[inline]
pub fn extract_slot_props(pattern: &str) -> SmallVec<[CompactString; 4]> {
    extract_slot_prop_bindings(pattern)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

#[inline]
pub fn extract_slot_prop_bindings(pattern: &str) -> SmallVec<[(CompactString, u32); 4]> {
    let leading_whitespace_len = pattern.len() - pattern.trim_start().len();
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return SmallVec::new();
    }

    let mut bindings = profile!(
        "croquis.helpers.slot_props.oxc",
        parse_slot_pattern_bindings(pattern)
    );
    for (_, offset) in &mut bindings {
        *offset += leading_whitespace_len as u32;
    }
    bindings
}

/// Parse slot pattern using OXC
fn parse_slot_pattern_bindings(pattern: &str) -> SmallVec<[(CompactString, u32); 4]> {
    const PREFIX: &str = "let ";
    const SUFFIX: &str = " = x";

    let mut pattern_str = String::with_capacity(PREFIX.len() + pattern.len() + SUFFIX.len());
    pattern_str.push_str(PREFIX);
    pattern_str.push_str(pattern);
    pattern_str.push_str(SUFFIX);

    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);
    let ret = profile!(
        "croquis.helpers.slot_props.oxc_parse",
        Parser::new(&allocator, pattern_str.as_str(), source_type).parse()
    );
    if !ret.diagnostics.is_empty() {
        return SmallVec::new();
    }

    let mut bindings = SmallVec::new();

    if let Some(oxc_ast::ast::Statement::VariableDeclaration(var_decl)) = ret.program.body.first()
        && let Some(declarator) = var_decl.declarations.first()
    {
        extract_slot_binding_names(&declarator.id, PREFIX.len() as u32, &mut bindings);
    }

    bindings
}

/// Extract binding names from slot pattern
fn extract_slot_binding_names(
    pattern: &BindingPattern<'_>,
    pattern_offset: u32,
    bindings: &mut SmallVec<[(CompactString, u32); 4]>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            let Some(relative) = id.span.start.checked_sub(pattern_offset) else {
                return;
            };
            bindings.push((CompactString::new(id.name.as_str()), relative));
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in obj.properties.iter() {
                extract_slot_binding_names(&prop.value, pattern_offset, bindings);
            }
            if let Some(rest) = &obj.rest {
                extract_slot_binding_names(&rest.argument, pattern_offset, bindings);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                extract_slot_binding_names(elem, pattern_offset, bindings);
            }
            if let Some(rest) = &arr.rest {
                extract_slot_binding_names(&rest.argument, pattern_offset, bindings);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            extract_slot_binding_names(&assign.left, pattern_offset, bindings);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_slot_prop_bindings, extract_slot_props};
    use vize_carton::{CompactString, SmallVec, cstr};

    fn names(pattern: &str) -> SmallVec<[CompactString; 4]> {
        extract_slot_props(pattern)
    }

    fn binding_offsets(pattern: &str) -> SmallVec<[(CompactString, u32); 4]> {
        extract_slot_prop_bindings(pattern)
    }

    #[test]
    fn extracts_simple_object_rest_with_default() {
        assert_eq!(
            names("{ open = false, ...rest }"),
            [cstr!("open"), cstr!("rest")].into()
        );
    }

    #[test]
    fn falls_back_for_nested_object_patterns() {
        assert_eq!(
            names("{ item: { id }, ...rest }"),
            [cstr!("id"), cstr!("rest")].into()
        );
    }

    #[test]
    fn falls_back_for_default_calls_with_commas() {
        assert_eq!(
            names("{ value = getDefault(a, b), rest }"),
            [cstr!("value"), cstr!("rest")].into()
        );
    }

    #[test]
    fn binding_offsets_point_to_local_object_pattern_bindings() {
        let pattern = "{ ラベル: value, name: name }";
        assert_eq!(
            binding_offsets(pattern),
            [
                (cstr!("value"), pattern.find("value").unwrap() as u32),
                (cstr!("name"), pattern.rfind("name").unwrap() as u32)
            ]
            .into()
        );
    }

    #[test]
    fn binding_offsets_include_trimmed_leading_whitespace() {
        let pattern = "  { ラベル: value, name: name }";
        assert_eq!(
            binding_offsets(pattern),
            [
                (cstr!("value"), pattern.find("value").unwrap() as u32),
                (cstr!("name"), pattern.rfind("name").unwrap() as u32)
            ]
            .into()
        );
    }
}
