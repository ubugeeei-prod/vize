use crate::virtual_ts::VizeMapping;
use std::borrow::Cow;
use vize_carton::{String, append};

pub(super) struct ScriptBoundaries<'a>(&'a [usize]);

impl<'a> ScriptBoundaries<'a> {
    pub(super) fn new(boundaries: &'a [usize]) -> Self {
        Self(boundaries)
    }

    pub(super) fn emit_through(
        &mut self,
        offset: usize,
        ts: &mut String,
        mappings: &mut Vec<VizeMapping>,
        source_offset: &dyn Fn(usize) -> usize,
    ) {
        while let Some((&end, rest)) = self.0.split_first()
            && end <= offset
        {
            // An authored EOF is a real syntax boundary. Without this token,
            // `const x =` consumes generated code and its error is unmappable.
            ts.push_str("  ");
            let start = ts.len();
            ts.push(';');
            let source = source_offset(end);
            mappings.push(VizeMapping {
                gen_range: start..ts.len(),
                src_range: source..source,
                sub_spans: Vec::new(),
            });
            ts.push('\n');
            self.0 = rest;
        }
    }
}

pub(super) fn finish_script(
    ts: &mut String,
    class_alias: Option<&str>,
    close_wrapper: bool,
    emitted_default_alias: bool,
    script_gen_start: usize,
    script_len: usize,
) {
    // Finish deferred wrappers even when an authored class/object did not close.
    if let Some(name) = class_alias {
        append!(*ts, "  const __default__ = {name};\n");
    }
    if close_wrapper {
        ts.push_str("  )\n");
    }
    if emitted_default_alias {
        ts.push_str("  void __default__;\n");
    }
    let script_gen_end = ts.len();
    append!(
        *ts,
        "  // @vize-map: {script_gen_start}:{script_gen_end} -> 0:{script_len}\n\n"
    );
}

/// Hoisted statements can share a line with setup declarations. Mask their
/// exact AST spans, keeping bytes and CRLF stable for subsequent source maps.
pub(super) fn setup_line<'a>(
    line: &'a str,
    start: usize,
    spans: &[(u32, u32)],
    index: &mut usize,
) -> Option<Cow<'a, str>> {
    let end = start + line.len();
    while *index < spans.len() && spans[*index].1 as usize <= start {
        *index += 1;
    }
    let overlapping: Vec<_> = spans[*index..]
        .iter()
        .take_while(|&&(left, _)| (left as usize) < end)
        .filter(|&&(left, right)| start < right as usize && end > left as usize)
        .collect();
    if overlapping.is_empty() {
        return Some(Cow::Borrowed(line));
    }
    let mut bytes = line.as_bytes().to_vec();
    for &&(left, right) in &overlapping {
        for byte in
            &mut bytes[(left as usize).saturating_sub(start)..(right as usize).min(end) - start]
        {
            if *byte != b'\r' {
                *byte = b' ';
            }
        }
    }
    if bytes.iter().all(u8::is_ascii_whitespace) {
        return None;
    }
    // Every complete UTF-8 sequence in a statement is replaced with ASCII;
    // AST boundaries never split a code point.
    #[allow(clippy::disallowed_types)]
    Some(Cow::Owned(
        std::string::String::from_utf8(bytes).expect("masked script is UTF-8"),
    ))
}

#[cfg(test)]
mod tests {
    use super::setup_line;

    #[test]
    fn retains_declarations_around_same_line_imports() {
        let source = "const a = 1; import X from 'x'; const b = 2;\r";
        let mut index = 0;
        let masked = setup_line(source, 0, &[(13, 31)], &mut index).unwrap();
        assert_eq!(masked, "const a = 1;                    const b = 2;\r");
    }

    #[test]
    fn whole_module_lines_are_still_omitted() {
        assert!(setup_line("import X from 'x';", 4, &[(4, 22)], &mut 0).is_none());
        assert_eq!(
            setup_line("const a = 1;", 24, &[(4, 22)], &mut 0).unwrap(),
            "const a = 1;"
        );
    }
}
