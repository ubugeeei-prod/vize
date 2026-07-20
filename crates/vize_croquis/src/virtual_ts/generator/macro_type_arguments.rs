use vize_carton::String;

use super::VirtualTsGenerator;

impl VirtualTsGenerator {
    /// Emit an internal alias from the type arguments captured for a macro call.
    pub(super) fn emit_macro_type_alias(&mut self, name: &str, type_arguments: &str) {
        let body = type_argument_body(type_arguments);
        let mut alias = String::with_capacity(8 + name.len() + body.len());
        alias.push_str("type ");
        alias.push_str(name);
        alias.push_str(" = ");
        alias.push_str(body);
        alias.push(';');
        self.emit_line(&alias);
    }
}

/// Return the type expression inside a macro call's outer `<...>` delimiters.
///
/// The parser stores the complete type-argument span so source locations remain
/// exact. Generated aliases need only its body because `<Type>` is not a valid
/// type expression on the right-hand side of an alias.
fn type_argument_body(type_arguments: &str) -> &str {
    let trimmed = type_arguments.trim();

    trimmed
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(trimmed)
}

#[cfg(test)]
mod tests {
    use super::type_argument_body;

    #[test]
    fn removes_only_complete_outer_delimiters() {
        assert_eq!(type_argument_body("<Props>"), "Props");
        assert_eq!(
            type_argument_body(" <Record<string, Value>> "),
            "Record<string, Value>"
        );
        assert_eq!(type_argument_body("Props"), "Props");
        assert_eq!(type_argument_body("<>"), "<>");
    }
}
