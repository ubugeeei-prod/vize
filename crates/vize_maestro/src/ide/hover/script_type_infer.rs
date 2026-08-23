//! Lightweight fallback inference for script-authored hover text.

use super::HoverService;
use vize_relief::BindingType;

impl HoverService {
    /// Infer a more specific type from the script content.
    pub(super) fn infer_type_from_script(
        content: &str,
        name: &str,
        binding_type: BindingType,
    ) -> Option<String> {
        if let Some(initializer) = Self::find_declaration_initializer(content, name)
            && let Some(inferred) = Self::infer_wrapped_initializer_type(initializer)
        {
            return Some(inferred);
        }
        if let Some(type_str) = Self::find_declaration_type_annotation(content, name) {
            return Some(type_str);
        }
        if binding_type == BindingType::Props {
            return Self::infer_prop_type(content, name);
        }
        None
    }

    fn find_declaration_initializer<'a>(content: &'a str, name: &str) -> Option<&'a str> {
        let after_name = Self::find_declaration_after_name(content, name)?.trim_start();
        let after_type = if let Some(type_annotation) = after_name.strip_prefix(':') {
            let eq_pos = Self::find_top_level_char(type_annotation, '=')?;
            &type_annotation[eq_pos + 1..]
        } else {
            after_name
        };
        after_type
            .trim_start()
            .strip_prefix('=')
            .map(str::trim_start)
    }

    fn find_declaration_type_annotation(content: &str, name: &str) -> Option<String> {
        let after_name = Self::find_declaration_after_name(content, name)?.trim_start();
        let type_annotation = after_name.strip_prefix(':')?;
        Self::extract_type_annotation(type_annotation)
    }

    fn find_declaration_after_name<'a>(content: &'a str, name: &str) -> Option<&'a str> {
        for keyword in ["const", "let", "var"] {
            let mut search = 0;
            while let Some(relative_pos) = content[search..].find(keyword) {
                let keyword_start = search + relative_pos;
                let keyword_end = keyword_start + keyword.len();
                search = keyword_end;
                if keyword_start > 0 && Self::is_ident_byte(content.as_bytes()[keyword_start - 1]) {
                    continue;
                }
                if !content[keyword_end..]
                    .chars()
                    .next()
                    .is_some_and(char::is_whitespace)
                {
                    continue;
                }
                let after_keyword = content[keyword_end..].trim_start();
                let Some(after_name) = after_keyword.strip_prefix(name) else {
                    continue;
                };
                if after_name
                    .as_bytes()
                    .first()
                    .is_some_and(|byte| Self::is_ident_byte(*byte))
                {
                    continue;
                }
                return Some(after_name);
            }
        }
        None
    }

    fn is_ident_byte(byte: u8) -> bool {
        byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
    }

    fn infer_wrapped_initializer_type(initializer: &str) -> Option<String> {
        let initializer = initializer.trim_start();
        for (callee, wrapper) in [
            ("shallowRef", "Ref"),
            ("ref", "Ref"),
            ("computed", "ComputedRef"),
            ("reactive", "Reactive"),
        ] {
            let Some(after_callee) = initializer.strip_prefix(callee) else {
                continue;
            };
            let after_callee = after_callee.trim_start();
            if let Some(after_type_start) = after_callee.strip_prefix('<') {
                if let Some(end) = Self::find_matching_bracket(after_type_start, '<', '>') {
                    return Some(Self::format_wrapper_type(
                        wrapper,
                        after_type_start[..end].trim(),
                    ));
                }
            } else if let Some(after_arg_start) = after_callee.strip_prefix('(')
                && let Some(arg_type) = Self::infer_type_from_arg(after_arg_start)
            {
                return Some(Self::format_wrapper_type(wrapper, &arg_type));
            }
        }
        None
    }

    #[allow(clippy::disallowed_macros)]
    fn format_wrapper_type(wrapper: &str, inner_type: &str) -> String {
        format!("{wrapper}<{inner_type}>")
    }

    fn infer_type_from_arg(arg_str: &str) -> Option<String> {
        let arg_str = arg_str.trim();
        if arg_str.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
            return Some("number".to_string());
        }
        if arg_str.starts_with('"') || arg_str.starts_with('\'') || arg_str.starts_with('`') {
            return Some("string".to_string());
        }
        if arg_str.starts_with("true") || arg_str.starts_with("false") {
            return Some("boolean".to_string());
        }
        if arg_str.starts_with('[') {
            return Some("unknown[]".to_string());
        }
        if arg_str.starts_with('{') {
            return Some("object".to_string());
        }
        if arg_str.starts_with("null") {
            return Some("null".to_string());
        }
        if arg_str.starts_with("undefined") {
            return Some("undefined".to_string());
        }
        if let Some(body) = Self::extract_arrow_function_body(arg_str)
            && let Some(return_type) = Self::infer_type_from_expression(body)
        {
            return Some(return_type);
        }
        None
    }

    fn extract_arrow_function_body(arg_str: &str) -> Option<&str> {
        let arrow = arg_str.find("=>")?;
        let body = arg_str[arrow + 2..].trim_start();
        if let Some(body) = body.strip_prefix('{')
            && let Some(return_pos) = body.find("return")
        {
            let returned = body[return_pos + "return".len()..].trim_start();
            let end = returned.find([';', '}']).unwrap_or(returned.len());
            return Some(returned[..end].trim());
        }
        let end = body.find(['\n', ';']).unwrap_or(body.len());
        Some(body[..end].trim().trim_end_matches(')').trim())
    }

    fn infer_type_from_expression(expression: &str) -> Option<String> {
        let expression = expression.trim();
        if expression.starts_with('"')
            || expression.starts_with('\'')
            || expression.starts_with('`')
        {
            return Some("string".to_string());
        }
        if expression.starts_with("true") || expression.starts_with("false") {
            return Some("boolean".to_string());
        }
        if expression.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
            return Some("number".to_string());
        }
        if expression.contains(".toUpperCase(")
            || expression.contains(".toLowerCase(")
            || expression.contains(".trim(")
        {
            return Some("string".to_string());
        }
        if expression.contains("===")
            || expression.contains("!==")
            || expression.contains(">=")
            || expression.contains("<=")
            || expression.contains(" > ")
            || expression.contains(" < ")
        {
            return Some("boolean".to_string());
        }
        if expression.contains('*') || expression.contains('/') || expression.contains(" - ") {
            return Some("number".to_string());
        }
        None
    }

    fn extract_type_annotation(s: &str) -> Option<String> {
        let s = s.trim();
        let end = Self::find_type_annotation_end(s);
        let type_str = s[..end].trim();
        (!type_str.is_empty()).then(|| type_str.to_string())
    }

    fn find_type_annotation_end(s: &str) -> usize {
        Self::find_top_level_char(s, '=')
            .or_else(|| Self::find_top_level_char(s, ';'))
            .or_else(|| Self::find_top_level_char(s, '\n'))
            .unwrap_or(s.len())
    }

    fn find_top_level_char(s: &str, target: char) -> Option<usize> {
        let mut depth = 0;
        for (index, c) in s.char_indices() {
            match c {
                '<' | '(' | '[' | '{' => depth += 1,
                '>' | ')' | ']' | '}' => depth -= 1,
                c if c == target && depth == 0 => {
                    if target == '=' && s[index..].starts_with("=>") {
                        continue;
                    }
                    return Some(index);
                }
                _ => {}
            }
        }
        None
    }

    fn find_matching_bracket(s: &str, open: char, close: char) -> Option<usize> {
        let mut depth = 1;
        for (i, c) in s.chars().enumerate() {
            if c == open {
                depth += 1;
            } else if c == close {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
        }
        None
    }

    #[allow(clippy::disallowed_macros)]
    fn infer_prop_type(content: &str, prop_name: &str) -> Option<String> {
        if let Some(props_start) = content.find("defineProps<") {
            let after = &content[props_start + "defineProps<".len()..];
            if let Some(end) = Self::find_matching_bracket(after, '<', '>') {
                let props_type = &after[..end];
                let prop_pattern = format!("{prop_name}: ");
                if let Some(prop_pos) = props_type.find(prop_pattern.as_str()) {
                    let after_prop = &props_type[prop_pos + prop_pattern.len()..];
                    if let Some(type_str) = Self::extract_prop_type(after_prop) {
                        return Some(type_str);
                    }
                }
                let opt_pattern = format!("{prop_name}?: ");
                if let Some(prop_pos) = props_type.find(opt_pattern.as_str()) {
                    let after_prop = &props_type[prop_pos + opt_pattern.len()..];
                    if let Some(type_str) = Self::extract_prop_type(after_prop) {
                        return Some(format!("{type_str} | undefined"));
                    }
                }
            }
        }
        None
    }

    fn extract_prop_type(s: &str) -> Option<String> {
        let s = s.trim();
        let mut depth = 0;
        let mut end = 0;
        for (i, c) in s.chars().enumerate() {
            match c {
                '<' | '(' | '[' | '{' => depth += 1,
                '>' | ')' | ']' | '}' => {
                    if depth == 0 {
                        end = i;
                        break;
                    }
                    depth -= 1;
                }
                ',' | ';' | '\n' if depth == 0 => {
                    end = i;
                    break;
                }
                _ => {}
            }
            end = i + 1;
        }
        let type_str = s[..end].trim();
        (!type_str.is_empty()).then(|| type_str.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::HoverService;
    use vize_relief::BindingType;

    #[test]
    fn fallback_inference_accepts_realistic_declaration_spacing() {
        let script = r#"
const count=ref(1)
let label : string = 'hello'
const flag = shallowRef<boolean>(false)
const doubled = computed(()=> count.value * 2)
const state = reactive<{ id: string }>({ id: 'a' })
"#;

        assert_eq!(
            HoverService::infer_type_from_script(script, "count", BindingType::SetupRef),
            Some("Ref<number>".to_string())
        );
        assert_eq!(
            HoverService::infer_type_from_script(script, "label", BindingType::SetupLet),
            Some("string".to_string())
        );
        assert_eq!(
            HoverService::infer_type_from_script(script, "flag", BindingType::SetupRef),
            Some("Ref<boolean>".to_string())
        );
        assert_eq!(
            HoverService::infer_type_from_script(script, "doubled", BindingType::SetupRef),
            Some("ComputedRef<number>".to_string())
        );
        assert_eq!(
            HoverService::infer_type_from_script(script, "state", BindingType::SetupReactiveConst),
            Some("Reactive<{ id: string }>".to_string())
        );
    }
}
