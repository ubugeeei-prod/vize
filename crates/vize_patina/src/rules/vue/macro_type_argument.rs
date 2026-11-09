//! Authored spans for compiler-macro type arguments.

fn inner_offset(authored_call: &str, type_args: &str) -> Option<usize> {
    let inner = type_args
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
        .unwrap_or(type_args);
    let open = authored_call.find('<')?;
    authored_call
        .get(open..)?
        .find(inner)
        .map(|relative| open + relative)
}

pub(super) fn macro_type_argument_span(
    source: &str,
    call: &vize_croquis::macros::MacroCall,
    setup_shift: u32,
) -> Option<(u32, u32)> {
    let type_args = call.type_args.as_deref()?;
    let authored_start = call.start.saturating_sub(setup_shift) as usize;
    let authored_end = call.end.saturating_sub(setup_shift) as usize;
    let authored_call = source.get(authored_start..authored_end)?;
    let relative = inner_offset(authored_call, type_args)? as u32;
    let inner_len = type_args
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
        .unwrap_or(type_args)
        .len() as u32;
    Some((call.start + relative, call.start + relative + inner_len))
}

#[cfg(test)]
mod tests {
    use super::inner_offset;

    #[test]
    fn searches_after_the_macro_name() {
        assert_eq!(inner_offset("defineProps<Props>()", "<Props>"), Some(12));
        assert_eq!(
            inner_offset("defineProps < Props >()", "< Props >"),
            Some(13)
        );
    }
}
