//! Slot props and inline callback parameter extraction.
//!
//! Handles parsing of:
//! - `v-slot` directive patterns (e.g., `v-slot="{ item, index }"`)
//! - Inline callback parameters from arrow functions and function expressions
//!   used in event handlers (e.g., `@click="(e) => handle(e)"`)

use oxc_allocator::Allocator;
use oxc_ast::ast::BindingPattern;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{profile, smallvec, CompactString, SmallVec};

use super::is_valid_identifier_fast;

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

    // SAFETY: we only copy ASCII bytes
    let pattern_str = unsafe { std::str::from_utf8_unchecked(&buffer[..total_len]) };
    profile!(
        "croquis.helpers.slot_props.parse_pattern",
        parse_slot_pattern(pattern_str)
    )
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

    if let Some(oxc_ast::ast::Statement::VariableDeclaration(var_decl)) = ret.program.body.first() {
        if let Some(declarator) = var_decl.declarations.first() {
            extract_slot_binding_names(&declarator.id, &mut props);
        }
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

/// Extract parameters from inline arrow function or function expression
#[inline]
pub fn extract_inline_callback_params(expr: &str) -> Option<SmallVec<[CompactString; 4]>> {
    let bytes = expr.as_bytes();
    let len = bytes.len();
    if len == 0 {
        return None;
    }

    // Skip leading whitespace
    let mut i = 0;
    while i < len && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= len {
        return None;
    }

    // Fast path: check for arrow "=>"
    let arrow_pos = find_arrow(bytes, i);

    if let Some(arrow_idx) = arrow_pos {
        let mut end = arrow_idx;
        while end > i && bytes[end - 1].is_ascii_whitespace() {
            end -= 1;
        }
        if end <= i {
            return None;
        }

        let before_bytes = &bytes[i..end];

        // Check for async prefix
        let (param_start, param_end) = if before_bytes.starts_with(b"async")
            && before_bytes.len() > 5
            && before_bytes[5].is_ascii_whitespace()
        {
            let mut s = 5;
            while s < before_bytes.len() && before_bytes[s].is_ascii_whitespace() {
                s += 1;
            }
            (i + s, end)
        } else {
            (i, end)
        };

        let param_bytes = &bytes[param_start..param_end];

        // (params) => pattern
        if param_bytes.first() == Some(&b'(') && param_bytes.last() == Some(&b')') {
            let inner = &expr[param_start + 1..param_end - 1];
            let inner_trimmed = inner.trim();
            if inner_trimmed.is_empty() {
                return Some(SmallVec::new());
            }
            return Some(extract_param_list_fast(inner_trimmed));
        }

        // Single param: e =>
        let param = &expr[param_start..param_end];
        if is_valid_identifier_fast(param.as_bytes()) {
            let mut result = SmallVec::new();
            result.push(CompactString::new(param));
            return Some(result);
        }
    }

    // Check for function expression
    if bytes[i..].starts_with(b"function") {
        let fn_end = i + 8;
        let mut paren_start = fn_end;
        while paren_start < len && bytes[paren_start] != b'(' {
            paren_start += 1;
        }
        if paren_start >= len {
            return None;
        }
        let mut paren_end = paren_start + 1;
        let mut depth = 1;
        while paren_end < len && depth > 0 {
            match bytes[paren_end] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            paren_end += 1;
        }
        if depth == 0 {
            let inner = &expr[paren_start + 1..paren_end - 1];
            let inner_trimmed = inner.trim();
            if inner_trimmed.is_empty() {
                return Some(SmallVec::new());
            }
            return Some(extract_param_list_fast(inner_trimmed));
        }
    }

    None
}

/// Find arrow "=>" position in bytes
#[inline]
fn find_arrow(bytes: &[u8], start: usize) -> Option<usize> {
    let len = bytes.len();
    if len < start + 2 {
        return None;
    }
    let mut i = start;
    while i < len - 1 {
        if bytes[i] == b'=' && bytes[i + 1] == b'>' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Extract parameter list from comma-separated string
#[inline]
fn extract_param_list_fast(params: &str) -> SmallVec<[CompactString; 4]> {
    let bytes = params.as_bytes();
    let len = bytes.len();
    let mut result = SmallVec::new();
    let mut i = 0;

    while i < len {
        // Skip whitespace
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= len {
            break;
        }

        // Skip rest parameter prefix (...)
        if i + 2 < len && bytes[i] == b'.' && bytes[i + 1] == b'.' && bytes[i + 2] == b'.' {
            i += 3;
            while i < len && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
        }

        // Skip destructuring patterns
        if i < len && (bytes[i] == b'{' || bytes[i] == b'[') {
            let open = bytes[i];
            let close = if open == b'{' { b'}' } else { b']' };
            let mut depth = 1;
            i += 1;
            while i < len && depth > 0 {
                if bytes[i] == open {
                    depth += 1;
                } else if bytes[i] == close {
                    depth -= 1;
                }
                i += 1;
            }
            while i < len && bytes[i] != b',' {
                i += 1;
            }
            if i < len {
                i += 1;
            }
            continue;
        }

        // Extract identifier
        let ident_start = i;
        while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
        {
            i += 1;
        }

        if i > ident_start {
            result.push(CompactString::new(&params[ident_start..i]));
        }

        // Skip to next comma
        while i < len && bytes[i] != b',' {
            i += 1;
        }
        if i < len {
            i += 1;
        }
    }

    result
}
