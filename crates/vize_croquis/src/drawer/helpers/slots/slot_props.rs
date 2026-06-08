use oxc_allocator::Allocator;
use oxc_ast::ast::BindingPattern;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, SmallVec, profile, smallvec};

use crate::drawer::helpers::is_valid_identifier_fast;

/// Extract prop names from v-slot expression pattern
#[inline]
pub fn extract_slot_props(pattern: &str) -> SmallVec<[CompactString; 4]> {
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return SmallVec::new();
    }

    let bytes = pattern.as_bytes();

    // Fast path: simple identifier
    if bytes[0] != b'{' && bytes[0] != b'[' {
        if is_valid_identifier_fast(bytes) {
            return smallvec![CompactString::new(pattern)];
        }
        return SmallVec::new();
    }

    // Fast path: simple object destructuring
    if bytes[0] == b'{' && !pattern.contains(':') && !pattern.contains('{') {
        let inner = &pattern[1..pattern.len().saturating_sub(1)];
        let mut props = SmallVec::new();
        for part in inner.split(',') {
            let part = part.trim();
            let name = if let Some(eq_pos) = part.find('=') {
                part[..eq_pos].trim()
            } else {
                part
            };
            if !name.is_empty() && is_valid_identifier_fast(name.as_bytes()) {
                props.push(CompactString::new(name));
            }
        }
        if !props.is_empty() {
            return props;
        }
    }

    // Complex case: use OXC parser
    profile!(
        "croquis.helpers.slot_props.oxc",
        extract_slot_props_with_oxc(pattern)
    )
}

/// Parse complex slot props using OXC
#[cold]
fn extract_slot_props_with_oxc(pattern: &str) -> SmallVec<[CompactString; 4]> {
    let mut buffer = [0u8; 256];
    let prefix = b"let ";
    let suffix = b" = x";

    let total_len = prefix.len() + pattern.len() + suffix.len();
    if total_len > buffer.len() {
        #[allow(clippy::disallowed_macros)]
        let pattern_str = format!("let {pattern} = x");
        return profile!(
            "croquis.helpers.slot_props.parse_pattern",
            parse_slot_pattern(&pattern_str)
        );
    }

    buffer[..prefix.len()].copy_from_slice(prefix);
    buffer[prefix.len()..prefix.len() + pattern.len()].copy_from_slice(pattern.as_bytes());
    buffer[prefix.len() + pattern.len()..total_len].copy_from_slice(suffix);

    match std::str::from_utf8(&buffer[..total_len]) {
        Ok(pattern_str) => profile!(
            "croquis.helpers.slot_props.parse_pattern",
            parse_slot_pattern(pattern_str)
        ),
        Err(_) => SmallVec::new(),
    }
}

/// Parse slot pattern using OXC
fn parse_slot_pattern(pattern_str: &str) -> SmallVec<[CompactString; 4]> {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);
    let ret = profile!(
        "croquis.helpers.slot_props.oxc_parse",
        Parser::new(&allocator, pattern_str, source_type).parse()
    );

    let mut props = SmallVec::new();

    if let Some(oxc_ast::ast::Statement::VariableDeclaration(var_decl)) = ret.program.body.first()
        && let Some(declarator) = var_decl.declarations.first()
    {
        extract_slot_binding_names(&declarator.id, &mut props);
    }

    props
}

/// Extract binding names from slot pattern
fn extract_slot_binding_names(
    pattern: &BindingPattern<'_>,
    names: &mut SmallVec<[CompactString; 4]>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            names.push(CompactString::new(id.name.as_str()));
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in obj.properties.iter() {
                extract_slot_binding_names(&prop.value, names);
            }
            if let Some(rest) = &obj.rest {
                extract_slot_binding_names(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                extract_slot_binding_names(elem, names);
            }
            if let Some(rest) = &arr.rest {
                extract_slot_binding_names(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            extract_slot_binding_names(&assign.left, names);
        }
    }
}
