use vize_croquis::Croquis;

pub(super) fn is_member_root_occurrence(summary: &Croquis, offset: u32, name: &str) -> bool {
    for expr in &summary.template_expressions {
        if offset < expr.start {
            continue;
        }
        let local = (offset - expr.start) as usize;
        let source = expr.content.as_str();
        if local + name.len() > source.len() || source.get(local..local + name.len()) != Some(name)
        {
            continue;
        }
        let tail = source[local + name.len()..].trim_start();
        if tail.starts_with('.') || tail.starts_with("?.") || tail.starts_with('[') {
            return true;
        }
    }
    false
}
