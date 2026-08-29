use std::path::{Path, PathBuf};
use vize_atelier_core::TemplateSyntaxMode;
use vize_s0::hash::hash_str;

#[derive(Default)]
pub(super) struct BatchStats {
    pub(super) success: usize,
    pub(super) failed: usize,
    pub(super) input_bytes: usize,
    pub(super) output_bytes: usize,
}

impl BatchStats {
    pub(super) fn failed() -> Self {
        Self {
            failed: 1,
            ..Default::default()
        }
    }

    pub(super) fn add(mut self, other: Self) -> Self {
        self.success += other.success;
        self.failed += other.failed;
        self.input_bytes += other.input_bytes;
        self.output_bytes += other.output_bytes;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct BatchCompileKey {
    source_hash: u64,
    source_len: usize,
    parent_hash: u64,
    parent_len: usize,
    component_name_len: usize,
    options: u16,
}

pub(super) struct BatchCompileJob {
    pub(super) path: PathBuf,
    pub(super) source: String,
    pub(super) repeats: usize,
    pub(super) input_bytes: usize,
}

impl BatchCompileJob {
    pub(super) fn single(path: PathBuf, source: String) -> Self {
        let input_bytes = source.len();
        Self {
            path,
            source,
            repeats: 1,
            input_bytes,
        }
    }
}

pub(super) fn batch_options_bits(
    ssr: bool,
    vapor: bool,
    is_ts: bool,
    template_syntax: TemplateSyntaxMode,
    standalone: bool,
    experimental_bits: u16,
) -> u16 {
    u16::from(ssr)
        | (u16::from(vapor) << 1)
        | (u16::from(is_ts) << 2)
        | (u16::from(template_syntax_bits(template_syntax)) << 3)
        | (u16::from(standalone) << 5)
        | experimental_bits
}

fn template_syntax_bits(template_syntax: TemplateSyntaxMode) -> u8 {
    match template_syntax {
        TemplateSyntaxMode::Standard => 0,
        TemplateSyntaxMode::Strict => 1,
        TemplateSyntaxMode::Quirks => 2,
        _ => 3,
    }
}

pub(super) fn should_cache_batch_compile(source: &str, component_name: &str) -> bool {
    if component_name.is_empty() {
        return true;
    }

    !source.contains(component_name)
        && !source.contains(component_name_to_kebab_case(component_name).as_str())
}

fn component_name_to_kebab_case(component_name: &str) -> String {
    let mut out = String::with_capacity(component_name.len());
    for (index, ch) in component_name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index != 0 {
                out.push('-');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

pub(super) fn batch_compile_key(
    path: &Path,
    source: &str,
    component_name: &str,
    option_bits: u16,
) -> BatchCompileKey {
    let (parent_hash, parent_len) = parent_cache_parts(path);
    BatchCompileKey {
        source_hash: hash_str(source),
        source_len: source.len(),
        parent_hash,
        parent_len,
        component_name_len: component_name.len(),
        options: option_bits,
    }
}

fn parent_cache_parts(path: &Path) -> (u64, usize) {
    let Some(parent) = path.parent() else {
        return (hash_str(""), 0);
    };
    let parent = parent.to_string_lossy();
    (hash_str(parent.as_ref()), parent.len())
}
