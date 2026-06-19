use vize_carton::String;
use vize_croquis::TemplateExpression;

/// Compute the enclosing v-if guard shared by all expressions in a v-for scope.
pub(super) fn common_vif_guard_prefix(exprs: &[&TemplateExpression]) -> Option<String> {
    let mut iter = exprs.iter();
    let first = iter.next()?.vif_guard.as_ref()?;
    let mut common: Vec<&str> = split_guard_terms(first.as_str());

    for expr in iter {
        let guard = expr.vif_guard.as_ref()?;
        let terms = split_guard_terms(guard.as_str());
        let shared = common
            .iter()
            .zip(terms.iter())
            .take_while(|(a, b)| a == b)
            .count();
        common.truncate(shared);
        if common.is_empty() {
            return None;
        }
    }

    (!common.is_empty()).then(|| String::from(common.join(" && ").as_str()))
}

fn split_guard_terms(guard: &str) -> Vec<&str> {
    let bytes = guard.as_bytes();
    let mut terms = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b'&' if depth == 0
                && bytes.get(index + 1) == Some(&b'&')
                && index >= 1
                && bytes[index - 1] == b' '
                && bytes.get(index + 2) == Some(&b' ') =>
            {
                terms.push(guard[start..index - 1].trim());
                index += 3;
                start = index;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    terms.push(guard[start..].trim());
    terms
}
